//! ユーザーロール

use serde::{Deserialize, Serialize};

/// ユーザーのロール
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// 管理者
    Admin,
    /// 一般ユーザー
    User,
}

impl UserRole {
    /// 管理者権限を持っているか
    #[must_use]
    pub const fn is_admin(self) -> bool {
        matches!(self, Self::Admin)
    }

    /// デフォルトのロール（一般ユーザー）
    #[must_use]
    pub const fn default_role() -> Self {
        Self::User
    }
}

impl Default for UserRole {
    fn default() -> Self {
        Self::default_role()
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "Admin"),
            Self::User => write!(f, "User"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_role_is_admin_should_return_true_for_admin() {
        // Given
        let role = UserRole::Admin;

        // When & Then
        assert!(role.is_admin());
    }

    #[test]
    fn user_role_is_admin_should_return_false_for_user() {
        // Given
        let role = UserRole::User;

        // When & Then
        assert!(!role.is_admin());
    }

    #[test]
    fn user_role_default_should_be_user() {
        // When
        let role = UserRole::default();

        // Then
        assert_eq!(role, UserRole::User);
    }

    #[test]
    fn user_role_serialization() {
        // Given
        let admin = UserRole::Admin;
        let user = UserRole::User;

        // When
        let admin_json = serde_json::to_string(&admin).unwrap();
        let user_json = serde_json::to_string(&user).unwrap();

        // Then
        assert_eq!(admin_json, r#""admin""#);
        assert_eq!(user_json, r#""user""#);
    }
}
