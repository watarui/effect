//! インメモリリポジトリ実装（開発環境用）

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use common_types::UserId;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::{domain::aggregates::user::User, ports::outbound::UserRepository};

/// リポジトリエラー
#[derive(Error, Debug)]
pub enum Error {
    /// ユーザーが見つからない
    #[error("User not found")]
    NotFound,
    /// Email が既に使用されている
    #[error("Email already exists")]
    EmailAlreadyExists,
    /// 内部エラー
    #[error("Internal repository error: {0}")]
    Internal(String),
}

/// インメモリユーザーリポジトリ
///
/// 開発環境用のリポジトリ実装。
/// データはメモリ上に保存され、アプリケーション再起動時に失われる。
#[derive(Debug, Clone)]
pub struct InMemoryRepository {
    /// ユーザー ID をキーとするユーザーデータ
    users_by_id:       Arc<RwLock<HashMap<UserId, User>>>,
    /// Email をキーとするユーザー ID のインデックス
    user_ids_by_email: Arc<RwLock<HashMap<String, UserId>>>,
}

impl InMemoryRepository {
    /// 新しいインメモリリポジトリを作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            users_by_id:       Arc::new(RwLock::new(HashMap::new())),
            user_ids_by_email: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 事前定義されたユーザーでリポジトリを初期化
    pub async fn with_users(users: Vec<User>) -> Self {
        let repo = Self::new();

        let mut users_by_id = repo.users_by_id.write().await;
        let mut user_ids_by_email = repo.user_ids_by_email.write().await;

        for user in users {
            let user_id = *user.id();
            let email = user.email().as_str().to_string();

            users_by_id.insert(user_id, user);
            user_ids_by_email.insert(email, user_id);
        }

        drop(users_by_id);
        drop(user_ids_by_email);

        repo
    }
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for InMemoryRepository {
    type Error = Error;

    async fn save(&self, user: &User) -> Result<(), Self::Error> {
        let user_id = *user.id();
        let email = user.email().as_str().to_string();

        // Email の重複チェック
        {
            let user_ids_by_email = self.user_ids_by_email.read().await;
            if let Some(existing_id) = user_ids_by_email.get(&email)
                && *existing_id != user_id
            {
                return Err(Error::EmailAlreadyExists);
            }
        }

        // 古い Email を削除（Email 更新の場合）
        let mut users_by_id = self.users_by_id.write().await;
        if let Some(existing_user) = users_by_id.get(&user_id) {
            let old_email = existing_user.email().as_str();
            if old_email != email {
                let mut user_ids_by_email = self.user_ids_by_email.write().await;
                user_ids_by_email.remove(old_email);
            }
        }

        // ユーザーを保存
        users_by_id.insert(user_id, user.clone());
        drop(users_by_id);

        // Email インデックスを更新
        self.user_ids_by_email.write().await.insert(email, user_id);

        Ok(())
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, Self::Error> {
        let users = self.users_by_id.read().await;
        Ok(users.get(id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Self::Error> {
        let user_ids_by_email = self.user_ids_by_email.read().await;
        if let Some(user_id) = user_ids_by_email.get(email) {
            let users = self.users_by_id.read().await;
            Ok(users.get(user_id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, id: &UserId) -> Result<(), Self::Error> {
        let mut users_by_id = self.users_by_id.write().await;
        if let Some(user) = users_by_id.remove(id) {
            let email = user.email().as_str().to_string();
            drop(users_by_id);

            self.user_ids_by_email.write().await.remove(&email);
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    async fn is_first_user(&self) -> Result<bool, Self::Error> {
        let users = self.users_by_id.read().await;
        Ok(users.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::email::Email;

    fn create_test_user(email: &str) -> User {
        User::create(
            UserId::new(),
            Email::new(email).unwrap(),
            "Test User",
            false,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn save_and_find_by_id_should_work() {
        // Given
        let repo = InMemoryRepository::new();
        let user = create_test_user("test@example.com");
        let user_id = *user.id();

        // When
        let save_result = repo.save(&user).await;
        let find_result = repo.find_by_id(&user_id).await;

        // Then
        assert!(save_result.is_ok());
        assert!(find_result.is_ok());
        assert_eq!(find_result.unwrap(), Some(user));
    }

    #[tokio::test]
    async fn save_and_find_by_email_should_work() {
        // Given
        let repo = InMemoryRepository::new();
        let user = create_test_user("test@example.com");

        // When
        let save_result = repo.save(&user).await;
        let find_result = repo.find_by_email("test@example.com").await;

        // Then
        assert!(save_result.is_ok());
        assert!(find_result.is_ok());
        assert_eq!(find_result.unwrap(), Some(user));
    }

    #[tokio::test]
    async fn save_duplicate_email_should_fail() {
        // Given
        let repo = InMemoryRepository::new();
        let user1 = create_test_user("test@example.com");
        let user2 = create_test_user("test@example.com");

        // When
        let save1_result = repo.save(&user1).await;
        let save2_result = repo.save(&user2).await;

        // Then
        assert!(save1_result.is_ok());
        assert!(save2_result.is_err());
        assert!(matches!(
            save2_result.unwrap_err(),
            Error::EmailAlreadyExists
        ));
    }

    #[tokio::test]
    async fn update_email_should_work() {
        // Given
        let repo = InMemoryRepository::new();
        let mut user = create_test_user("old@example.com");
        repo.save(&user).await.unwrap();

        // When
        user.update_email(Email::new("new@example.com").unwrap());
        let update_result = repo.save(&user).await;

        // Then
        assert!(update_result.is_ok());
        assert!(
            repo.find_by_email("old@example.com")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            repo.find_by_email("new@example.com")
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn delete_should_remove_user() {
        // Given
        let repo = InMemoryRepository::new();
        let user = create_test_user("test@example.com");
        let user_id = *user.id();
        repo.save(&user).await.unwrap();

        // When
        let delete_result = repo.delete(&user_id).await;

        // Then
        assert!(delete_result.is_ok());
        assert!(repo.find_by_id(&user_id).await.unwrap().is_none());
        assert!(
            repo.find_by_email("test@example.com")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn is_first_user_should_return_true_when_empty() {
        // Given
        let repo = InMemoryRepository::new();

        // When
        let result = repo.is_first_user().await;

        // Then
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn is_first_user_should_return_false_when_users_exist() {
        // Given
        let repo = InMemoryRepository::new();
        let user = create_test_user("test@example.com");
        repo.save(&user).await.unwrap();

        // When
        let result = repo.is_first_user().await;

        // Then
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
