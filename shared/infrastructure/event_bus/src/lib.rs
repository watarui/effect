//! イベントバス実装
//!
//! このモジュールは [`EventBus`] トレイトの異なるメッセージングシステム向けの
//! 実装を提供します。

//! Event Bus 共通インターフェース
//!
//! マイクロサービス間のイベント通信を抽象化

use std::error::Error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod pubsub;

/// Event Bus のエラー型
#[derive(Debug, Error)]
pub enum EventBusError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Publish error: {0}")]
    Publish(String),

    #[error("Subscribe error: {0}")]
    Subscribe(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// イベントの基本トレイト
pub trait Event: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// イベントタイプを取得
    fn event_type(&self) -> &str;

    /// 集約IDを取得
    fn aggregate_id(&self) -> &str;
}

/// Event Bus の共通インターフェース
#[async_trait]
pub trait EventBus: Send + Sync {
    /// イベントを発行
    async fn publish<E: Event>(&self, topic: &str, event: &E) -> Result<(), EventBusError>;

    /// イベントを購読
    async fn subscribe<E, F>(
        &self,
        topic: &str,
        subscription: &str,
        handler: F,
    ) -> Result<(), EventBusError>
    where
        E: Event,
        F: Fn(E) -> Result<(), Box<dyn Error>> + Send + Sync + 'static;

    /// 購読を停止
    async fn unsubscribe(&self, subscription: &str) -> Result<(), EventBusError>;
}

// Re-export
pub use pubsub::PubSubEventBus;
