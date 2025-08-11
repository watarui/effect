use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::{
        Disambiguation,
        EntryId,
        ItemId,
        Spelling,
        Version,
        VocabularyItem,
        VocabularyStatus,
    },
    error::{Error, Result},
    ports::repositories::VocabularyItemRepository,
};

/// PostgreSQL 実装の VocabularyItemRepository
#[derive(Clone)]
pub struct PostgresVocabularyItemRepository {
    pool: PgPool,
}

impl PostgresVocabularyItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VocabularyItemRepository for PostgresVocabularyItemRepository {
    async fn find_by_id(&self, item_id: &ItemId) -> Result<Option<VocabularyItem>> {
        let row = sqlx::query(
            r#"
            SELECT 
                item_id,
                entry_id,
                spelling,
                disambiguation,
                is_primary,
                status,
                created_at,
                updated_at,
                version
            FROM vocabulary_items
            WHERE item_id = $1
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        match row {
            Some(row) => {
                let item = VocabularyItem {
                    item_id:        ItemId::from_uuid(row.get::<Uuid, _>("item_id")),
                    entry_id:       EntryId::from_uuid(row.get::<Uuid, _>("entry_id")),
                    spelling:       Spelling::new(row.get::<String, _>("spelling"))
                        .map_err(Error::Validation)?,
                    disambiguation: Disambiguation::new(
                        row.get::<Option<String>, _>("disambiguation"),
                    )
                    .map_err(Error::Validation)?,
                    is_primary:     row.get::<bool, _>("is_primary"),
                    status:         match row.get::<String, _>("status").as_str() {
                        "draft" => VocabularyStatus::Draft,
                        "pending_ai" => VocabularyStatus::PendingAI,
                        "published" => VocabularyStatus::Published,
                        _ => {
                            return Err(Error::DatabaseString(format!(
                                "Invalid status value: {}",
                                row.get::<String, _>("status")
                            )));
                        },
                    },
                    created_at:     row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at:     row.get::<DateTime<Utc>, _>("updated_at"),
                    version:        Version::new(row.get::<i64, _>("version"))
                        .map_err(Error::Validation)?,
                };
                Ok(Some(item))
            },
            None => Ok(None),
        }
    }

    async fn save(&self, item: &VocabularyItem) -> Result<()> {
        // 楽観的ロック: バージョンチェック付きの更新
        let result = sqlx::query(
            r#"
            INSERT INTO vocabulary_items (
                item_id,
                entry_id,
                spelling,
                disambiguation,
                is_primary,
                status,
                created_at,
                updated_at,
                version
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (item_id) 
            DO UPDATE SET
                spelling = EXCLUDED.spelling,
                disambiguation = EXCLUDED.disambiguation,
                is_primary = EXCLUDED.is_primary,
                status = EXCLUDED.status,
                updated_at = EXCLUDED.updated_at,
                version = EXCLUDED.version
            WHERE vocabulary_items.version = $9 - 1
            "#,
        )
        .bind(item.item_id.as_uuid())
        .bind(item.entry_id.as_uuid())
        .bind(item.spelling.value())
        .bind(item.disambiguation.as_option())
        .bind(item.is_primary)
        .bind(item.status.as_str())
        .bind(item.created_at)
        .bind(item.updated_at)
        .bind(item.version.value())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        // 更新された行数をチェック（楽観的ロックの確認）
        if result.rows_affected() == 0 {
            // バージョンが一致しない場合は、競合エラー
            return Err(Error::Conflict(
                "Version conflict: the item has been modified by another process".to_string(),
            ));
        }

        Ok(())
    }

    async fn find_by_entry_id(&self, entry_id: &EntryId) -> Result<Vec<VocabularyItem>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                item_id,
                entry_id,
                spelling,
                disambiguation,
                is_primary,
                status,
                created_at,
                updated_at,
                version
            FROM vocabulary_items
            WHERE entry_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(entry_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            let item = VocabularyItem {
                item_id:        ItemId::from_uuid(row.get::<Uuid, _>("item_id")),
                entry_id:       EntryId::from_uuid(row.get::<Uuid, _>("entry_id")),
                spelling:       Spelling::new(row.get::<String, _>("spelling"))
                    .map_err(Error::Validation)?,
                disambiguation: Disambiguation::new(row.get::<Option<String>, _>("disambiguation"))
                    .map_err(Error::Validation)?,
                is_primary:     row.get::<bool, _>("is_primary"),
                status:         match row.get::<String, _>("status").as_str() {
                    "draft" => VocabularyStatus::Draft,
                    "pending_ai" => VocabularyStatus::PendingAI,
                    "published" => VocabularyStatus::Published,
                    _ => return Err(Error::DatabaseString("Invalid status value".to_string())),
                },
                created_at:     row.get::<DateTime<Utc>, _>("created_at"),
                updated_at:     row.get::<DateTime<Utc>, _>("updated_at"),
                version:        Version::new(row.get::<i64, _>("version"))
                    .map_err(Error::Validation)?,
            };
            items.push(item);
        }

        Ok(items)
    }

    // find_by_spelling is not in the trait
    // async fn find_by_spelling(
    // &self,
    // spelling: &str,
    // disambiguation: Option<&str>,
    // ) -> Result<Option<VocabularyItem>> {
    // let query = if let Some(disambiguation) = disambiguation {
    // sqlx::query(
    // r#"
    // SELECT
    // item_id,
    // entry_id,
    // spelling,
    // disambiguation,
    // status,
    // version,
    // created_at,
    // updated_at
    // FROM vocabulary_items
    // WHERE spelling = $1 AND disambiguation = $2
    // LIMIT 1
    // "#,
    // )
    // .bind(spelling)
    // .bind(disambiguation)
    // } else {
    // sqlx::query(
    // r#"
    // SELECT
    // item_id,
    // entry_id,
    // spelling,
    // disambiguation,
    // status,
    // version,
    // created_at,
    // updated_at
    // FROM vocabulary_items
    // WHERE spelling = $1 AND disambiguation IS NULL
    // LIMIT 1
    // "#,
    // )
    // .bind(spelling)
    // };
    //
    // let row = query
    // .fetch_optional(&self.pool)
    // .await
    // .map_err(|e| Error::DatabaseString(e.to_string()))?;
    //
    // match row {
    // Some(row) => {
    // let item = VocabularyItem {
    // item_id: ItemId::from_uuid(row.get::<Uuid, _>("item_id")),
    // entry_id: EntryId::from_uuid(row.get::<Uuid, _>("entry_id")),
    // spelling: Spelling::new(row.get::<String, _>("spelling"))
    // .map_err(|e| Error::Validation(e))?,
    // disambiguation: Disambiguation::new(row.get::<Option<String>,
    // _>("disambiguation")) .map_err(|e| Error::Validation(e))?,
    // is_primary: row.get::<bool, _>("is_primary"),
    // status: match row.get::<String, _>("status").as_str() {
    // "draft" => VocabularyStatus::Draft,
    // "pending_ai" => VocabularyStatus::PendingAI,
    // "published" => VocabularyStatus::Published,
    // _ => return Err(Error::DatabaseString(format!("Invalid status value: {}",
    // row.get::<String, _>("status")))), },
    // created_at: row.get::<DateTime<Utc>, _>("created_at"),
    // updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    // version: Version::new(row.get::<i64, _>("version"))
    // .map_err(|e| Error::Validation(e))?,
    // };
    // Ok(Some(item))
    // }
    // None => Ok(None),
    // }
    // }

    // delete is not in the trait
    // async fn delete(&self, item_id: &ItemId) -> Result<()> {
    // sqlx::query(
    // r#"
    // DELETE FROM vocabulary_items
    // WHERE item_id = $1
    // "#,
    // )
    // .bind(item_id.as_uuid())
    // .execute(&self.pool)
    // .await
    // .map_err(|e| Error::DatabaseString(e.to_string()))?;
    //
    // Ok(())
    // }

    async fn find_primary_by_entry_id(&self, entry_id: &EntryId) -> Result<Option<VocabularyItem>> {
        // 同じエントリに対して disambiguation が NULL のアイテムを主要アイテムとする
        let row = sqlx::query(
            r#"
            SELECT 
                item_id,
                entry_id,
                spelling,
                disambiguation,
                is_primary,
                status,
                created_at,
                updated_at,
                version
            FROM vocabulary_items
            WHERE entry_id = $1 AND disambiguation IS NULL
            LIMIT 1
            "#,
        )
        .bind(entry_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        match row {
            Some(row) => {
                let item = VocabularyItem {
                    item_id:        ItemId::from_uuid(row.get::<Uuid, _>("item_id")),
                    entry_id:       EntryId::from_uuid(row.get::<Uuid, _>("entry_id")),
                    spelling:       Spelling::new(row.get::<String, _>("spelling"))
                        .map_err(Error::Validation)?,
                    disambiguation: Disambiguation::new(
                        row.get::<Option<String>, _>("disambiguation"),
                    )
                    .map_err(Error::Validation)?,
                    is_primary:     row.get::<bool, _>("is_primary"),
                    status:         match row.get::<String, _>("status").as_str() {
                        "draft" => VocabularyStatus::Draft,
                        "pending_ai" => VocabularyStatus::PendingAI,
                        "published" => VocabularyStatus::Published,
                        _ => {
                            return Err(Error::DatabaseString(format!(
                                "Invalid status value: {}",
                                row.get::<String, _>("status")
                            )));
                        },
                    },
                    created_at:     row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at:     row.get::<DateTime<Utc>, _>("updated_at"),
                    version:        Version::new(row.get::<i64, _>("version"))
                        .map_err(Error::Validation)?,
                };
                Ok(Some(item))
            },
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::postgres::PgPoolOptions;

    use super::*;

    #[tokio::test]
    #[ignore] // 統合テストは明示的に実行
    async fn test_item_repository_operations() {
        // テスト用のデータベース接続
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://effect:effect_password@localhost:5432/effect_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let repo = PostgresVocabularyItemRepository::new(pool.clone());

        // テストデータ
        let entry_id = EntryId::new();
        let spelling = Spelling::new("test".to_string()).unwrap();
        let disambiguation = Disambiguation::new(Some("first meaning".to_string())).unwrap();
        let item = VocabularyItem::create(entry_id, spelling, disambiguation);

        // 保存テスト
        repo.save(&item).await.expect("Failed to save item");

        // ID検索テスト
        let found = repo
            .find_by_id(&item.item_id)
            .await
            .expect("Failed to find by id");
        assert!(found.is_some());
        let found_item = found.unwrap();
        assert_eq!(found_item.spelling.value(), item.spelling.value());

        // エントリID検索テスト
        let items = repo
            .find_by_entry_id(&entry_id)
            .await
            .expect("Failed to find by entry id");
        assert_eq!(items.len(), 1);

        // 主要アイテム検索テスト
        let found = repo
            .find_primary_by_entry_id(&entry_id)
            .await
            .expect("Failed to find primary item");
        // デフォルトでは is_primary = false なので None のはず
        assert!(found.is_none());
    }

    #[tokio::test]
    #[ignore] // 統合テストは明示的に実行
    async fn test_optimistic_locking() {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://effect:effect_password@localhost:5432/effect_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let repo = PostgresVocabularyItemRepository::new(pool.clone());

        // 初期データの作成
        let entry_id = EntryId::new();
        let spelling = Spelling::new("conflict_test".to_string()).unwrap();
        let disambiguation = Disambiguation::new(None).unwrap();
        let mut item = VocabularyItem::create(entry_id, spelling, disambiguation);

        // 初回保存
        repo.save(&item).await.expect("Failed to save initial item");

        // 別のプロセスで更新をシミュレート
        let mut item_clone = item.clone();
        item_clone
            .update_disambiguation(Disambiguation::new(Some("updated".to_string())).unwrap())
            .unwrap();
        repo.save(&item_clone)
            .await
            .expect("Failed to save updated item");

        // 古いバージョンで更新を試みる（失敗するはず）
        item.update_disambiguation(Disambiguation::new(Some("conflicting".to_string())).unwrap())
            .unwrap();
        let result = repo.save(&item).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Conflict(_)));

        // クリーンアップ
        sqlx::query("DELETE FROM vocabulary_items WHERE item_id = $1")
            .bind(item.item_id.as_uuid())
            .execute(&pool)
            .await
            .expect("Failed to clean up");
    }
}
