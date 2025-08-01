//! ユーザードメインサービス
//!
//! 複数の集約やリポジトリにまたがるビジネスロジックを扱う

use std::sync::Arc;

use async_trait::async_trait;

use crate::{domain::value_objects::user_role::UserRole, ports::outbound::UserRepository};

/// ユーザー関連のドメインロジックを提供するサービス
#[async_trait]
pub trait UserDomainService: Send + Sync {
    /// 新規ユーザーが Admin 権限を持つべきかを判定する
    ///
    /// ビジネスルール:
    /// - 明示的に `first_user` フラグが設定されている場合は Admin
    /// - システムに他のユーザーが存在しない場合は Admin
    /// - それ以外は通常のユーザー権限
    async fn determine_initial_role(
        &self,
        is_explicitly_first_user: bool,
    ) -> Result<UserRole, DomainServiceError>;
}

/// `UserDomainService` の具象実装
pub struct UserDomainServiceImpl<R>
where
    R: UserRepository,
{
    repository: Arc<R>,
}

impl<R> UserDomainServiceImpl<R>
where
    R: UserRepository,
{
    /// 新しいインスタンスを作成
    pub const fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> UserDomainService for UserDomainServiceImpl<R>
where
    R: UserRepository + Send + Sync,
{
    async fn determine_initial_role(
        &self,
        is_explicitly_first_user: bool,
    ) -> Result<UserRole, DomainServiceError> {
        if is_explicitly_first_user {
            return Ok(UserRole::Admin);
        }

        // システムに他のユーザーが存在するかチェック
        match self.repository.is_first_user().await {
            Ok(is_first) => {
                if is_first {
                    Ok(UserRole::Admin)
                } else {
                    Ok(UserRole::User)
                }
            },
            Err(e) => Err(DomainServiceError::RepositoryError(e.to_string())),
        }
    }
}

/// ドメインサービスのエラー
#[derive(Debug, thiserror::Error)]
pub enum DomainServiceError {
    /// リポジトリエラー
    #[error("Repository error: {0}")]
    RepositoryError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapters::outbound::repository::memory::InMemoryRepository,
        domain::{aggregates::user::User, value_objects::email::Email},
    };

    #[tokio::test]
    async fn test_determine_initial_role_explicitly_first_user() {
        // Given
        let repository = Arc::new(InMemoryRepository::new());
        let service = UserDomainServiceImpl::new(repository);

        // When
        let role = service.determine_initial_role(true).await.unwrap();

        // Then
        assert_eq!(role, UserRole::Admin);
    }

    #[tokio::test]
    async fn test_determine_initial_role_first_user_in_system() {
        // Given
        let repository = Arc::new(InMemoryRepository::new());
        let service = UserDomainServiceImpl::new(repository);

        // When
        let role = service.determine_initial_role(false).await.unwrap();

        // Then
        assert_eq!(role, UserRole::Admin);
    }

    #[tokio::test]
    async fn test_determine_initial_role_not_first_user() {
        use common_types::UserId;

        // Given
        let repository = Arc::new(InMemoryRepository::new());

        // 既存ユーザーを追加
        let existing_user = User::create(
            UserId::new(),
            Email::new("existing@example.com").unwrap(),
            "Existing User",
            true, // Admin user
        )
        .unwrap();
        repository.save(&existing_user).await.unwrap();

        let service = UserDomainServiceImpl::new(repository);

        // When
        let role = service.determine_initial_role(false).await.unwrap();

        // Then
        assert_eq!(role, UserRole::User);
    }
}
