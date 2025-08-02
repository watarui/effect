//! `PostgreSQL` を使用したユーザーリポジトリの実装

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common_types::UserId;
use infrastructure::{
    Entity,
    Repository,
    RepositoryError as RepoError,
    SoftDeletable,
    count,
    delete,
    exists,
    restore,
    select_all,
    select_by_id,
    select_by_id_with_deleted,
    select_by_ids,
    select_deleted,
    soft_delete,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::{
        aggregates::user::User,
        value_objects::{
            account_status::AccountStatus,
            email::Email,
            user_profile::UserProfile,
            user_role::UserRole,
        },
    },
    ports::outbound::UserRepository,
};

/// `PostgreSQL` を使用したユーザーリポジトリの実装
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    /// 新しいインスタンスを作成
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// User エンティティの Entity トレイト実装
impl Entity for User {
    type Id = UserId;

    fn id(&self) -> &Self::Id {
        self.id()
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at()
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at()
    }

    fn version(&self) -> u64 {
        self.version()
    }

    fn increment_version(&mut self) {
        // User の内部状態を変更するメソッドが必要
        // 今は version フィールドが private なので、ドメインモデルに追加が必要
        // 暫定的に何もしない
    }

    fn touch(&mut self) {
        // User の内部状態を変更するメソッドが必要
        // 今は updated_at フィールドが private
        // なので、ドメインモデルに追加が必要 暫定的に何もしない
    }
}

/// `sqlx::Row` から User への変換
fn map_row_to_user(row: &sqlx::postgres::PgRow) -> Result<User, sqlx::Error> {
    let id_bytes: Vec<u8> = row.try_get("id")?;
    let id_uuid = Uuid::from_slice(&id_bytes).map_err(|e| {
        sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid UUID: {e}"),
        )))
    })?;
    let id = UserId::from(id_uuid);

    let email_str: String = row.try_get("email")?;
    let email = Email::new(&email_str).map_err(|e| {
        sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid email: {e}"),
        )))
    })?;

    let display_name: String = row.try_get("display_name")?;
    let profile = UserProfile::new(&display_name).map_err(|e| {
        sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid profile: {e:?}"),
        )))
    })?;

    let role_str: String = row.try_get("role")?;
    let role = match role_str.as_str() {
        "admin" => UserRole::Admin,
        _ => UserRole::User,
    };

    let status_str: String = row.try_get("status")?;
    let status = match status_str.as_str() {
        "active" => AccountStatus::Active,
        "deleted" => AccountStatus::Deleted,
        _ => AccountStatus::default(),
    };

    let created_at: DateTime<Utc> = row.try_get("created_at")?;
    let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
    let version: i64 = row.try_get("version")?;

    // User構造体を再構築
    // Note: User の内部フィールドは private なので、serde を使って再構築
    #[allow(clippy::cast_sign_loss)]
    let version_u64 = version as u64;
    let user_data = serde_json::json!({
        "id": id,
        "email": email,
        "profile": profile,
        "role": role,
        "status": status,
        "created_at": created_at,
        "updated_at": updated_at,
        "version": version_u64,
    });

    serde_json::from_value(user_data).map_err(|e| {
        sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to deserialize user: {e}"),
        )))
    })
}

#[async_trait]
impl Repository<User> for PostgresUserRepository {
    async fn save(&self, entity: &User) -> Result<(), RepoError> {
        // 新規作成か更新かを判定
        let exists = exists!(
            table: "users",
            id_column: "id",
            id: entity.id().as_bytes(),
            pool: &self.pool
        )?;

        // 共通の値を先に計算
        let role_str = match entity.role() {
            UserRole::Admin => "admin",
            UserRole::User => "user",
        };
        let status_str = match entity.status() {
            AccountStatus::Active => "active",
            AccountStatus::Deleted => "deleted",
        };

        if exists {
            // 更新

            let query = r"
                UPDATE users
                SET email = $1, display_name = $2, role = $3, status = $4, updated_at = $5, version = version + 1
                WHERE id = $6 AND version = $7 AND deleted_at IS NULL
                RETURNING version
            ";

            let now = Utc::now();
            #[allow(clippy::cast_possible_wrap)]
            let version_i64 = entity.version() as i64;

            let new_version: Option<i64> = sqlx::query_scalar(query)
                .bind(entity.email().as_str())
                .bind(entity.profile().display_name())
                .bind(role_str)
                .bind(status_str)
                .bind(now)
                .bind(entity.id().as_bytes())
                .bind(version_i64)
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from_sqlx)?;

            if new_version.is_some() {
                return Ok(());
            }

            // バージョン不一致または存在しない
            let check_query = "SELECT version FROM users WHERE id = $1 AND deleted_at IS NULL";
            let actual_version: Option<i64> = sqlx::query_scalar(check_query)
                .bind(entity.id().as_bytes())
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from_sqlx)?;

            return actual_version.map_or_else(
                || Err(RepoError::not_found("User", entity.id())),
                |v| {
                    #[allow(clippy::cast_sign_loss)]
                    let actual = v as u64;
                    Err(RepoError::optimistic_lock_failure(entity.version(), actual))
                },
            );
        }

        // 新規作成
        let query = r"
            INSERT INTO users (id, email, display_name, role, status, created_at, updated_at, version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ";

        let now = Utc::now();

        sqlx::query(query)
            .bind(entity.id().as_bytes())
            .bind(entity.email().as_str())
            .bind(entity.profile().display_name())
            .bind(role_str)
            .bind(status_str)
            .bind(now)
            .bind(now)
            .bind(1_i64)
            .execute(&self.pool)
            .await
            .map_err(RepoError::from_sqlx)?;

        Ok(())
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepoError> {
        select_by_id!(
            table: "users",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool,
            mapper: |row| map_row_to_user(&row)
        )
    }

    async fn delete(&self, id: &UserId) -> Result<(), RepoError> {
        delete!(
            table: "users",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool
        )
    }

    async fn exists(&self, id: &UserId) -> Result<bool, RepoError> {
        exists!(
            table: "users",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool
        )
    }

    async fn find_by_ids(&self, ids: &[UserId]) -> Result<Vec<User>, RepoError> {
        let id_bytes: Vec<&[u8]> = ids.iter().map(common_types::UserId::as_bytes).collect();
        select_by_ids!(
            table: "users",
            id_column: "id",
            ids: &id_bytes,
            pool: &self.pool,
            mapper: |row| map_row_to_user(&row)
        )
    }

    async fn find_all(&self) -> Result<Vec<User>, RepoError> {
        select_all!(
            table: "users",
            pool: &self.pool,
            mapper: |row| map_row_to_user(&row)
        )
    }

    async fn count(&self) -> Result<i64, RepoError> {
        count!(table: "users", pool: &self.pool)
    }
}

#[async_trait]
impl SoftDeletable<User> for PostgresUserRepository {
    async fn soft_delete(&self, id: &UserId) -> Result<(), RepoError> {
        soft_delete!(
            table: "users",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool
        )
    }

    async fn restore(&self, id: &UserId) -> Result<(), RepoError> {
        restore!(
            table: "users",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool
        )
    }

    async fn find_by_id_with_deleted(&self, id: &UserId) -> Result<Option<User>, RepoError> {
        select_by_id_with_deleted!(
            table: "users",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool,
            mapper: |row| map_row_to_user(&row)
        )
    }

    async fn find_deleted(&self) -> Result<Vec<User>, RepoError> {
        select_deleted!(
            table: "users",
            pool: &self.pool,
            mapper: |row| map_row_to_user(&row)
        )
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    type Error = RepoError;

    async fn save(&self, user: &User) -> Result<(), Self::Error> {
        Repository::save(self, user).await
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, Self::Error> {
        Repository::find_by_id(self, id).await
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Self::Error> {
        let query = r"
            SELECT * FROM users 
            WHERE email = $1 AND deleted_at IS NULL
        ";

        sqlx::query(query)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepoError::from_sqlx)?
            .map(|row| map_row_to_user(&row))
            .transpose()
            .map_err(RepoError::from_sqlx)
    }

    async fn delete(&self, id: &UserId) -> Result<(), Self::Error> {
        // ソフトデリートを使用
        self.soft_delete(id).await
    }

    async fn is_first_user(&self) -> Result<bool, Self::Error> {
        let count = self.count().await?;
        Ok(count == 0)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::postgres::PgPoolOptions;

    use super::*;

    #[tokio::test]
    async fn test_user_repository_operations() {
        // This is an integration test that requires a real database
        // Skip if DATABASE_URL is not set
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("Skipping integration test: DATABASE_URL not set");
            return;
        };

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap();

        let repo = PostgresUserRepository::new(pool);

        // Create a test user
        let user_id = UserId::new();
        let email = Email::new("test@example.com").unwrap();
        let user = User::create(user_id, email, "Test User", false).unwrap();

        // Test save
        Repository::save(&repo, &user).await.unwrap();

        // Test find_by_id
        let found = Repository::find_by_id(&repo, &user_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), &user_id);

        // Test find_by_email
        let found = repo.find_by_email("test@example.com").await.unwrap();
        assert!(found.is_some());

        // Test soft delete
        repo.soft_delete(&user_id).await.unwrap();

        // Verify user is not found after soft delete
        let found = Repository::find_by_id(&repo, &user_id).await.unwrap();
        assert!(found.is_none());

        // But can be found with deleted
        let found = SoftDeletable::find_by_id_with_deleted(&repo, &user_id)
            .await
            .unwrap();
        assert!(found.is_some());

        // Test restore
        SoftDeletable::restore(&repo, &user_id).await.unwrap();

        // Verify user is found after restore
        let found = Repository::find_by_id(&repo, &user_id).await.unwrap();
        assert!(found.is_some());

        // Clean up
        Repository::delete(&repo, &user_id).await.unwrap();
    }
}
