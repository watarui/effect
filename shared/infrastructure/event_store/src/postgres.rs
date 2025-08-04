//! PostgreSQL Event Store 実装

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{EventStore, EventStoreError, Snapshot, StoredEvent};

/// PostgreSQL ベースの Event Store 実装
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    /// 新しい Event Store を作成
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    #[instrument(skip(self, events))]
    async fn save_events(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        events: Vec<serde_json::Value>,
        expected_version: Option<u32>,
    ) -> Result<(), EventStoreError> {
        let mut tx = self.pool.begin().await?;

        // ストリームの存在確認または作成
        let stream_id = sqlx::query(
            r#"
            INSERT INTO event_streams (aggregate_id, aggregate_type)
            VALUES ($1, $2)
            ON CONFLICT (aggregate_id, aggregate_type) 
            DO UPDATE SET aggregate_id = EXCLUDED.aggregate_id
            RETURNING stream_id
            "#,
        )
        .bind(aggregate_id)
        .bind(aggregate_type)
        .fetch_one(&mut *tx)
        .await?
        .get::<Uuid, _>("stream_id");

        // 現在のバージョンを取得
        let current_version = sqlx::query(
            r#"
            SELECT COALESCE(MAX(event_version), 0) as version
            FROM events
            WHERE stream_id = $1
            "#,
        )
        .bind(stream_id)
        .fetch_one(&mut *tx)
        .await?
        .get::<i32, _>("version") as u32;

        // 楽観的ロックのチェック
        if let Some(expected) = expected_version
            && current_version != expected
        {
            return Err(EventStoreError::VersionConflict {
                expected,
                actual: current_version,
            });
        }

        // イベントを保存
        let mut next_version = current_version + 1;
        let events_count = events.len();
        for event_data in events {
            let event_type = event_data
                .get("event_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| EventStoreError::Internal("Missing event_type".to_string()))?;

            let occurred_at = event_data
                .get("occurred_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            sqlx::query(
                r#"
                INSERT INTO events (
                    stream_id, aggregate_id, aggregate_type, 
                    event_type, event_version, event_data, occurred_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(stream_id)
            .bind(aggregate_id)
            .bind(aggregate_type)
            .bind(event_type)
            .bind(next_version as i32)
            .bind(&event_data)
            .bind(occurred_at)
            .execute(&mut *tx)
            .await?;

            next_version += 1;
        }

        tx.commit().await?;
        info!(
            aggregate_id = %aggregate_id,
            aggregate_type = %aggregate_type,
            events_count = events_count,
            "Events saved successfully"
        );

        Ok(())
    }

    #[instrument(skip(self))]
    async fn load_events(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        from_version: Option<u32>,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let from_version = from_version.unwrap_or(0) as i32;

        let rows = sqlx::query(
            r#"
            SELECT 
                event_id, aggregate_id, aggregate_type, event_type,
                event_version, event_data, metadata, occurred_at, created_at
            FROM events
            WHERE aggregate_id = $1 AND aggregate_type = $2 AND event_version > $3
            ORDER BY event_version
            "#,
        )
        .bind(aggregate_id)
        .bind(aggregate_type)
        .bind(from_version)
        .fetch_all(&self.pool)
        .await?;

        let events = rows
            .into_iter()
            .map(|row| StoredEvent {
                event_id:       row.get("event_id"),
                aggregate_id:   row.get("aggregate_id"),
                aggregate_type: row.get("aggregate_type"),
                event_type:     row.get("event_type"),
                event_version:  row.get::<i32, _>("event_version") as u32,
                event_data:     row.get("event_data"),
                metadata:       row.get("metadata"),
                occurred_at:    row.get("occurred_at"),
                created_at:     row.get("created_at"),
            })
            .collect();

        Ok(events)
    }

    #[instrument(skip(self, data))]
    async fn save_snapshot(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        version: u32,
        data: serde_json::Value,
    ) -> Result<(), EventStoreError> {
        sqlx::query(
            r#"
            INSERT INTO snapshots (aggregate_id, aggregate_type, aggregate_version, aggregate_data)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (aggregate_id, aggregate_type, aggregate_version) 
            DO UPDATE SET 
                aggregate_data = EXCLUDED.aggregate_data,
                created_at = NOW()
            "#,
        )
        .bind(aggregate_id)
        .bind(aggregate_type)
        .bind(version as i32)
        .bind(&data)
        .execute(&self.pool)
        .await?;

        info!(
            aggregate_id = %aggregate_id,
            aggregate_type = %aggregate_type,
            version = version,
            "Snapshot saved successfully"
        );

        Ok(())
    }

    #[instrument(skip(self))]
    async fn load_snapshot(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
    ) -> Result<Option<Snapshot>, EventStoreError> {
        let row = sqlx::query(
            r#"
            SELECT aggregate_id, aggregate_type, aggregate_version, aggregate_data, created_at
            FROM snapshots
            WHERE aggregate_id = $1 AND aggregate_type = $2
            ORDER BY aggregate_version DESC
            LIMIT 1
            "#,
        )
        .bind(aggregate_id)
        .bind(aggregate_type)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Snapshot {
            aggregate_id:      row.get("aggregate_id"),
            aggregate_type:    row.get("aggregate_type"),
            aggregate_version: row.get::<i32, _>("aggregate_version") as u32,
            aggregate_data:    row.get("aggregate_data"),
            created_at:        row.get("created_at"),
        }))
    }
}
