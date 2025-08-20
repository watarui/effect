//! イベントストア実装

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    domain::events::ProgressEvent,
    error::{Error, Result},
    ports::outbound::EventStorePort,
};

/// PostgreSQL イベントストア
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStorePort for PostgresEventStore {
    async fn save_event(&self, stream_id: String, event: ProgressEvent) -> Result<i64> {
        let event_data =
            serde_json::to_value(&event).map_err(|e| Error::Serialization(e.to_string()))?;

        let event_type = match &event {
            ProgressEvent::LearningStarted { .. } => "LearningStarted",
            ProgressEvent::ItemCompleted { .. } => "ItemCompleted",
            ProgressEvent::SessionCompleted { .. } => "SessionCompleted",
            ProgressEvent::StreakUpdated { .. } => "StreakUpdated",
            ProgressEvent::AchievementUnlocked { .. } => "AchievementUnlocked",
            ProgressEvent::DailyGoalCompleted { .. } => "DailyGoalCompleted",
        };

        let result = sqlx::query!(
            r#"
            INSERT INTO events (event_id, stream_id, event_type, event_data, event_version, occurred_at)
            VALUES ($1, $2, $3, $4, 
                (SELECT COALESCE(MAX(event_version), 0) + 1 FROM events WHERE stream_id = $5),
                $6)
            RETURNING event_version
            "#,
            uuid::Uuid::new_v4(),
            stream_id.clone(),
            event_type,
            event_data,
            stream_id,
            chrono::Utc::now()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(result.event_version)
    }

    async fn get_events(&self, stream_id: String) -> Result<Vec<ProgressEvent>> {
        let records = sqlx::query!(
            r#"
            SELECT event_data
            FROM events
            WHERE stream_id = $1
            ORDER BY event_version
            "#,
            stream_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;

        let mut events = Vec::new();
        for record in records {
            let event: ProgressEvent = serde_json::from_value(record.event_data)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_events_from(
        &self,
        stream_id: String,
        from_version: i64,
    ) -> Result<Vec<ProgressEvent>> {
        let records = sqlx::query!(
            r#"
            SELECT event_data
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

        let mut events = Vec::new();
        for record in records {
            let event: ProgressEvent = serde_json::from_value(record.event_data)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            events.push(event);
        }

        Ok(events)
    }
}
