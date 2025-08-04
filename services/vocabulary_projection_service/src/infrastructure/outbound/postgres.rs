//! PostgreSQL リポジトリ実装
//!
//! Read Model の永続化

use async_trait::async_trait;
use shared_error::{DomainError, DomainResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::read_models::{ProjectionState, VocabularyItemView},
    ports::outbound::{ProjectionStateRepository, ReadModelRepository},
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
    async fn save_item_view(&self, view: &VocabularyItemView) -> DomainResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO vocabulary_items_view (
                item_id, entry_id, spelling, disambiguation,
                pronunciation, phonetic_respelling, audio_url,
                register, cefr_level,
                definitions, synonyms, antonyms, collocations,
                definition_count, example_count, quality_score,
                status, created_by_type, created_by_id,
                created_at, last_modified_at, last_modified_by, version
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                $10, $11, $12, $13, $14, $15, $16,
                $17, $18, $19, $20, $21, $22, $23
            )
            ON CONFLICT (item_id) DO UPDATE SET
                spelling = EXCLUDED.spelling,
                disambiguation = EXCLUDED.disambiguation,
                pronunciation = EXCLUDED.pronunciation,
                phonetic_respelling = EXCLUDED.phonetic_respelling,
                audio_url = EXCLUDED.audio_url,
                register = EXCLUDED.register,
                cefr_level = EXCLUDED.cefr_level,
                definitions = EXCLUDED.definitions,
                synonyms = EXCLUDED.synonyms,
                antonyms = EXCLUDED.antonyms,
                collocations = EXCLUDED.collocations,
                definition_count = EXCLUDED.definition_count,
                example_count = EXCLUDED.example_count,
                quality_score = EXCLUDED.quality_score,
                status = EXCLUDED.status,
                last_modified_at = EXCLUDED.last_modified_at,
                last_modified_by = EXCLUDED.last_modified_by,
                version = EXCLUDED.version
            "#,
            view.item_id,
            view.entry_id,
            view.spelling,
            view.disambiguation,
            view.pronunciation,
            view.phonetic_respelling,
            view.audio_url,
            view.register,
            view.cefr_level,
            view.definitions as _,
            view.synonyms as _,
            view.antonyms as _,
            view.collocations as _,
            view.definition_count,
            view.example_count,
            view.quality_score as _,
            view.status,
            view.created_by_type,
            view.created_by_id,
            view.created_at,
            view.last_modified_at,
            view.last_modified_by,
            view.version
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn get_item_view(&self, item_id: Uuid) -> DomainResult<Option<VocabularyItemView>> {
        let view = sqlx::query_as!(
            VocabularyItemView,
            r#"
            SELECT 
                item_id, entry_id, spelling, disambiguation,
                pronunciation, phonetic_respelling, audio_url,
                register, cefr_level,
                definitions as "definitions!: _",
                synonyms as "synonyms: _",
                antonyms as "antonyms: _",
                collocations as "collocations: _",
                definition_count, example_count, quality_score,
                status, created_by_type, created_by_id,
                created_at, last_modified_at, last_modified_by, version
            FROM vocabulary_items_view
            WHERE item_id = $1
            "#,
            item_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(view)
    }

    async fn update_item_view(&self, view: &VocabularyItemView) -> DomainResult<()> {
        let affected = sqlx::query!(
            r#"
            UPDATE vocabulary_items_view
            SET 
                spelling = $2,
                disambiguation = $3,
                pronunciation = $4,
                phonetic_respelling = $5,
                audio_url = $6,
                register = $7,
                cefr_level = $8,
                definitions = $9,
                synonyms = $10,
                antonyms = $11,
                collocations = $12,
                definition_count = $13,
                example_count = $14,
                quality_score = $15,
                status = $16,
                last_modified_at = $17,
                last_modified_by = $18,
                version = $19
            WHERE item_id = $1 AND version = $20
            "#,
            view.item_id,
            view.spelling,
            view.disambiguation,
            view.pronunciation,
            view.phonetic_respelling,
            view.audio_url,
            view.register,
            view.cefr_level,
            view.definitions as _,
            view.synonyms as _,
            view.antonyms as _,
            view.collocations as _,
            view.definition_count,
            view.example_count,
            view.quality_score as _,
            view.status,
            view.last_modified_at,
            view.last_modified_by,
            view.version,
            view.version - 1 // 楽観的ロック
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if affected.rows_affected() == 0 {
            return Err(DomainError::OptimisticLockError);
        }

        Ok(())
    }

    async fn delete_item_view(&self, item_id: Uuid) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM vocabulary_items_view
            WHERE item_id = $1
            "#,
            item_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn get_item_views_by_entry(
        &self,
        entry_id: Uuid,
    ) -> DomainResult<Vec<VocabularyItemView>> {
        let views = sqlx::query_as!(
            VocabularyItemView,
            r#"
            SELECT 
                item_id, entry_id, spelling, disambiguation,
                pronunciation, phonetic_respelling, audio_url,
                register, cefr_level,
                definitions as "definitions!: _",
                synonyms as "synonyms: _",
                antonyms as "antonyms: _",
                collocations as "collocations: _",
                definition_count, example_count, quality_score,
                status, created_by_type, created_by_id,
                created_at, last_modified_at, last_modified_by, version
            FROM vocabulary_items_view
            WHERE entry_id = $1
            ORDER BY created_at
            "#,
            entry_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(views)
    }
}

/// PostgreSQL プロジェクション状態リポジトリ
pub struct PostgresProjectionStateRepository {
    pool: PgPool,
}

impl PostgresProjectionStateRepository {
    /// 新しいリポジトリを作成
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectionStateRepository for PostgresProjectionStateRepository {
    async fn get_state(&self, projection_name: &str) -> DomainResult<Option<ProjectionState>> {
        let state = sqlx::query_as!(
            ProjectionState,
            r#"
            SELECT 
                projection_name,
                last_processed_event_id,
                last_processed_timestamp,
                event_store_position,
                status,
                error_count as "error_count!",
                last_error,
                updated_at
            FROM projection_state
            WHERE projection_name = $1
            "#,
            projection_name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(state)
    }

    async fn save_state(&self, state: &ProjectionState) -> DomainResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO projection_state (
                projection_name,
                last_processed_event_id,
                last_processed_timestamp,
                event_store_position,
                status,
                error_count,
                last_error,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (projection_name) DO UPDATE SET
                last_processed_event_id = EXCLUDED.last_processed_event_id,
                last_processed_timestamp = EXCLUDED.last_processed_timestamp,
                event_store_position = EXCLUDED.event_store_position,
                status = EXCLUDED.status,
                error_count = EXCLUDED.error_count,
                last_error = EXCLUDED.last_error,
                updated_at = EXCLUDED.updated_at
            "#,
            state.projection_name,
            state.last_processed_event_id,
            state.last_processed_timestamp,
            state.event_store_position,
            state.status,
            state.error_count,
            state.last_error,
            state.updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}
