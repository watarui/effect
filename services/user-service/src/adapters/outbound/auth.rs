//! 認証アダプター

use thiserror::Error;

pub mod mock;

/// 認証エラー
#[derive(Error, Debug)]
pub enum Error {
    /// 無効なトークン
    #[error("Invalid token")]
    InvalidToken,
    /// トークン生成エラー
    #[error("Failed to generate token")]
    TokenGeneration,
    /// 内部エラー
    #[error("Internal auth error: {0}")]
    Internal(String),
}
