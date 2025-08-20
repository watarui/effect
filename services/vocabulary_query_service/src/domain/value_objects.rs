//! 値オブジェクト

use serde::{Deserialize, Serialize};

/// ページネーションのカーソル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor(String);

impl Cursor {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

/// ページサイズ
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PageSize(u32);

impl PageSize {
    pub const DEFAULT: u32 = 20;
    pub const MAX: u32 = 100;

    pub fn new(size: u32) -> Self {
        let size = size.min(Self::MAX);
        Self(size)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for PageSize {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

/// 検索クエリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub term:       String,
    pub min_length: usize,
}

impl SearchQuery {
    pub fn new(term: String) -> Option<Self> {
        let trimmed = term.trim();
        if trimmed.len() < 2 {
            return None;
        }
        Some(Self {
            term:       trimmed.to_string(),
            min_length: 2,
        })
    }

    pub fn is_valid(&self) -> bool {
        self.term.len() >= self.min_length
    }
}
