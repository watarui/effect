//! アウトバウンドポート

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{Progress, events::ProgressEvent},
    error::Result,
};

/// イベントストアポート
#[async_trait]
pub trait EventStorePort: Send + Sync {
    async fn save_event(&self, stream_id: String, event: ProgressEvent) -> Result<i64>;
    async fn get_events(&self, stream_id: String) -> Result<Vec<ProgressEvent>>;
    async fn get_events_from(
        &self,
        stream_id: String,
        from_version: i64,
    ) -> Result<Vec<ProgressEvent>>;
}

/// スナップショットストアポート
#[async_trait]
pub trait SnapshotStorePort: Send + Sync {
    async fn save_snapshot(&self, aggregate_id: Uuid, snapshot: Progress) -> Result<()>;
    async fn get_latest_snapshot(&self, aggregate_id: Uuid) -> Result<Option<Progress>>;
}

/// イベントパブリッシャーポート
#[async_trait]
pub trait EventPublisherPort: Send + Sync {
    async fn publish(&self, event: ProgressEvent) -> Result<()>;
}

/// イベントメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id:      Uuid,
    pub stream_id:     String,
    pub event_type:    String,
    pub event_version: i64,
    pub occurred_at:   chrono::DateTime<chrono::Utc>,
}
