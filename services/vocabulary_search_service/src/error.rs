//! エラー定義

use thiserror::Error;

pub type Result<T> = std::result::Result<T, SearchError>;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Search engine error: {0}")]
    SearchEngine(String),

    #[error("Index not found: {0}")]
    IndexNotFound(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Index is rebuilding")]
    IndexRebuilding,

    #[error("Internal error: {0}")]
    Internal(String),
}
