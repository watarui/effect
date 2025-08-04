//! Event Store - イベントストア実装
//!
//! CQRS/Event Sourcing パターンのための永続化層

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

pub mod postgres;

/// Event Store のエラー型
#[derive(Error, Debug)]
pub enum EventStoreError {
    #[error("Version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u32, actual: u32 },

    #[error("Aggregate not found: {0}")]
    AggregateNotFound(Uuid),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Event Store trait
#[async_trait]
pub trait EventStore: Send + Sync {
    /// イベントを保存
    async fn save_events(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        events: Vec<serde_json::Value>,
        expected_version: Option<u32>,
    ) -> Result<(), EventStoreError>;

    /// 集約のイベントを読み込み
    async fn load_events(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        from_version: Option<u32>,
    ) -> Result<Vec<StoredEvent>, EventStoreError>;

    /// スナップショットを保存
    async fn save_snapshot(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        version: u32,
        data: serde_json::Value,
    ) -> Result<(), EventStoreError>;

    /// 最新のスナップショットを読み込み
    async fn load_snapshot(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
    ) -> Result<Option<Snapshot>, EventStoreError>;
}

/// 保存されたイベント
#[derive(Debug, Clone)]
pub struct StoredEvent {
    pub event_id:       Uuid,
    pub aggregate_id:   Uuid,
    pub aggregate_type: String,
    pub event_type:     String,
    pub event_version:  u32,
    pub event_data:     serde_json::Value,
    pub metadata:       Option<serde_json::Value>,
    pub occurred_at:    DateTime<Utc>,
    pub created_at:     DateTime<Utc>,
}

/// スナップショット
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub aggregate_id:      Uuid,
    pub aggregate_type:    String,
    pub aggregate_version: u32,
    pub aggregate_data:    serde_json::Value,
    pub created_at:        DateTime<Utc>,
}
