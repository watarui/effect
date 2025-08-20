//! インバウンドポート

use async_trait::async_trait;
use uuid::Uuid;

use crate::{domain::commands::ProgressCommand, error::Result};

/// コマンドハンドラーポート
#[async_trait]
pub trait CommandHandler: Send + Sync {
    async fn handle(&self, command: ProgressCommand) -> Result<()>;
}

/// 進捗照会ポート
#[async_trait]
pub trait ProgressQueryPort: Send + Sync {
    async fn get_user_progress(&self, user_id: Uuid) -> Result<crate::domain::Progress>;
}
