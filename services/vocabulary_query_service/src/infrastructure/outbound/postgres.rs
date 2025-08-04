//! PostgreSQL リポジトリ実装
//!
//! Read Model の取得

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared_error::{DomainError, DomainResult};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::read_models::{VocabularyEntryView, VocabularyItemView, VocabularyStats},
    ports::outbound::ReadModelRepository,
};

/// PostgreSQL Read Model リポジトリ
pub struct PostgresReadModelRepository {
    pool: PgPool,
}

impl PostgresReadModelRepository {
    /// 新しいリポジトリを作成
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReadModelRepository for PostgresReadModelRepository {
    async fn get_item(&self, item_id: Uuid) -> DomainResult<Option<VocabularyItemView>> {
        // vocabulary_items_view から項目を取得し、シンプルなモデルにマッピング
        let row = sqlx::query(
            r#"
            SELECT 
                item_id,
                entry_id,
                spelling,
                disambiguation,
                definition_count,
                example_count,
                quality_score,
                status,
                created_at,
                last_modified_at,
                version
            FROM vocabulary_items_view
            WHERE item_id = $1
            "#,
        )
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(|r| VocabularyItemView {
            item_id:          r.get("item_id"),
            entry_id:         r.get("entry_id"),
            spelling:         r.get("spelling"),
            disambiguation:   r.get("disambiguation"),
            definition_count: r.get("definition_count"),
            example_count:    r.get("example_count"),
            quality_score:    r.get("quality_score"),
            status:           r.get("status"),
            created_at:       r.get("created_at"),
            last_modified_at: r.get("last_modified_at"),
            version:          r.get::<i32, _>("version") as i64,
        }))
    }

    async fn get_entry(&self, entry_id: Uuid) -> DomainResult<Option<VocabularyEntryView>> {
        // vocabulary_items_view から entry_id でグループ化してエントリー情報を取得
        let row = sqlx::query(
            r#"
            SELECT 
                entry_id,
                spelling,
                MIN(created_at) as created_at,
                MAX(last_modified_at) as updated_at,
                COUNT(DISTINCT item_id) as item_count
            FROM vocabulary_items_view
            WHERE entry_id = $1
            GROUP BY entry_id, spelling
            "#,
        )
        .bind(entry_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(|r| VocabularyEntryView {
            entry_id:       r.get("entry_id"),
            spelling:       r.get("spelling"),
            part_of_speech: String::from("noun"), // TODO: 実際の品詞情報を取得する必要がある
            item_count:     r.get::<i64, _>("item_count") as i32,
            created_at:     r.get("created_at"),
            updated_at:     r.get("updated_at"),
        }))
    }

    async fn get_stats(&self) -> DomainResult<VocabularyStats> {
        // 統計情報を集計
        let stats = sqlx::query(
            r#"
            SELECT 
                COUNT(DISTINCT entry_id) as total_entries,
                COUNT(DISTINCT item_id) as total_items,
                COALESCE(SUM(example_count), 0) as total_examples,
                MAX(last_modified_at) as last_updated
            FROM vocabulary_items_view
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(VocabularyStats {
            total_entries:  stats.get::<i64, _>("total_entries"),
            total_items:    stats.get::<i64, _>("total_items"),
            total_examples: stats.get::<i64, _>("total_examples"),
            last_updated:   stats
                .get::<Option<DateTime<Utc>>, _>("last_updated")
                .unwrap_or_else(chrono::Utc::now),
        })
    }
}
