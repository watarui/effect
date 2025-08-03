// 共通 Event Store インターフェース

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use futures::Stream;
use std::pin::Pin;

/// イベントメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: String,
    pub causation_id: Option<String>,
    pub user_id: String,
    pub source_service: String,
    pub idempotency_key: Option<String>,
}

/// イベントエンベロープ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub event_id: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_data: T,
    pub event_version: i64,
    pub sequence_number: i64,
    pub occurred_at: DateTime<Utc>,
    pub metadata: EventMetadata,
}

/// ストリーム情報
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub stream_id: String,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 読み取りオプション
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    pub from_version: Option<i64>,
    pub to_version: Option<i64>,
    pub max_count: Option<usize>,
    pub backward: bool,
}

/// スナップショット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot<T> {
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub data: T,
    pub version: i64,
    pub created_at: DateTime<Utc>,
}

/// Event Store のエラー型
#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Stream not found: {0}")]
    StreamNotFound(String),
    
    #[error("Concurrency conflict: expected version {expected}, actual {actual}")]
    ConcurrencyConflict { expected: i64, actual: i64 },
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Event Store の共通インターフェース
#[async_trait]
pub trait EventStore: Send + Sync {
    /// イベントをストリームに追加
    async fn append_to_stream<T>(
        &self,
        stream_id: &str,
        expected_version: Option<i64>,
        events: Vec<EventEnvelope<T>>,
    ) -> Result<i64, EventStoreError>
    where
        T: Serialize + Send + Sync;
    
    /// ストリームからイベントを読み取り
    async fn read_stream<T>(
        &self,
        stream_id: &str,
        options: ReadOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<EventEnvelope<T>, EventStoreError>> + Send>>, EventStoreError>
    where
        T: for<'de> Deserialize<'de> + Send;
    
    /// 全イベントを前方から読み取り
    async fn read_all_forward<T>(
        &self,
        from_position: i64,
        max_count: Option<usize>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<EventEnvelope<T>, EventStoreError>> + Send>>, EventStoreError>
    where
        T: for<'de> Deserialize<'de> + Send;
    
    /// ストリーム情報を取得
    async fn get_stream_info(&self, stream_id: &str) -> Result<Option<StreamInfo>, EventStoreError>;
    
    /// ストリームを削除（ソフトデリート）
    async fn delete_stream(&self, stream_id: &str, expected_version: Option<i64>) -> Result<(), EventStoreError>;
    
    /// スナップショットを保存
    async fn save_snapshot<T>(
        &self,
        snapshot: Snapshot<T>,
    ) -> Result<(), EventStoreError>
    where
        T: Serialize + Send + Sync;
    
    /// スナップショットを取得
    async fn get_snapshot<T>(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
    ) -> Result<Option<Snapshot<T>>, EventStoreError>
    where
        T: for<'de> Deserialize<'de> + Send;
}

/// 楽観的並行性制御
pub trait OptimisticConcurrency {
    fn expected_version(&self) -> Option<i64>;
}

/// イベントストリームのビルダー
pub struct EventStreamBuilder {
    stream_id: String,
    events: Vec<Box<dyn std::any::Any + Send + Sync>>,
    expected_version: Option<i64>,
}

impl EventStreamBuilder {
    pub fn new(stream_id: impl Into<String>) -> Self {
        Self {
            stream_id: stream_id.into(),
            events: Vec::new(),
            expected_version: None,
        }
    }
    
    pub fn append<E: 'static + Send + Sync>(mut self, event: E) -> Self {
        self.events.push(Box::new(event));
        self
    }
    
    pub fn expected_version(mut self, version: i64) -> Self {
        self.expected_version = Some(version);
        self
    }
    
    pub fn build(self) -> (String, Vec<Box<dyn std::any::Any + Send + Sync>>, Option<i64>) {
        (self.stream_id, self.events, self.expected_version)
    }
}

/// Event Store のサブスクリプション
#[async_trait]
pub trait EventSubscription: Send + Sync {
    /// サブスクリプションを開始
    async fn subscribe<T>(
        &self,
        stream_id: Option<&str>, // None の場合は全ストリーム
        from_position: Option<i64>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<EventEnvelope<T>, EventStoreError>> + Send>>, EventStoreError>
    where
        T: for<'de> Deserialize<'de> + Send;
    
    /// サブスクリプションを停止
    async fn unsubscribe(&self) -> Result<(), EventStoreError>;
}

/// イベントのプロジェクション用トレイト
#[async_trait]
pub trait EventProjection: Send + Sync {
    type Event;
    type Error: Error + Send + Sync + 'static;
    
    /// プロジェクション名
    fn name(&self) -> &str;
    
    /// イベントを処理
    async fn handle(&self, event: EventEnvelope<Self::Event>) -> Result<(), Self::Error>;
    
    /// 現在の位置を取得
    async fn get_position(&self) -> Result<i64, Self::Error>;
    
    /// 位置を更新
    async fn update_position(&self, position: i64) -> Result<(), Self::Error>;
}

pub mod postgres;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_stream_builder() {
        let (stream_id, events, version) = EventStreamBuilder::new("test-stream")
            .append("event1")
            .append("event2")
            .expected_version(42)
            .build();
            
        assert_eq!(stream_id, "test-stream");
        assert_eq!(events.len(), 2);
        assert_eq!(version, Some(42));
    }
}