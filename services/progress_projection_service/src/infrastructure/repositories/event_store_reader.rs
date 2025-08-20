//! イベントストア読み取り実装

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    error::{Error, Result},
    ports::outbound::{Event, EventStoreReader},
};

/// PostgreSQL イベントストアリーダー
pub struct PostgresEventStoreReader {
    pool: PgPool,
}

impl PostgresEventStoreReader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStoreReader for PostgresEventStoreReader {
    async fn read_events(&self, from_position: i64, batch_size: usize) -> Result<Vec<Event>> {
        // NOTE: position カラムがないため event_version で代用
        let records = sqlx::query!(
            r#"
            SELECT 
                event_id, 
                stream_id, 
                event_type, 
                event_data, 
                event_version,
                occurred_at
            FROM events
            WHERE event_version > $1
            ORDER BY event_version
            LIMIT $2
            "#,
            from_position,
            batch_size as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;

        let events = records
            .into_iter()
            .map(|r| Event {
                event_id:      r.event_id,
                stream_id:     r.stream_id,
                event_type:    r.event_type,
                event_data:    r.event_data,
                event_version: r.event_version,
                position:      r.event_version, // position として event_version を使用
                occurred_at:   r.occurred_at,
            })
            .collect();

        Ok(events)
    }

    async fn read_stream_events(&self, stream_id: &str, from_version: i64) -> Result<Vec<Event>> {
        let records = sqlx::query!(
            r#"
            SELECT 
                event_id, 
                stream_id, 
                event_type, 
                event_data, 
                event_version,
                occurred_at
            FROM events
            WHERE stream_id = $1 AND event_version > $2
            ORDER BY event_version
            "#,
            stream_id,
            from_version
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;

        let events = records
            .into_iter()
            .map(|r| Event {
                event_id:      r.event_id,
                stream_id:     r.stream_id,
                event_type:    r.event_type,
                event_data:    r.event_data,
                event_version: r.event_version,
                position:      r.event_version,
                occurred_at:   r.occurred_at,
            })
            .collect();

        Ok(events)
    }
}
