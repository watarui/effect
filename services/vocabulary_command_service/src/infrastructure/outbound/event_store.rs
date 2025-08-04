//! Event Store 実装

use std::sync::Arc;

use async_trait::async_trait;
use shared_error::{DomainError, DomainResult};
use shared_event_store::{
    EventStore as SharedEventStore,
    EventStoreError,
    postgres::PostgresEventStore as SharedPostgresEventStore,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{aggregates::VocabularyEntry, events::VocabularyDomainEvent},
    ports::outbound::EventStore,
};

/// PostgreSQL ベースの Event Store 実装
pub struct PostgresEventStore {
    inner: Arc<SharedPostgresEventStore>,
}

impl PostgresEventStore {
    /// 新しい Event Store を作成
    pub fn new(pool: PgPool) -> Self {
        Self {
            inner: Arc::new(SharedPostgresEventStore::new(pool)),
        }
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn save_aggregate(
        &self,
        aggregate_id: Uuid,
        events: Vec<VocabularyDomainEvent>,
        expected_version: Option<u32>,
    ) -> DomainResult<()> {
        // イベントを JSON に変換
        let event_jsons: Vec<serde_json::Value> = events
            .into_iter()
            .map(|event| {
                let mut value = serde_json::to_value(&event)?;
                // event_type を追加
                if let Some(obj) = value.as_object_mut() {
                    obj.insert(
                        "event_type".to_string(),
                        serde_json::Value::String(event.event_type()),
                    );
                    obj.insert(
                        "occurred_at".to_string(),
                        serde_json::Value::String(event.occurred_at().to_rfc3339()),
                    );
                }
                Ok(value)
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Event Store に保存
        self.inner
            .save_events(
                aggregate_id,
                "VocabularyEntry",
                event_jsons,
                expected_version,
            )
            .await
            .map_err(|e| match e {
                EventStoreError::VersionConflict {
                    expected: _,
                    actual: _,
                } => DomainError::OptimisticLockError,
                _ => DomainError::Internal(e.to_string()),
            })?;

        Ok(())
    }

    async fn load_aggregate(&self, aggregate_id: Uuid) -> DomainResult<Option<VocabularyEntry>> {
        // イベントを読み込み
        let stored_events = self
            .inner
            .load_events(aggregate_id, "VocabularyEntry", None)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if stored_events.is_empty() {
            return Ok(None);
        }

        // イベントをドメインイベントに変換
        let domain_events: Vec<VocabularyDomainEvent> = stored_events
            .into_iter()
            .map(|stored| {
                serde_json::from_value(stored.event_data.clone())
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize event: {e}")))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // 集約を再構築
        let entry = VocabularyEntry::new_from_events(aggregate_id, domain_events)?;

        Ok(Some(entry))
    }

    async fn get_events(
        &self,
        aggregate_id: Uuid,
        from_version: Option<u32>,
    ) -> DomainResult<Vec<VocabularyDomainEvent>> {
        let stored_events = self
            .inner
            .load_events(aggregate_id, "VocabularyEntry", from_version)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        stored_events
            .into_iter()
            .map(|stored| {
                serde_json::from_value(stored.event_data)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize event: {e}")))
            })
            .collect()
    }
}
