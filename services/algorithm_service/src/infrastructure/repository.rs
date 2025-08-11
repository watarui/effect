//! Repository implementations for Algorithm Service

pub mod learning_item;
pub mod mock;
pub mod review_history;
pub mod statistics;
pub mod strategy;

use thiserror::Error;

/// Repository error types
#[derive(Debug, Error)]
#[allow(clippy::module_name_repetitions)]
pub enum RepositoryError {
    /// データベースエラー
    #[error("Database error: {0}")]
    Database(String),

    /// エンティティが見つからない
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// 競合エラー
    #[error("Conflict: {0}")]
    Conflict(String),

    /// 無効なデータ
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound("Entity not found".to_string()),
            _ => Self::Database(err.to_string()),
        }
    }
}

/// Result type for repository operations
#[allow(clippy::module_name_repetitions)]
pub type RepositoryResult<T> = Result<T, RepositoryError>;
