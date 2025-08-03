//! アプリケーション層
//!
//! コマンドハンドラーの実装

/// コマンドハンドラー（仮実装）
pub struct CommandHandler;

impl CommandHandler {
    /// 新しいコマンドハンドラーを作成
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}
