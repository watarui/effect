//! PostgreSQL データソースリポジトリ

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{domain::VocabularySearchItem, error::Result, ports::outbound::DataSourceRepository};

/// PostgreSQL データソースリポジトリ
pub struct PostgresDataSourceRepository {
    pool: PgPool,
}

impl PostgresDataSourceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DataSourceRepository for PostgresDataSourceRepository {
    async fn get_all_items(&self) -> Result<Vec<VocabularySearchItem>> {
        let items = sqlx::query_as!(
            VocabularySearchItem,
            r#"
            SELECT 
                item_id,
                entry_id,
                spelling,
                disambiguation,
                part_of_speech,
                definition,
                ipa_pronunciation,
                cefr_level,
                frequency_rank,
                example_count,
                0.0::float4 as "score!: f32",
                created_at,
                updated_at
            FROM vocabulary_items_read
            WHERE NOT is_deleted
                AND is_published
            ORDER BY created_at
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    async fn get_updated_items(&self, since: DateTime<Utc>) -> Result<Vec<VocabularySearchItem>> {
        let items = sqlx::query_as!(
            VocabularySearchItem,
            r#"
            SELECT 
                item_id,
                entry_id,
                spelling,
                disambiguation,
                part_of_speech,
                definition,
                ipa_pronunciation,
                cefr_level,
                frequency_rank,
                example_count,
                0.0::float4 as "score!: f32",
                created_at,
                updated_at
            FROM vocabulary_items_read
            WHERE updated_at > $1
                AND NOT is_deleted
                AND is_published
            ORDER BY updated_at
            "#,
            since
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    async fn get_item_by_id(&self, item_id: Uuid) -> Result<Option<VocabularySearchItem>> {
        let item = sqlx::query_as!(
            VocabularySearchItem,
            r#"
            SELECT 
                item_id,
                entry_id,
                spelling,
                disambiguation,
                part_of_speech,
                definition,
                ipa_pronunciation,
                cefr_level,
                frequency_rank,
                example_count,
                0.0::float4 as "score!: f32",
                created_at,
                updated_at
            FROM vocabulary_items_read
            WHERE item_id = $1
                AND NOT is_deleted
                AND is_published
            "#,
            item_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(item)
    }
}
