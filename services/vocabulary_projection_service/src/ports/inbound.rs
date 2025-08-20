//! 入力ポート（ユースケースインターフェース）

use async_trait::async_trait;

use crate::error::Result;

/// イベント処理のユースケース
#[async_trait]
pub trait EventProcessorUseCase: Send + Sync {
    /// イベント処理を開始
    async fn start_processing(&self) -> Result<()>;

    /// イベント処理を停止
    async fn stop_processing(&self) -> Result<()>;

    /// 処理状態を取得
    async fn get_status(&self) -> Result<ProcessorStatus>;
}

/// プロセッサーの状態
#[derive(Debug, Clone)]
pub struct ProcessorStatus {
    pub is_running:              bool,
    pub last_processed_position: i64,
    pub events_processed_total:  u64,
    pub error_count:             u32,
}
