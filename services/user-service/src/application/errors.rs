//! アプリケーション層のエラー定義

use thiserror::Error;

/// アプリケーションエラー
#[derive(Error, Debug)]
pub enum ApplicationError {
    /// ユーザーが見つからない
    #[error("User not found")]
    UserNotFound,

    /// Email が既に使用されている
    #[error("Email already exists")]
    EmailAlreadyExists,

    /// 無効な Email フォーマット
    #[error("Invalid email format")]
    InvalidEmail,

    /// 無効なプロフィール
    #[error("Invalid profile: {0}")]
    InvalidProfile(String),

    /// 認証エラー
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// 権限エラー
    #[error("Permission denied")]
    PermissionDenied,

    /// リポジトリエラー
    #[error("Repository error: {0}")]
    Repository(String),

    /// イベント発行エラー
    #[error("Event publishing error: {0}")]
    EventPublishing(String),

    /// 内部エラー
    #[error("Internal error: {0}")]
    Internal(String),
}

/// リポジトリエラーからアプリケーションエラーへの変換
impl From<crate::adapters::outbound::repository::memory::Error> for ApplicationError {
    fn from(err: crate::adapters::outbound::repository::memory::Error) -> Self {
        use crate::adapters::outbound::repository::memory::Error;

        match err {
            Error::NotFound => Self::UserNotFound,
            Error::EmailAlreadyExists => Self::EmailAlreadyExists,
            Error::Internal(msg) => Self::Repository(msg),
        }
    }
}

/// 認証エラーからアプリケーションエラーへの変換
impl From<crate::adapters::outbound::auth::Error> for ApplicationError {
    fn from(err: crate::adapters::outbound::auth::Error) -> Self {
        use crate::adapters::outbound::auth::Error;

        match err {
            Error::InvalidToken => Self::Authentication("Invalid token".to_string()),
            Error::TokenGeneration => Self::Authentication("Failed to generate token".to_string()),
            Error::Internal(msg) => Self::Authentication(msg),
        }
    }
}

/// イベントパブリッシャーエラーからアプリケーションエラーへの変換
impl From<crate::adapters::outbound::event::memory::Error> for ApplicationError {
    fn from(err: crate::adapters::outbound::event::memory::Error) -> Self {
        use crate::adapters::outbound::event::memory::Error;

        match err {
            Error::Internal(msg) => Self::EventPublishing(msg),
        }
    }
}

/// Email エラーからアプリケーションエラーへの変換
impl From<crate::domain::value_objects::email::Error> for ApplicationError {
    fn from(_: crate::domain::value_objects::email::Error) -> Self {
        Self::InvalidEmail
    }
}

/// プロフィールエラーからアプリケーションエラーへの変換
impl From<crate::domain::value_objects::user_profile::ProfileError> for ApplicationError {
    fn from(err: crate::domain::value_objects::user_profile::ProfileError) -> Self {
        Self::InvalidProfile(err.to_string())
    }
}
