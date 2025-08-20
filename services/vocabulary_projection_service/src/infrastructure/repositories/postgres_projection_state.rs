//! PostgreSQL プロジェクション状態リポジトリ実装

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    domain::projections::{ProjectionCheckpoint, ProjectionState},
    error::Result,
    ports::outbound::ProjectionStateRepository,
};

/// PostgreSQL プロジェクション状態リポジトリ
pub struct PostgresProjectionStateRepository {
    pool: PgPool,
}

impl PostgresProjectionStateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectionStateRepository for PostgresProjectionStateRepository {
    async fn get_state(&self, name: &str) -> Result<Option<ProjectionState>> {
        let state = sqlx::query_as!(
            ProjectionState,
            r#"
            SELECT 
                projection_name,
                last_processed_position,
                last_processed_event_id,
                last_processed_at,
                error_count,
                last_error,
                last_error_at
            FROM projection_state
            WHERE projection_name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(state)
    }

    async fn save_state(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        state: &ProjectionState,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO projection_state (
                projection_name, 
                last_processed_position, 
                last_processed_event_id,
                last_processed_at,
                error_count,
                last_error,
                last_error_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (projection_name) DO UPDATE SET
                last_processed_position = EXCLUDED.last_processed_position,
                last_processed_event_id = EXCLUDED.last_processed_event_id,
                last_processed_at = EXCLUDED.last_processed_at,
                error_count = EXCLUDED.error_count,
                last_error = EXCLUDED.last_error,
                last_error_at = EXCLUDED.last_error_at,
                updated_at = NOW()
            "#,
            state.projection_name,
            state.last_processed_position,
            state.last_processed_event_id,
            state.last_processed_at,
            state.error_count,
            state.last_error,
            state.last_error_at
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn record_error(&self, name: &str, error: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE projection_state
            SET error_count = error_count + 1,
                last_error = $2,
                last_error_at = NOW(),
                updated_at = NOW()
            WHERE projection_name = $1
            "#,
            name,
            error
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save_checkpoint(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        checkpoint: &ProjectionCheckpoint,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO projection_checkpoints (
                projection_name, position, event_id, events_processed, created_at
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
            checkpoint.projection_name,
            checkpoint.position,
            checkpoint.event_id,
            checkpoint.events_processed,
            checkpoint.created_at
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
