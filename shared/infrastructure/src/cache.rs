// Cache - キャッシュ実装
// TODO: Redis クライアント実装予定

use thiserror::Error;

/// キャッシュクライアント
pub struct Client;

/// キャッシュエラー
#[derive(Debug, Error)]
pub enum Error {
    /// 内部エラー
    #[error("Internal error: {0}")]
    Internal(String),
}
