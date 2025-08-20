//! Vocabulary Projection Service のエラー型

use thiserror::Error;

/// プロジェクションサービスのエラー
#[derive(Debug, Error)]
pub enum ProjectionError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Event Store connection error: {0}")]
    EventStore(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Projection state error: {0}")]
    ProjectionState(String),

    #[error("Event processing error: {0}")]
    EventProcessing(String),

    #[error("Checkpoint error: {0}")]
    Checkpoint(String),

    #[error("Network error: {0}")]
    Network(String),
}

pub type Result<T> = std::result::Result<T, ProjectionError>;
