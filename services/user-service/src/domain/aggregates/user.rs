//! User 集約

use chrono::{DateTime, Utc};
use common_types::UserId;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{
    account_status::AccountStatus,
    email::Email,
    user_profile::UserProfile,
    user_role::UserRole,
};

/// User 集約ルート
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// ユーザー ID
    id:         UserId,
    /// Email アドレス（一意）
    email:      Email,
    /// ユーザープロフィール
    profile:    UserProfile,
    /// ユーザーロール
    role:       UserRole,
    /// アカウント状態
    status:     AccountStatus,
    /// アカウント作成日時
    created_at: DateTime<Utc>,
    /// アカウント更新日時
    updated_at: DateTime<Utc>,
    /// 楽観的ロック用バージョン
    version:    u64,
}

impl User {
    /// 新しいユーザーを作成
    ///
    /// # Arguments
    ///
    /// * `id` - ユーザー ID
    /// * `email` - Email アドレス
    /// * `display_name` - 表示名
    /// * `is_first_user` - 最初のユーザーかどうか（true の場合 Admin ロール）
    ///
    /// # Errors
    ///
    /// * `ProfileError` - プロフィール作成に失敗した場合
    pub fn create(
        id: UserId,
        email: Email,
        display_name: &str,
        is_first_user: bool,
    ) -> Result<Self, crate::domain::value_objects::user_profile::ProfileError> {
        let profile = UserProfile::new(display_name)?;
        let role = if is_first_user {
            UserRole::Admin
        } else {
            UserRole::default_role()
        };

        let now = Utc::now();

        Ok(Self {
            id,
            email,
            profile,
            role,
            status: AccountStatus::default(),
            created_at: now,
            updated_at: now,
            version: 0,
        })
    }

    /// 指定されたロールで新しいユーザーを作成
    ///
    /// # Errors
    ///
    /// * `ProfileError` - プロフィールの検証に失敗した場合
    pub fn new_with_role(
        id: UserId,
        email: Email,
        display_name: &str,
        role: UserRole,
    ) -> Result<Self, crate::domain::value_objects::user_profile::ProfileError> {
        let profile = UserProfile::new(display_name)?;
        let now = Utc::now();

        Ok(Self {
            id,
            email,
            profile,
            role,
            status: AccountStatus::default(),
            created_at: now,
            updated_at: now,
            version: 0,
        })
    }

    /// プロフィールを更新
    ///
    /// # Errors
    ///
    /// * `ProfileError` - プロフィール更新に失敗した場合
    pub fn update_profile<F>(
        &mut self,
        update_fn: F,
    ) -> Result<(), crate::domain::value_objects::user_profile::ProfileError>
    where
        F: FnOnce(
            &mut UserProfile,
        ) -> Result<(), crate::domain::value_objects::user_profile::ProfileError>,
    {
        update_fn(&mut self.profile)?;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// ロールを変更
    pub fn change_role(&mut self, new_role: UserRole) {
        if self.role != new_role {
            self.role = new_role;
            self.updated_at = Utc::now();
            self.version += 1;
        }
    }

    /// Email を更新
    pub fn update_email(&mut self, new_email: Email) {
        if self.email != new_email {
            self.email = new_email;
            self.updated_at = Utc::now();
            self.version += 1;
        }
    }

    /// ユーザー ID を取得
    #[must_use]
    pub const fn id(&self) -> &UserId {
        &self.id
    }

    /// Email アドレスを取得
    #[must_use]
    pub const fn email(&self) -> &Email {
        &self.email
    }

    /// プロフィールを取得
    #[must_use]
    pub const fn profile(&self) -> &UserProfile {
        &self.profile
    }

    /// ロールを取得
    #[must_use]
    pub const fn role(&self) -> UserRole {
        self.role
    }

    /// 作成日時を取得
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// 更新日時を取得
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// アカウント状態を取得
    #[must_use]
    pub const fn status(&self) -> AccountStatus {
        self.status
    }

    /// バージョンを取得
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    /// 管理者権限を持っているか
    #[must_use]
    pub const fn is_admin(&self) -> bool {
        self.role.is_admin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_create_should_set_admin_role_for_first_user() {
        // Given
        let id = UserId::new();
        let email = Email::new("admin@example.com").unwrap();
        let display_name = "Admin User";

        // When
        let result = User::create(id, email, display_name, true);

        // Then
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.role(), UserRole::Admin);
        assert!(user.is_admin());
    }

    #[test]
    fn user_create_should_set_user_role_for_non_first_user() {
        // Given
        let id = UserId::new();
        let email = Email::new("user@example.com").unwrap();
        let display_name = "Normal User";

        // When
        let result = User::create(id, email, display_name, false);

        // Then
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.role(), UserRole::User);
        assert!(!user.is_admin());
    }

    #[test]
    fn user_update_profile_should_update_timestamp() {
        // Given
        let mut user = User::create(
            UserId::new(),
            Email::new("user@example.com").unwrap(),
            "Test User",
            false,
        )
        .unwrap();

        let original_updated_at = user.updated_at();

        // Wait a bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        // When
        let result = user.update_profile(|profile| profile.update_display_name("New Name"));

        // Then
        assert!(result.is_ok());
        assert_eq!(user.profile().display_name(), "New Name");
        assert!(user.updated_at() > original_updated_at);
    }

    #[test]
    fn user_change_role_should_update_timestamp() {
        // Given
        let mut user = User::create(
            UserId::new(),
            Email::new("user@example.com").unwrap(),
            "Test User",
            false,
        )
        .unwrap();

        let original_updated_at = user.updated_at();

        // Wait a bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        // When
        user.change_role(UserRole::Admin);

        // Then
        assert_eq!(user.role(), UserRole::Admin);
        assert!(user.updated_at() > original_updated_at);
    }

    #[test]
    fn user_update_email_should_update_timestamp() {
        // Given
        let mut user = User::create(
            UserId::new(),
            Email::new("old@example.com").unwrap(),
            "Test User",
            false,
        )
        .unwrap();

        let original_updated_at = user.updated_at();
        let new_email = Email::new("new@example.com").unwrap();

        // Wait a bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        // When
        user.update_email(new_email.clone());

        // Then
        assert_eq!(user.email(), &new_email);
        assert!(user.updated_at() > original_updated_at);
    }
}
