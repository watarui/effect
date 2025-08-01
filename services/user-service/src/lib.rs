//! User Service
//!
//! ユーザー管理を担当するマイクロサービス
//! - 認証・認可
//! - ユーザープロフィール管理
//! - ユーザー関連イベントの発行

// Proto 生成コードの clippy 警告を抑制
#![allow(clippy::module_name_repetitions)]

pub mod adapters;
pub mod application;
pub mod config;
pub mod domain;
pub mod ports;

// Re-export main types
pub use domain::{
    aggregates::user::User,
    commands::{
        ChangeUserRole,
        CreateUser,
        DeleteUser,
        SetLearningGoal,
        UpdateUserEmail,
        UpdateUserProfile,
    },
    events::UserEventBuilder,
    value_objects::{email::Email, user_profile::UserProfile, user_role::UserRole},
};
