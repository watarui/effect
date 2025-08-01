//! アカウント状態

use serde::{Deserialize, Serialize};

/// アカウント状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountStatus {
    /// アクティブ
    Active,
    /// 削除済み（論理削除）
    Deleted,
}

impl AccountStatus {
    /// デフォルト状態（Active）
    #[must_use]
    pub const fn default_status() -> Self {
        Self::Active
    }

    /// アクティブかどうか
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    /// 削除済みかどうか
    #[must_use]
    pub const fn is_deleted(self) -> bool {
        matches!(self, Self::Deleted)
    }
}

impl Default for AccountStatus {
    fn default() -> Self {
        Self::default_status()
    }
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Deleted => write!(f, "Deleted"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_status_should_be_active() {
        assert_eq!(AccountStatus::default(), AccountStatus::Active);
        assert_eq!(AccountStatus::default_status(), AccountStatus::Active);
    }

    #[test]
    fn is_active_should_return_correct_value() {
        assert!(AccountStatus::Active.is_active());
        assert!(!AccountStatus::Deleted.is_active());
    }

    #[test]
    fn is_deleted_should_return_correct_value() {
        assert!(!AccountStatus::Active.is_deleted());
        assert!(AccountStatus::Deleted.is_deleted());
    }

    #[test]
    fn display_should_format_correctly() {
        assert_eq!(AccountStatus::Active.to_string(), "Active");
        assert_eq!(AccountStatus::Deleted.to_string(), "Deleted");
    }

    #[test]
    fn serde_roundtrip() {
        let status = AccountStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: AccountStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);

        let status = AccountStatus::Deleted;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: AccountStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}
