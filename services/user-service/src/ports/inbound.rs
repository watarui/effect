//! インバウンドポート（外部からの入力を受け付けるインターフェース）

use async_trait::async_trait;
use common_types::UserId;

use crate::domain::{
    aggregates::user::User,
    commands::{ChangeUserRole, CreateUser, DeleteUser, UpdateUserEmail, UpdateUserProfile},
};

/// ユーザーサービスのユースケースインターフェース
#[async_trait]
pub trait UserUseCase: Send + Sync {
    /// エラー型
    type Error: std::error::Error + Send + Sync + 'static;

    /// ユーザーを作成する
    ///
    /// # Errors
    ///
    /// * ユーザー作成に失敗した場合
    async fn create_user(&self, command: CreateUser) -> Result<User, Self::Error>;

    /// ユーザーを ID で取得する
    ///
    /// # Errors
    ///
    /// * ユーザーが見つからない場合
    async fn get_user(&self, user_id: &UserId) -> Result<User, Self::Error>;

    /// Email でユーザーを取得する
    ///
    /// # Errors
    ///
    /// * ユーザーが見つからない場合
    async fn get_user_by_email(&self, email: &str) -> Result<User, Self::Error>;

    /// ユーザープロフィールを更新する
    ///
    /// # Errors
    ///
    /// * 更新に失敗した場合
    async fn update_profile(&self, command: UpdateUserProfile) -> Result<User, Self::Error>;

    /// ユーザーのロールを変更する
    ///
    /// # Errors
    ///
    /// * ロール変更に失敗した場合
    async fn change_role(&self, command: ChangeUserRole) -> Result<User, Self::Error>;

    /// ユーザーの Email を更新する
    ///
    /// # Errors
    ///
    /// * Email 更新に失敗した場合
    async fn update_email(&self, command: UpdateUserEmail) -> Result<User, Self::Error>;

    /// ユーザーを削除する
    ///
    /// # Errors
    ///
    /// * 削除に失敗した場合
    async fn delete_user(&self, command: DeleteUser) -> Result<(), Self::Error>;
}
