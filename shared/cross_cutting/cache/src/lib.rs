//! Cache - キャッシュ実装
//!
//! キャッシュ機能を提供するクレート。
//! 現在は基本的な構造のみ定義しており、Redis
//! クライアントの実装を予定しています。

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
