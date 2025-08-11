use thiserror::Error;

/// Vocabulary Command Service のエラー型
#[derive(Error, Debug)]
pub enum Error {
    /// 設定エラー
    #[error("Configuration error: {0}")]
    Config(String),

    /// データベースエラー
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// データベースエラー（文字列）
    #[error("Database error: {0}")]
    DatabaseString(String),

    /// ドメインエラー
    #[error("Domain error: {0}")]
    Domain(String),

    /// 検証エラー
    #[error("Validation error: {0}")]
    Validation(String),

    /// 競合エラー（楽観的ロック）
    #[error("Conflict error: {0}")]
    Conflict(String),

    /// リソースが見つからない
    #[error("Not found: {0}")]
    NotFound(String),

    /// Event Store エラー
    #[error("Event store error: {0}")]
    EventStore(String),

    /// gRPC エラー
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    /// シリアライゼーションエラー
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// 内部エラー
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result 型のエイリアス
pub type Result<T> = std::result::Result<T, Error>;

/// エラーを gRPC ステータスに変換
impl From<Error> for tonic::Status {
    fn from(err: Error) -> Self {
        match err {
            Error::Validation(msg) => tonic::Status::invalid_argument(msg),
            Error::NotFound(msg) => tonic::Status::not_found(msg),
            Error::Conflict(msg) => tonic::Status::aborted(msg),
            Error::Domain(msg) => tonic::Status::failed_precondition(msg),
            _ => tonic::Status::internal(err.to_string()),
        }
    }
}
