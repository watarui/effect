//! Mock 認証プロバイダー（開発環境用）

use std::sync::Arc;

use async_trait::async_trait;
use common_types::UserId;
use tokio::sync::RwLock;

use super::Error;
use crate::ports::outbound::AuthProvider;

/// Mock 認証プロバイダー
///
/// 開発環境用の認証プロバイダー実装。
/// 実際の認証は行わず、トークンとユーザー ID の単純なマッピングを提供。
#[derive(Debug, Clone)]
pub struct Provider {
    /// トークンとユーザー ID のマッピング
    tokens: Arc<RwLock<std::collections::HashMap<String, UserId>>>,
}

impl Provider {
    /// 新しい `Provider` を作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 事前定義されたユーザーでプロバイダーを初期化
    ///
    /// # Arguments
    ///
    /// * `predefined_users` - (token, `user_id`) のペアのベクタ
    pub async fn with_predefined_users(predefined_users: Vec<(String, UserId)>) -> Self {
        let provider = Self::new();
        let mut tokens = provider.tokens.write().await;
        for (token, user_id) in predefined_users {
            tokens.insert(token, user_id);
        }
        drop(tokens);
        provider
    }
}

impl Default for Provider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for Provider {
    type Error = Error;

    async fn verify_token(&self, token: &str) -> Result<UserId, Self::Error> {
        let tokens = self.tokens.read().await;
        tokens.get(token).copied().ok_or(Error::InvalidToken)
    }

    async fn generate_token(&self, user_id: &UserId) -> Result<String, Self::Error> {
        // Mock トークンを生成（開発環境用）
        let token = format!("mock_token_{user_id}");
        self.tokens.write().await.insert(token.clone(), *user_id);
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_auth_provider_should_verify_valid_token() {
        // Given
        let user_id = UserId::new();
        let token = "test_token";
        let provider = Provider::with_predefined_users(vec![(token.to_string(), user_id)]).await;

        // When
        let result = provider.verify_token(token).await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user_id);
    }

    #[tokio::test]
    async fn mock_auth_provider_should_reject_invalid_token() {
        // Given
        let provider = Provider::new();
        let invalid_token = "invalid_token";

        // When
        let result = provider.verify_token(invalid_token).await;

        // Then
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidToken));
    }

    #[tokio::test]
    async fn mock_auth_provider_should_generate_and_verify_token() {
        // Given
        let provider = Provider::new();
        let user_id = UserId::new();

        // When
        let token_result = provider.generate_token(&user_id).await;
        assert!(token_result.is_ok());
        let token = token_result.unwrap();

        let verify_result = provider.verify_token(&token).await;

        // Then
        assert!(verify_result.is_ok());
        assert_eq!(verify_result.unwrap(), user_id);
    }
}
