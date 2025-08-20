//! プロジェクション管理

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    domain::ProjectionState,
    error::Result,
    ports::{inbound::ProjectionManager, outbound::ProjectionStateStore},
};

/// プロジェクション管理実装
pub struct ProjectionManagerImpl {
    state_store: Arc<dyn ProjectionStateStore>,
}

impl ProjectionManagerImpl {
    pub fn new(state_store: Arc<dyn ProjectionStateStore>) -> Self {
        Self { state_store }
    }
}

#[async_trait]
impl ProjectionManager for ProjectionManagerImpl {
    async fn get_projection_states(&self) -> Result<Vec<ProjectionState>> {
        self.state_store.get_all_states().await
    }

    async fn pause_projection(&self, _projection_name: &str) -> Result<()> {
        // TODO: プロジェクションの一時停止ロジックを実装
        Ok(())
    }

    async fn resume_projection(&self, _projection_name: &str) -> Result<()> {
        // TODO: プロジェクションの再開ロジックを実装
        Ok(())
    }
}
