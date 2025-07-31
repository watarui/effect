//! 共通エラー型
//!
//! このモジュールは全ての境界づけられたコンテキストで共有されるエラー型を含みます。

use thiserror::Error;

use crate::ids::{ItemId, SessionId, UserId};

/// ドメイン操作用の共通 Result 型
pub type DomainResult<T> = Result<T, DomainError>;

/// 共通ドメインエラー
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    /// エンティティが見つからない場合のエラー
    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound {
        /// エンティティの種類（例: "User", "Item", "Session"）
        entity_type: &'static str,
        /// エンティティのID
        id:          String,
    },

    /// ユーザーが見つからない
    #[error("User not found: {0}")]
    UserNotFound(UserId),

    /// 項目が見つからない
    #[error("Item not found: {0}")]
    ItemNotFound(ItemId),

    /// セッションが見つからない
    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),

    /// 無効な状態の場合のエラー
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// バリデーションエラー
    #[error("Validation error: {0}")]
    Validation(String),

    /// 認証・認可エラー
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// 楽観的ロックエラー
    #[error("Optimistic lock error: entity was modified by another process")]
    OptimisticLockError,

    /// 重複エラー
    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    /// 内部エラー
    #[error("Internal error: {0}")]
    Internal(String),
}

impl DomainError {
    /// エンティティが見つからないエラーを生成
    pub fn not_found(entity_type: &'static str, id: impl std::fmt::Display) -> Self {
        Self::NotFound {
            entity_type,
            id: id.to_string(),
        }
    }

    /// エラーが `NotFound` 系かどうか判定
    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::NotFound { .. }
                | Self::UserNotFound(_)
                | Self::ItemNotFound(_)
                | Self::SessionNotFound(_)
        )
    }

    /// エラーが認証・認可関連かどうか判定
    #[must_use]
    pub const fn is_unauthorized(&self) -> bool {
        matches!(self, Self::Unauthorized(_))
    }

    /// エラーがバリデーション関連かどうか判定
    #[must_use]
    pub const fn is_validation_error(&self) -> bool {
        matches!(self, Self::Validation(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_error_should_format_correctly() {
        let error = DomainError::not_found("User", "123");
        assert_eq!(error.to_string(), "Entity not found: User with id 123");
    }

    #[test]
    fn user_not_found_should_display_user_id() {
        let user_id = UserId::new();
        let error = DomainError::UserNotFound(user_id);
        assert!(error.to_string().contains(&user_id.to_string()));
    }

    #[test]
    fn is_not_found_should_identify_not_found_errors() {
        let user_id = UserId::new();
        let item_id = ItemId::new();
        let session_id = SessionId::new();

        assert!(DomainError::not_found("Entity", "123").is_not_found());
        assert!(DomainError::UserNotFound(user_id).is_not_found());
        assert!(DomainError::ItemNotFound(item_id).is_not_found());
        assert!(DomainError::SessionNotFound(session_id).is_not_found());

        assert!(!DomainError::Validation("error".to_string()).is_not_found());
        assert!(!DomainError::OptimisticLockError.is_not_found());
    }

    #[test]
    fn is_unauthorized_should_identify_auth_errors() {
        assert!(DomainError::Unauthorized("no access".to_string()).is_unauthorized());
        assert!(!DomainError::Validation("error".to_string()).is_unauthorized());
    }

    #[test]
    fn is_validation_error_should_identify_validation_errors() {
        assert!(DomainError::Validation("invalid email".to_string()).is_validation_error());
        assert!(!DomainError::Internal("error".to_string()).is_validation_error());
    }

    #[test]
    fn optimistic_lock_error_should_have_descriptive_message() {
        let error = DomainError::OptimisticLockError;
        assert_eq!(
            error.to_string(),
            "Optimistic lock error: entity was modified by another process"
        );
    }

    #[test]
    fn errors_should_be_comparable() {
        let error1 = DomainError::DuplicateEntry("email".to_string());
        let error2 = DomainError::DuplicateEntry("email".to_string());
        let error3 = DomainError::DuplicateEntry("username".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn domain_result_should_work_with_question_mark() -> DomainResult<()> {
        fn may_fail(should_fail: bool) -> DomainResult<String> {
            if should_fail {
                Err(DomainError::InvalidState("test error".to_string()))
            } else {
                Ok("success".to_string())
            }
        }

        // この関数内で ? 演算子が使えることを確認
        let _result = may_fail(false)?;
        Ok(())
    }
}
