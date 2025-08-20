//! インバウンドポート

use async_trait::async_trait;

use crate::error::Result;

/// イベントプロセッサーポート
#[async_trait]
pub trait EventProcessor: Send + Sync {
    /// イベントを処理
    async fn process_events(&self) -> Result<()>;

    /// 特定のプロジェクションを再構築
    async fn rebuild_projection(&self, projection_name: &str) -> Result<()>;
}

/// プロジェクション管理ポート
#[async_trait]
pub trait ProjectionManager: Send + Sync {
    /// すべてのプロジェクションの状態を取得
    async fn get_projection_states(&self) -> Result<Vec<crate::domain::ProjectionState>>;

    /// プロジェクションを一時停止
    async fn pause_projection(&self, projection_name: &str) -> Result<()>;

    /// プロジェクションを再開
    async fn resume_projection(&self, projection_name: &str) -> Result<()>;
}
