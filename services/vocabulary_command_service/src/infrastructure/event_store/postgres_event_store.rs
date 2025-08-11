use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::DomainEvent,
    error::{Error, Result},
    ports::event_store::{AggregateSnapshot, EventStore},
};

/// PostgreSQL 実装の EventStore
#[derive(Clone)]
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn append_event(&self, event: DomainEvent) -> Result<()> {
        // イベントをJSONにシリアライズ
        let event_data =
            serde_json::to_value(&event).map_err(|e| Error::Serialization(e.to_string()))?;

        // イベントのメタデータを抽出
        let metadata = event.metadata();
        let (aggregate_id, version, event_type) =
            (metadata.aggregate_id, metadata.version, event.event_type());

        // イベントをデータベースに保存
        sqlx::query(
            r#"
            INSERT INTO domain_events (
                event_id,
                aggregate_id,
                aggregate_type,
                event_type,
                event_version,
                event_data,
                created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(aggregate_id)
        .bind("VocabularyItem") // 集約タイプ
        .bind(event_type)
        .bind(version as i32)
        .bind(event_data)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        Ok(())
    }

    async fn get_events_by_aggregate_id(&self, aggregate_id: Uuid) -> Result<Vec<DomainEvent>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                event_type,
                event_data,
                event_version,
                created_at
            FROM domain_events
            WHERE aggregate_id = $1
            ORDER BY event_version ASC
            "#,
        )
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let event_type: String = row.get("event_type");
            let event_data: serde_json::Value = row.get("event_data");

            // イベントタイプに応じてデシリアライズ
            let event = match event_type.as_str() {
                "VocabularyItemCreated" => serde_json::from_value::<DomainEvent>(event_data)
                    .map_err(|e| Error::Serialization(e.to_string()))?,
                "VocabularyItemDisambiguationUpdated" => {
                    serde_json::from_value::<DomainEvent>(event_data)
                        .map_err(|e| Error::Serialization(e.to_string()))?
                },
                "VocabularyItemPublished" => serde_json::from_value::<DomainEvent>(event_data)
                    .map_err(|e| Error::Serialization(e.to_string()))?,
                _ => {
                    return Err(Error::DatabaseString(format!(
                        "Unknown event type: {}",
                        event_type
                    )));
                },
            };

            events.push(event);
        }

        Ok(events)
    }

    async fn get_events_since_version(
        &self,
        aggregate_id: Uuid,
        version: i64,
    ) -> Result<Vec<DomainEvent>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                event_type,
                event_data,
                event_version,
                created_at
            FROM domain_events
            WHERE aggregate_id = $1 AND event_version >= $2
            ORDER BY event_version ASC
            "#,
        )
        .bind(aggregate_id)
        .bind(version)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let event_type: String = row.get("event_type");
            let event_data: serde_json::Value = row.get("event_data");

            // イベントタイプに応じてデシリアライズ
            let event = match event_type.as_str() {
                "VocabularyItemCreated" => serde_json::from_value::<DomainEvent>(event_data)
                    .map_err(|e| Error::Serialization(e.to_string()))?,
                "VocabularyItemDisambiguationUpdated" => {
                    serde_json::from_value::<DomainEvent>(event_data)
                        .map_err(|e| Error::Serialization(e.to_string()))?
                },
                "VocabularyItemPublished" => serde_json::from_value::<DomainEvent>(event_data)
                    .map_err(|e| Error::Serialization(e.to_string()))?,
                _ => {
                    return Err(Error::DatabaseString(format!(
                        "Unknown event type: {}",
                        event_type
                    )));
                },
            };

            events.push(event);
        }

        Ok(events)
    }

    async fn get_events_by_type(
        &self,
        event_type: &str,
        limit: Option<usize>,
    ) -> Result<Vec<DomainEvent>> {
        let query = if let Some(limit) = limit {
            format!(
                r#"
                SELECT 
                    event_type,
                    event_data,
                    event_version,
                    created_at
                FROM domain_events
                WHERE event_type = $1
                ORDER BY created_at DESC
                LIMIT {}
                "#,
                limit
            )
        } else {
            r#"
            SELECT 
                event_type,
                event_data,
                event_version,
                created_at
            FROM domain_events
            WHERE event_type = $1
            ORDER BY created_at DESC
            "#
            .to_string()
        };

        let rows = sqlx::query(&query)
            .bind(event_type)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::DatabaseString(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let event_data: serde_json::Value = row.get("event_data");
            let event = serde_json::from_value::<DomainEvent>(event_data)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<DomainEvent>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                event_type,
                event_data,
                event_version,
                created_at
            FROM domain_events
            WHERE created_at >= $1 AND created_at <= $2
            ORDER BY created_at ASC
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let event_data: serde_json::Value = row.get("event_data");
            let event = serde_json::from_value::<DomainEvent>(event_data)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_latest_snapshot(&self, aggregate_id: Uuid) -> Result<Option<AggregateSnapshot>> {
        let row = sqlx::query(
            r#"
            SELECT 
                aggregate_id,
                aggregate_type,
                data,
                version,
                created_at
            FROM snapshots
            WHERE aggregate_id = $1
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .bind(aggregate_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        match row {
            Some(row) => {
                let snapshot = AggregateSnapshot {
                    aggregate_id:   row.get::<Uuid, _>("aggregate_id"),
                    aggregate_type: row.get("aggregate_type"),
                    data:           row.get("data"),
                    version:        row.get("version"),
                    created_at:     row.get("created_at"),
                };
                Ok(Some(snapshot))
            },
            None => Ok(None),
        }
    }

    async fn save_snapshot(&self, snapshot: AggregateSnapshot) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO snapshots (
                aggregate_id,
                aggregate_type,
                data,
                version,
                created_at
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(snapshot.aggregate_id)
        .bind(snapshot.aggregate_type)
        .bind(snapshot.data)
        .bind(snapshot.version)
        .bind(snapshot.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseString(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::postgres::PgPoolOptions;

    use super::*;
    use crate::domain::{
        Disambiguation,
        EntryId,
        EventMetadata,
        Spelling,
        VocabularyItem,
        VocabularyItemCreated,
    };

    #[tokio::test]
    #[ignore] // 統合テストは明示的に実行
    async fn test_event_store_operations() {
        // テスト用のデータベース接続
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://effect:effect_password@localhost:5432/effect_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let event_store = PostgresEventStore::new(pool.clone());

        // テスト用の集約を作成
        let entry_id = EntryId::new();
        let spelling = Spelling::new("test".to_string()).unwrap();
        let disambiguation = Disambiguation::new(Some("test meaning".to_string())).unwrap();
        let item = VocabularyItem::create(entry_id, spelling, disambiguation);

        // イベントを作成
        let event = DomainEvent::VocabularyItemCreated(VocabularyItemCreated {
            metadata:       EventMetadata::new(*item.item_id.as_uuid(), 1),
            item_id:        *item.item_id.as_uuid(),
            entry_id:       *entry_id.as_uuid(),
            spelling:       "test".to_string(),
            disambiguation: Some("test meaning".to_string()),
        });

        // イベントを保存
        event_store
            .append_event(event.clone())
            .await
            .expect("Failed to append event");

        // イベントを取得
        let events = event_store
            .get_events_by_aggregate_id(*item.item_id.as_uuid())
            .await
            .expect("Failed to get events");
        assert_eq!(events.len(), 1);

        // バージョン指定でイベントを取得
        let events = event_store
            .get_events_since_version(*item.item_id.as_uuid(), 1)
            .await
            .expect("Failed to get events since version");
        assert_eq!(events.len(), 1);

        // クリーンアップ
        sqlx::query("DELETE FROM domain_events WHERE aggregate_id = $1")
            .bind(item.item_id.as_uuid())
            .execute(&pool)
            .await
            .expect("Failed to clean up");
    }
}
