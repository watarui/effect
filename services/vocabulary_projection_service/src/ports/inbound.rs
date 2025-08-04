//! インバウンドポート
//!
//! 外部からのイベントを受け取るインターフェース

use async_trait::async_trait;
use shared_error::DomainResult;

/// イベントハンドラーのインターフェース
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// イベントを処理
    async fn handle_event(&self, event_data: Vec<u8>) -> DomainResult<()>;
}
