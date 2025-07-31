//! User Service
//!
//! ユーザー管理を担当するマイクロサービス
//! - 認証・認可
//! - ユーザープロフィール管理
//! - ユーザー関連イベントの発行

pub mod domain;

// Re-export main types
pub use domain::{
    aggregates::user::User,
    commands::{
        ChangeUserRole,
        CreateUser,
        DeleteUser,
        TargetLevelUpdate,
        UpdateUserEmail,
        UpdateUserProfile,
    },
    events::UserEventBuilder,
    value_objects::{email::Email, user_profile::UserProfile, user_role::UserRole},
};
