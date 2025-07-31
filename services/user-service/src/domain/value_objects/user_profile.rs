//! ユーザープロフィール

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// CEFR レベル（ヨーロッパ言語共通参照枠）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CefrLevel {
    /// 初級前半
    A1,
    /// 初級後半
    A2,
    /// 中級前半
    B1,
    /// 中級後半
    B2,
    /// 上級前半
    C1,
    /// 上級後半
    C2,
}

impl CefrLevel {
    /// デフォルトレベル（B1）
    #[must_use]
    pub const fn default_level() -> Self {
        Self::B1
    }
}

impl Default for CefrLevel {
    fn default() -> Self {
        Self::default_level()
    }
}

impl std::fmt::Display for CefrLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A1 => write!(f, "A1"),
            Self::A2 => write!(f, "A2"),
            Self::B1 => write!(f, "B1"),
            Self::B2 => write!(f, "B2"),
            Self::C1 => write!(f, "C1"),
            Self::C2 => write!(f, "C2"),
        }
    }
}

/// ユーザープロフィール
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfile {
    /// 表示名
    display_name:          String,
    /// 現在の英語レベル
    current_level:         CefrLevel,
    /// 目標レベル
    target_level:          Option<CefrLevel>,
    /// 1セッションあたりの問題数
    questions_per_session: u8,
    /// プロフィール作成日時
    created_at:            DateTime<Utc>,
    /// プロフィール更新日時
    updated_at:            DateTime<Utc>,
}

/// プロフィールエラー
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    /// 表示名が空
    #[error("Display name is empty")]
    EmptyDisplayName,

    /// 表示名が長すぎる
    #[error("Display name is too long (max 100 characters): {0}")]
    DisplayNameTooLong(usize),

    /// 1セッションあたりの問題数が不正
    #[error("Questions per session must be between 1 and 100, got: {0}")]
    InvalidQuestionsPerSession(u8),

    /// 目標レベルが現在のレベル以下
    #[error("Target level ({target}) must be higher than current level ({current})")]
    InvalidTargetLevel {
        /// 現在のレベル
        current: CefrLevel,
        /// 目標レベル
        target:  CefrLevel,
    },
}

impl UserProfile {
    /// 新しいプロフィールを作成
    ///
    /// # Errors
    ///
    /// * `ProfileError::EmptyDisplayName` - 表示名が空の場合
    /// * `ProfileError::DisplayNameTooLong` - 表示名が100文字を超える場合
    pub fn new(display_name: &str) -> Result<Self, ProfileError> {
        let display_name = display_name.trim().to_string();

        if display_name.is_empty() {
            return Err(ProfileError::EmptyDisplayName);
        }

        if display_name.len() > 100 {
            return Err(ProfileError::DisplayNameTooLong(display_name.len()));
        }

        let now = Utc::now();

        Ok(Self {
            display_name,
            current_level: CefrLevel::default(),
            target_level: None,
            questions_per_session: 50, // デフォルト値
            created_at: now,
            updated_at: now,
        })
    }

    /// 表示名を更新
    ///
    /// # Errors
    ///
    /// * `ProfileError::EmptyDisplayName` - 表示名が空の場合
    /// * `ProfileError::DisplayNameTooLong` - 表示名が100文字を超える場合
    pub fn update_display_name(&mut self, display_name: &str) -> Result<(), ProfileError> {
        let display_name = display_name.trim().to_string();

        if display_name.is_empty() {
            return Err(ProfileError::EmptyDisplayName);
        }

        if display_name.len() > 100 {
            return Err(ProfileError::DisplayNameTooLong(display_name.len()));
        }

        self.display_name = display_name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// レベル設定を更新
    ///
    /// # Errors
    ///
    /// * `ProfileError::InvalidTargetLevel` -
    ///   目標レベルが現在のレベル以下の場合
    pub fn update_levels(
        &mut self,
        current: CefrLevel,
        target: Option<CefrLevel>,
    ) -> Result<(), ProfileError> {
        if let Some(target_level) = target {
            // 目標レベルは現在レベルより高い必要がある
            if target_level as u8 <= current as u8 {
                return Err(ProfileError::InvalidTargetLevel {
                    current,
                    target: target_level,
                });
            }
        }

        self.current_level = current;
        self.target_level = target;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 1セッションあたりの問題数を更新
    ///
    /// # Errors
    ///
    /// * `ProfileError::InvalidQuestionsPerSession` -
    ///   問題数が0または100を超える場合
    pub fn update_questions_per_session(&mut self, count: u8) -> Result<(), ProfileError> {
        if count == 0 || count > 100 {
            return Err(ProfileError::InvalidQuestionsPerSession(count));
        }

        self.questions_per_session = count;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 表示名を取得
    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// 現在のレベルを取得
    #[must_use]
    pub const fn current_level(&self) -> CefrLevel {
        self.current_level
    }

    /// 目標レベルを取得
    #[must_use]
    pub const fn target_level(&self) -> Option<CefrLevel> {
        self.target_level
    }

    /// 1セッションあたりの問題数を取得
    #[must_use]
    pub const fn questions_per_session(&self) -> u8 {
        self.questions_per_session
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_profile_new_should_create_with_defaults() {
        // Given
        let display_name = "Test User";

        // When
        let result = UserProfile::new(display_name);

        // Then
        assert!(result.is_ok());
        let profile = result.unwrap();
        assert_eq!(profile.display_name(), "Test User");
        assert_eq!(profile.current_level(), CefrLevel::B1);
        assert_eq!(profile.target_level(), None);
        assert_eq!(profile.questions_per_session(), 50);
    }

    #[test]
    fn user_profile_new_should_trim_display_name() {
        // Given
        let display_name = "  Test User  ";

        // When
        let result = UserProfile::new(display_name);

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().display_name(), "Test User");
    }

    #[test]
    fn user_profile_new_should_reject_empty_name() {
        // Given
        let display_name = "   ";

        // When
        let result = UserProfile::new(display_name);

        // Then
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProfileError::EmptyDisplayName);
    }

    #[test]
    fn user_profile_update_levels_should_validate_target() {
        // Given
        let mut profile = UserProfile::new("Test").unwrap();

        // When - Valid target
        let result = profile.update_levels(CefrLevel::B1, Some(CefrLevel::C1));
        assert!(result.is_ok());

        // When - Invalid target (same level)
        let result = profile.update_levels(CefrLevel::B1, Some(CefrLevel::B1));
        assert!(result.is_err());

        // When - Invalid target (lower level)
        let result = profile.update_levels(CefrLevel::B2, Some(CefrLevel::A2));
        assert!(result.is_err());
    }

    #[test]
    fn user_profile_update_questions_should_validate_range() {
        // Given
        let mut profile = UserProfile::new("Test").unwrap();

        // When - Valid counts
        assert!(profile.update_questions_per_session(1).is_ok());
        assert!(profile.update_questions_per_session(100).is_ok());
        assert!(profile.update_questions_per_session(50).is_ok());

        // When - Invalid counts
        assert!(profile.update_questions_per_session(0).is_err());
        assert!(profile.update_questions_per_session(101).is_err());
    }
}
