//! イベントハンドリングトレイト
//!
//! このモジュールはイベントハンドリング、保存、配信のためのトレイトを含みます。

use async_trait::async_trait;

use crate::{DomainEvent, EventError};

/// ドメインイベントをハンドリングするためのトレイト
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// イベントをハンドルする
    async fn handle(&self, event: DomainEvent) -> Result<(), EventError>;
}

/// イベントの発行と購読のためのイベントバストレイト
#[async_trait]
pub trait EventBus: Send + Sync {
    /// イベントを発行する
    async fn publish(&self, event: DomainEvent) -> Result<(), EventError>;
    /// イベントハンドラーを購読する
    async fn subscribe(&self, handler: Box<dyn EventHandler>) -> Result<(), EventError>;
}

/// イベントを永続化するためのイベントストアトレイト
#[async_trait]
pub trait EventStore: Send + Sync {
    /// イベントをストリームに追加する
    async fn append(&self, stream_id: &str, events: Vec<DomainEvent>) -> Result<(), EventError>;
    /// ストリームからイベントを読み込む
    async fn read(&self, stream_id: &str, from: usize) -> Result<Vec<DomainEvent>, EventError>;
}
