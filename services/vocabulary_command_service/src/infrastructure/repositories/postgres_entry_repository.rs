use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::{EntryId, Spelling, Version, VocabularyEntry},
    error::{Error, Result},
    ports::repositories::VocabularyEntryRepository,
};

/// PostgreSQL 実装の VocabularyEntryRepository
#[derive(Clone)]
pub struct PostgresVocabularyEntryRepository {
    pool: PgPool,
}

impl PostgresVocabularyEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VocabularyEntryRepository for PostgresVocabularyEntryRepository {
    async fn find_by_id(&self, entry_id: &EntryId) -> Result<Option<VocabularyEntry>> {
        let row = sqlx::query(
            r#"
            SELECT 
                entry_id,
                spelling,
                version,
                created_at,
                updated_at
            FROM vocabulary_entries
            WHERE entry_id = $1
            "#,
        )
        .bind(entry_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        match row {
            Some(row) => {
                let entry = VocabularyEntry {
                    entry_id:   EntryId::from_uuid(row.get::<Uuid, _>("entry_id")),
                    spelling:   Spelling::new(row.get::<String, _>("spelling"))
                        .map_err(Error::Validation)?,
                    version:    Version::new(row.get::<i64, _>("version"))
                        .map_err(Error::Validation)?,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                };
                Ok(Some(entry))
            },
            None => Ok(None),
        }
    }

    async fn exists(&self, entry_id: &EntryId) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM vocabulary_entries
                WHERE entry_id = $1
            )
            "#,
        )
        .bind(entry_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        Ok(exists)
    }

    async fn save(&self, entry: &VocabularyEntry) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO vocabulary_entries (
                entry_id,
                spelling,
                version,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (entry_id) 
            DO UPDATE SET
                spelling = EXCLUDED.spelling,
                version = EXCLUDED.version,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(entry.entry_id.as_uuid())
        .bind(entry.spelling.value())
        .bind(entry.version.value())
        .bind(entry.created_at)
        .bind(entry.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        Ok(())
    }

    async fn find_by_spelling(
        &self,
        spelling: &crate::domain::Spelling,
    ) -> Result<Option<VocabularyEntry>> {
        let row = sqlx::query(
            r#"
            SELECT 
                entry_id,
                spelling,
                version,
                created_at,
                updated_at
            FROM vocabulary_entries
            WHERE spelling = $1
            LIMIT 1
            "#,
        )
        .bind(spelling.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        match row {
            Some(row) => {
                let entry = VocabularyEntry {
                    entry_id:   EntryId::from_uuid(row.get::<Uuid, _>("entry_id")),
                    spelling:   Spelling::new(row.get::<String, _>("spelling"))
                        .map_err(Error::Validation)?,
                    version:    Version::new(row.get::<i64, _>("version"))
                        .map_err(Error::Validation)?,
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
                };
                Ok(Some(entry))
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
    async fn test_entry_repository_operations() {
        // テスト用のデータベース接続
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://effect:effect_password@localhost:5432/effect_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let repo = PostgresVocabularyEntryRepository::new(pool.clone());

        // テストデータ
        let entry = VocabularyEntry {
            entry_id:   EntryId::new(),
            spelling:   Spelling::new("test".to_string()).unwrap(),
            version:    Version::initial(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // 保存テスト
        repo.save(&entry).await.expect("Failed to save entry");

        // 存在確認テスト
        let exists = repo
            .exists(&entry.entry_id)
            .await
            .expect("Failed to check existence");
        assert!(exists);

        // ID検索テスト
        let found = repo
            .find_by_id(&entry.entry_id)
            .await
            .expect("Failed to find by id");
        assert!(found.is_some());
        let found_entry = found.unwrap();
        assert_eq!(found_entry.spelling.value(), entry.spelling.value());

        // スペリング検索テスト
        let found = repo
            .find_by_spelling(&entry.spelling)
            .await
            .expect("Failed to find by spelling");
        assert!(found.is_some());

        // クリーンアップ
        sqlx::query("DELETE FROM vocabulary_entries WHERE entry_id = $1")
            .bind(entry.entry_id.as_uuid())
            .execute(&pool)
            .await
            .expect("Failed to clean up");
    }
}
