//! 共通エラー型
//!
//! このモジュールは全ての境界づけられたコンテキストで共有されるエラー型を含みます。

use thiserror::Error;

/// ドメイン操作用の共通 Result 型
pub type DomainResult<T> = Result<T, DomainError>;

/// 共通ドメインエラー
#[derive(Debug, Error)]
pub enum DomainError {
    /// エンティティが見つからない場合のエラー
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// 無効な状態の場合のエラー
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// バリデーションエラー
    #[error("Validation error: {0}")]
    Validation(String),

    /// 認証・認可エラー
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// 内部エラー
    #[error("Internal error: {0}")]
    Internal(String),
}
