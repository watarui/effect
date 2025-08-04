//! 検索関連の値オブジェクト

#![allow(clippy::should_implement_trait)]

use std::fmt;

use serde::{Deserialize, Serialize};

/// 検索クエリ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchQuery(String);

impl SearchQuery {
    /// 新しい検索クエリを作成
    pub fn new(query: impl Into<String>) -> Result<Self, SearchQueryError> {
        let query = query.into().trim().to_string();

        if query.is_empty() {
            return Err(SearchQueryError::Empty);
        }

        if query.len() > 1000 {
            return Err(SearchQueryError::TooLong);
        }

        Ok(Self(query))
    }

    /// クエリ文字列を取得
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// クエリを消費して String に変換
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for SearchQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SearchQueryError {
    #[error("search query cannot be empty")]
    Empty,

    #[error("search query is too long (max 1000 characters)")]
    TooLong,
}

/// 検索スコア
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SearchScore(f32);

impl SearchScore {
    /// 新しい検索スコアを作成
    pub fn new(score: f32) -> Result<Self, SearchScoreError> {
        if score < 0.0 {
            return Err(SearchScoreError::Negative);
        }

        if score.is_nan() || score.is_infinite() {
            return Err(SearchScoreError::Invalid);
        }

        Ok(Self(score))
    }

    /// スコア値を取得
    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for SearchScore {
    fn default() -> Self {
        Self(0.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SearchScoreError {
    #[error("search score cannot be negative")]
    Negative,

    #[error("search score must be a valid number")]
    Invalid,
}

/// ページネーション
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pagination {
    pub offset: u32,
    pub limit:  u32,
}

impl Pagination {
    /// 新しいページネーションを作成
    pub fn new(offset: u32, limit: u32) -> Result<Self, PaginationError> {
        if limit == 0 {
            return Err(PaginationError::ZeroLimit);
        }

        if limit > 100 {
            return Err(PaginationError::LimitTooLarge);
        }

        if offset > 10000 {
            return Err(PaginationError::OffsetTooLarge);
        }

        Ok(Self { offset, limit })
    }

    /// デフォルトのページネーション（最初の10件）
    pub fn default_first_page() -> Self {
        Self {
            offset: 0,
            limit:  10,
        }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::default_first_page()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PaginationError {
    #[error("limit cannot be zero")]
    ZeroLimit,

    #[error("limit is too large (max 100)")]
    LimitTooLarge,

    #[error("offset is too large (max 10000)")]
    OffsetTooLarge,
}

/// ファジネス（あいまい度）
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Fuzziness(f32);

impl Fuzziness {
    /// 新しいファジネスを作成（0.0 - 1.0）
    pub fn new(value: f32) -> Result<Self, FuzzinessError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(FuzzinessError::OutOfRange);
        }

        Ok(Self(value))
    }

    /// ファジネス値を取得
    pub fn value(&self) -> f32 {
        self.0
    }

    /// 厳密一致（ファジネスなし）
    pub fn exact() -> Self {
        Self(0.0)
    }

    /// デフォルトのファジネス（中程度）
    pub fn default() -> Self {
        Self(0.3)
    }

    /// 高いファジネス
    pub fn high() -> Self {
        Self(0.8)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FuzzinessError {
    #[error("fuzziness must be between 0.0 and 1.0")]
    OutOfRange,
}
