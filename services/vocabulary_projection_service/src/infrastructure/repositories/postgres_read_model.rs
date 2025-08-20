//! PostgreSQL Read Model リポジトリ実装

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::projections::{
        VocabularyEntryProjection,
        VocabularyExampleProjection,
        VocabularyItemProjection,
    },
    error::Result,
    ports::outbound::{ItemEnrichmentData, ReadModelRepository},
};

/// PostgreSQL Read Model リポジトリ
#[derive(Clone)]
pub struct PostgresReadModelRepository {
    pool: PgPool,
}

impl PostgresReadModelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReadModelRepository for PostgresReadModelRepository {
    async fn save_entry(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        entry: &VocabularyEntryProjection,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO vocabulary_entries_read (
                entry_id, spelling, primary_item_id, item_count,
                created_at, updated_at, last_event_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (entry_id) DO UPDATE SET
                spelling = EXCLUDED.spelling,
                primary_item_id = EXCLUDED.primary_item_id,
                item_count = EXCLUDED.item_count,
                updated_at = EXCLUDED.updated_at,
                last_event_version = EXCLUDED.last_event_version
            WHERE vocabulary_entries_read.last_event_version < EXCLUDED.last_event_version
            "#,
            entry.entry_id,
            entry.spelling,
            entry.primary_item_id,
            entry.item_count,
            entry.created_at,
            entry.updated_at,
            entry.last_event_version
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn save_item(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item: &VocabularyItemProjection,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO vocabulary_items_read (
                item_id, entry_id, spelling, disambiguation,
                part_of_speech, definition, ipa_pronunciation,
                cefr_level, frequency_rank, is_published, is_deleted,
                example_count, created_at, updated_at, last_event_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (item_id) DO UPDATE SET
                entry_id = EXCLUDED.entry_id,
                spelling = EXCLUDED.spelling,
                disambiguation = EXCLUDED.disambiguation,
                part_of_speech = EXCLUDED.part_of_speech,
                definition = EXCLUDED.definition,
                ipa_pronunciation = EXCLUDED.ipa_pronunciation,
                cefr_level = EXCLUDED.cefr_level,
                frequency_rank = EXCLUDED.frequency_rank,
                is_published = EXCLUDED.is_published,
                is_deleted = EXCLUDED.is_deleted,
                example_count = EXCLUDED.example_count,
                updated_at = EXCLUDED.updated_at,
                last_event_version = EXCLUDED.last_event_version
            WHERE vocabulary_items_read.last_event_version < EXCLUDED.last_event_version
            "#,
            item.item_id,
            item.entry_id,
            item.spelling,
            item.disambiguation,
            item.part_of_speech,
            item.definition,
            item.ipa_pronunciation,
            item.cefr_level,
            item.frequency_rank,
            item.is_published,
            item.is_deleted,
            item.example_count,
            item.created_at,
            item.updated_at,
            item.last_event_version
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn add_example(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        example: &VocabularyExampleProjection,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO vocabulary_examples_read (
                example_id, item_id, example, translation, added_by, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            example.example_id,
            example.item_id,
            example.example,
            example.translation,
            example.added_by,
            example.created_at
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn update_item_published(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item_id: Uuid,
        is_published: bool,
        version: i64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE vocabulary_items_read
            SET is_published = $2,
                updated_at = NOW(),
                last_event_version = $3
            WHERE item_id = $1 AND last_event_version < $3
            "#,
            item_id,
            is_published,
            version
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn update_item_deleted(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item_id: Uuid,
        is_deleted: bool,
        version: i64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE vocabulary_items_read
            SET is_deleted = $2,
                updated_at = NOW(),
                last_event_version = $3
            WHERE item_id = $1 AND last_event_version < $3
            "#,
            item_id,
            is_deleted,
            version
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn update_item_enrichment(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item_id: Uuid,
        enrichment: ItemEnrichmentData,
        version: i64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE vocabulary_items_read
            SET part_of_speech = COALESCE($2, part_of_speech),
                definition = COALESCE($3, definition),
                ipa_pronunciation = COALESCE($4, ipa_pronunciation),
                cefr_level = COALESCE($5, cefr_level),
                frequency_rank = COALESCE($6, frequency_rank),
                updated_at = NOW(),
                last_event_version = $7
            WHERE item_id = $1 AND last_event_version < $7
            "#,
            item_id,
            enrichment.part_of_speech,
            enrichment.definition,
            enrichment.ipa_pronunciation,
            enrichment.cefr_level,
            enrichment.frequency_rank,
            version
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn update_entry_primary_item(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        entry_id: Uuid,
        primary_item_id: Option<Uuid>,
        version: i64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE vocabulary_entries_read
            SET primary_item_id = $2,
                updated_at = NOW(),
                last_event_version = $3
            WHERE entry_id = $1 AND last_event_version < $3
            "#,
            entry_id,
            primary_item_id,
            version
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn update_item_count(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        entry_id: Uuid,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE vocabulary_entries_read
            SET item_count = (
                SELECT COUNT(*)
                FROM vocabulary_items_read
                WHERE entry_id = $1 AND NOT is_deleted
            ),
            updated_at = NOW()
            WHERE entry_id = $1
            "#,
            entry_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn increment_example_count(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item_id: Uuid,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE vocabulary_items_read
            SET example_count = example_count + 1,
                updated_at = NOW()
            WHERE item_id = $1
            "#,
            item_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>> {
        Ok(self.pool.begin().await?)
    }
}
