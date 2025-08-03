//! `PostgreSQL` を使用したユーザーリポジトリの実装

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common_types::UserId;
use shared_repository::{
    Entity,
    Error as RepoError,
    Repository,
    SoftDeletable,
    count,
    delete,
    exists,
    insert,
    restore,
    select_all,
    select_by_id,
    select_by_id_with_deleted,
    select_by_ids,
    select_deleted,
    soft_delete,
    update,
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
    let id_uuid: Uuid = row.try_get("id")?;
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
        // User エンティティ用のダミー実装を追加
        // User の increment_version() と touch() が実際に動作しないため、
        // 一時的な wrapper を作成
        struct UserWrapper<'a> {
            user:         &'a User,
            email:        String,
            display_name: String,
            role:         String,
            status:       String,
        }

        impl Entity for UserWrapper<'_> {
            type Id = UserId;

            fn id(&self) -> &Self::Id {
                self.user.id()
            }

            fn version(&self) -> u64 {
                self.user.version()
            }

            fn created_at(&self) -> DateTime<Utc> {
                self.user.created_at()
            }

            fn updated_at(&self) -> DateTime<Utc> {
                self.user.updated_at()
            }

            fn increment_version(&mut self) {
                // User の private フィールドのため実装不可
            }

            fn touch(&mut self) {
                // User の private フィールドのため実装不可
            }
        }

        let wrapper = UserWrapper {
            user:         entity,
            email:        entity.email().as_str().to_string(),
            display_name: entity.profile().display_name().to_string(),
            role:         match entity.role() {
                UserRole::Admin => "admin".to_string(),
                UserRole::User => "user".to_string(),
            },
            status:       match entity.status() {
                AccountStatus::Active => "active".to_string(),
                AccountStatus::Deleted => "deleted".to_string(),
            },
        };

        let exists = exists!(
            table: "users",
            id_column: "id",
            id: entity.id().as_uuid(),
            pool: &self.pool
        )?;

        if exists {
            update!(
                table: "users",
                entity: wrapper,
                id_column: "id",
                columns: [email, display_name, role, status],
                pool: &self.pool
            )
        } else {
            insert!(
                table: "users",
                entity: wrapper,
                columns: [email, display_name, role, status],
                pool: &self.pool
            )
        }
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepoError> {
        select_by_id!(
            table: "users",
            id_column: "id",
            id: id.as_uuid(),
            pool: &self.pool,
            mapper: |row| map_row_to_user(&row)
        )
    }

    async fn delete(&self, id: &UserId) -> Result<(), RepoError> {
        delete!(
            table: "users",
            id_column: "id",
            id: id.as_uuid(),
            pool: &self.pool
        )
    }

    async fn exists(&self, id: &UserId) -> Result<bool, RepoError> {
        exists!(
            table: "users",
            id_column: "id",
            id: id.as_uuid(),
            pool: &self.pool
        )
    }

    async fn find_by_ids(&self, ids: &[UserId]) -> Result<Vec<User>, RepoError> {
        let uuids: Vec<&Uuid> = ids.iter().map(common_types::UserId::as_uuid).collect();
        select_by_ids!(
            table: "users",
            id_column: "id",
            ids: &uuids,
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
            id: id.as_uuid(),
            pool: &self.pool
        )
    }

    async fn restore(&self, id: &UserId) -> Result<(), RepoError> {
        restore!(
            table: "users",
            id_column: "id",
            id: id.as_uuid(),
            pool: &self.pool
        )
    }

    async fn find_by_id_with_deleted(&self, id: &UserId) -> Result<Option<User>, RepoError> {
        select_by_id_with_deleted!(
            table: "users",
            id_column: "id",
            id: id.as_uuid(),
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
