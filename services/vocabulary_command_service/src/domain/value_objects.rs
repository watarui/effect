use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// エントリID（見出し語ID）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntryId(Uuid);

impl Default for EntryId {
    fn default() -> Self {
        Self::new()
    }
}

impl EntryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 語彙項目ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(Uuid);

impl Default for ItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// スペリング（語彙の綴り）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Spelling(String);

impl Spelling {
    pub fn new(value: String) -> Result<Self, String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err("Spelling cannot be empty".to_string());
        }
        if trimmed.len() > 255 {
            return Err("Spelling cannot exceed 255 characters".to_string());
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Spelling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 曖昧性解消（意味の区別）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Disambiguation(Option<String>);

impl Disambiguation {
    pub fn new(value: Option<String>) -> Result<Self, String> {
        match value {
            Some(v) => {
                let trimmed = v.trim();
                if trimmed.is_empty() {
                    Ok(Self(None))
                } else if trimmed.len() > 255 {
                    Err("Disambiguation cannot exceed 255 characters".to_string())
                } else {
                    Ok(Self(Some(trimmed.to_string())))
                }
            },
            None => Ok(Self(None)),
        }
    }

    pub fn as_option(&self) -> Option<&str> {
        self.0.as_deref()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl fmt::Display for Disambiguation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(v) => write!(f, "({})", v),
            None => write!(f, ""),
        }
    }
}

/// 語彙項目のステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VocabularyStatus {
    Draft,
    PendingAI,
    Published,
}

impl VocabularyStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Draft => "draft",
            Self::PendingAI => "pending_ai",
            Self::Published => "published",
        }
    }
}

impl std::str::FromStr for VocabularyStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Self::Draft),
            "pending_ai" => Ok(Self::PendingAI),
            "published" => Ok(Self::Published),
            _ => Err(format!("Invalid VocabularyStatus: {}", s)),
        }
    }
}

impl fmt::Display for VocabularyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// バージョン（楽観的ロック用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version(i64);

impl Version {
    pub fn new(value: i64) -> Result<Self, String> {
        if value < 1 {
            return Err("Version must be positive".to_string());
        }
        Ok(Self(value))
    }

    pub fn initial() -> Self {
        Self(1)
    }

    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spelling_validation() {
        // 正常ケース
        assert!(Spelling::new("apple".to_string()).is_ok());
        assert!(Spelling::new(" apple ".to_string()).is_ok()); // トリミングされる

        // エラーケース
        assert!(Spelling::new("".to_string()).is_err());
        assert!(Spelling::new("   ".to_string()).is_err());
        assert!(Spelling::new("a".repeat(256)).is_err());
    }

    #[test]
    fn test_disambiguation() {
        // None の場合
        let d = Disambiguation::new(None).unwrap();
        assert!(d.is_none());

        // 空文字列の場合（None として扱う）
        let d = Disambiguation::new(Some("  ".to_string())).unwrap();
        assert!(d.is_none());

        // 正常な値
        let d = Disambiguation::new(Some("fruit".to_string())).unwrap();
        assert_eq!(d.as_option(), Some("fruit"));
    }

    #[test]
    fn test_version() {
        let v = Version::initial();
        assert_eq!(v.value(), 1);

        let v2 = v.increment();
        assert_eq!(v2.value(), 2);

        // 不正な値
        assert!(Version::new(0).is_err());
        assert!(Version::new(-1).is_err());
    }
}
