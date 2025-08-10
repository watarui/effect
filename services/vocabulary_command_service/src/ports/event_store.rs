use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{domain::DomainEvent, error::Result};

/// イベントストアのトレイト
#[async_trait]
pub trait EventStore: Send + Sync {
    /// イベントを追加
    async fn append_event(&self, event: DomainEvent) -> Result<()>;

    /// 集約ID でイベントを取得
    async fn get_events_by_aggregate_id(&self, aggregate_id: Uuid) -> Result<Vec<DomainEvent>>;

    /// 特定バージョン以降のイベントを取得
    async fn get_events_since_version(
        &self,
        aggregate_id: Uuid,
        version: i64,
    ) -> Result<Vec<DomainEvent>>;

    /// イベントタイプでフィルタリング
    async fn get_events_by_type(
        &self,
        event_type: &str,
        limit: Option<usize>,
    ) -> Result<Vec<DomainEvent>>;

    /// 時間範囲でイベントを取得
    async fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<DomainEvent>>;

    /// 最新のスナップショットを取得
    async fn get_latest_snapshot(&self, aggregate_id: Uuid) -> Result<Option<AggregateSnapshot>>;

    /// スナップショットを保存
    async fn save_snapshot(&self, snapshot: AggregateSnapshot) -> Result<()>;
}

/// 集約のスナップショット
#[derive(Debug, Clone)]
pub struct AggregateSnapshot {
    pub aggregate_id:   Uuid,
    pub aggregate_type: String,
    pub data:           Vec<u8>,
    pub version:        i64,
    pub created_at:     DateTime<Utc>,
}

/// イベントバスのトレイト（イベント発行用）
#[async_trait]
pub trait EventBus: Send + Sync {
    /// イベントを発行
    async fn publish(&self, event: DomainEvent) -> Result<()>;

    /// 複数のイベントをバッチで発行
    async fn publish_batch(&self, events: Vec<DomainEvent>) -> Result<()>;
}
