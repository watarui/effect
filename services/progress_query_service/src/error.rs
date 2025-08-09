use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ServiceError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

impl From<std::num::ParseIntError> for ServiceError {
    fn from(err: std::num::ParseIntError) -> Self {
        ServiceError::Parse(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ServiceError>;
