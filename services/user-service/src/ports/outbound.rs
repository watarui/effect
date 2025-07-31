//! アウトバウンドポート（外部リソースへアクセスするインターフェース）

use async_trait::async_trait;
use common_types::UserId;
use domain_events::DomainEvent;

use crate::domain::aggregates::user::User;

/// ユーザーリポジトリインターフェース
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// エラー型
    type Error: std::error::Error + Send + Sync + 'static;

    /// ユーザーを保存する
    ///
    /// # Errors
    ///
    /// * 保存に失敗した場合
    async fn save(&self, user: &User) -> Result<(), Self::Error>;

    /// ID でユーザーを取得する
    ///
    /// # Errors
    ///
    /// * 取得に失敗した場合
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, Self::Error>;

    /// Email でユーザーを取得する
    ///
    /// # Errors
    ///
    /// * 取得に失敗した場合
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Self::Error>;

    /// ユーザーを削除する
    ///
    /// # Errors
    ///
    /// * 削除に失敗した場合
    async fn delete(&self, id: &UserId) -> Result<(), Self::Error>;

    /// 最初のユーザーかどうかを確認する
    ///
    /// # Errors
    ///
    /// * 確認に失敗した場合
    async fn is_first_user(&self) -> Result<bool, Self::Error>;
}

/// イベントパブリッシャーインターフェース
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// エラー型
    type Error: std::error::Error + Send + Sync + 'static;

    /// イベントを発行する
    ///
    /// # Errors
    ///
    /// * 発行に失敗した場合
    async fn publish(&self, event: &DomainEvent) -> Result<(), Self::Error>;
}

/// 認証プロバイダーインターフェース
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// エラー型
    type Error: std::error::Error + Send + Sync + 'static;

    /// トークンからユーザー ID を取得する
    ///
    /// # Errors
    ///
    /// * トークンが無効な場合
    async fn verify_token(&self, token: &str) -> Result<UserId, Self::Error>;

    /// ユーザー ID からトークンを生成する（Mock 用）
    ///
    /// # Errors
    ///
    /// * トークン生成に失敗した場合
    async fn generate_token(&self, user_id: &UserId) -> Result<String, Self::Error>;
}
