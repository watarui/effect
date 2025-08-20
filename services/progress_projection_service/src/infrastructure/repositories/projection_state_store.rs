//! プロジェクション状態ストア実装

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    domain::ProjectionState,
    error::{Error, Result},
    ports::outbound::ProjectionStateStore,
};

/// PostgreSQL プロジェクション状態ストア
pub struct PostgresProjectionStateStore {
    pool: PgPool,
}

impl PostgresProjectionStateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectionStateStore for PostgresProjectionStateStore {
    async fn save_state(&self, state: &ProjectionState) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO projection_states (projection_name, last_position, last_event_id, updated_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (projection_name)
            DO UPDATE SET 
                last_position = EXCLUDED.last_position,
                last_event_id = EXCLUDED.last_event_id,
                updated_at = EXCLUDED.updated_at
            "#,
            state.projection_name,
            state.last_position,
            state.last_event_id,
            state.updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    async fn get_state(&self, projection_name: &str) -> Result<Option<ProjectionState>> {
        let record = sqlx::query!(
            r#"
            SELECT projection_name, last_position, last_event_id, updated_at
            FROM projection_states
            WHERE projection_name = $1
            "#,
            projection_name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(record.map(|r| ProjectionState {
            projection_name: r.projection_name,
            last_position:   r.last_position,
            last_event_id:   r.last_event_id,
            updated_at:      r.updated_at,
        }))
    }

    async fn get_all_states(&self) -> Result<Vec<ProjectionState>> {
        let records = sqlx::query!(
            r#"
            SELECT projection_name, last_position, last_event_id, updated_at
            FROM projection_states
            ORDER BY projection_name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(records
            .into_iter()
            .map(|r| ProjectionState {
                projection_name: r.projection_name,
                last_position:   r.last_position,
                last_event_id:   r.last_event_id,
                updated_at:      r.updated_at,
            })
            .collect())
    }
}
