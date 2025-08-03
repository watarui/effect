//! イベント関連エラー

use thiserror::Error;

/// イベントハンドリングに関連するエラー
#[derive(Debug, Error)]
pub enum EventError {
    /// イベント発行時のエラー
    #[error("Failed to publish event: {0}")]
    Publish(String),

    /// イベントストア保存時のエラー
    #[error("Failed to store event: {0}")]
    Store(String),

    /// イベントデシリアライズ時のエラー
    #[error("Failed to deserialize event: {0}")]
    Deserialization(String),

    /// イベントハンドラー実行時のエラー
    #[error("Handler error: {0}")]
    Handler(String),
}
