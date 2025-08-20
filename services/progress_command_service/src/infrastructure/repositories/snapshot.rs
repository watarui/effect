//! スナップショットストア実装

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::Progress,
    error::{Error, Result},
    ports::outbound::SnapshotStorePort,
};

/// PostgreSQL スナップショットストア
pub struct PostgresSnapshotStore {
    pool: PgPool,
}

impl PostgresSnapshotStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SnapshotStorePort for PostgresSnapshotStore {
    async fn save_snapshot(&self, aggregate_id: Uuid, snapshot: Progress) -> Result<()> {
        let snapshot_data =
            serde_json::to_value(&snapshot).map_err(|e| Error::Serialization(e.to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO snapshots (aggregate_id, aggregate_type, snapshot_data, version, created_at)
            VALUES ($1, 'Progress', $2, $3, $4)
            ON CONFLICT (aggregate_id, aggregate_type)
            DO UPDATE SET 
                snapshot_data = EXCLUDED.snapshot_data,
                version = EXCLUDED.version,
                created_at = EXCLUDED.created_at
            "#,
            aggregate_id,
            snapshot_data,
            snapshot.version,
            chrono::Utc::now()
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    async fn get_latest_snapshot(&self, aggregate_id: Uuid) -> Result<Option<Progress>> {
        let record = sqlx::query!(
            r#"
            SELECT snapshot_data
            FROM snapshots
            WHERE aggregate_id = $1 AND aggregate_type = 'Progress'
            "#,
            aggregate_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        match record {
            Some(r) => {
                let snapshot: Progress = serde_json::from_value(r.snapshot_data)
                    .map_err(|e| Error::Serialization(e.to_string()))?;
                Ok(Some(snapshot))
            },
            None => Ok(None),
        }
    }
}
