//! ID 値オブジェクト
//!
//! このモジュールは境界づけられたコンテキスト間で使用される全ての ID
//! 値オブジェクトを含みます。

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ユーザー ID 値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    /// 新しい `UserId` を作成
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 内部のUUIDを取得
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// バイト配列として取得
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UserId {
    type Err = uuid::Error;

    /// 文字列から `UserId` を作成
    ///
    /// # Errors
    ///
    /// UUID として無効な文字列が渡された場合はエラーを返します
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// 語彙項目 ID 値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(Uuid);

impl ItemId {
    /// 新しい `ItemId` を作成
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 内部のUUIDを取得
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// バイト配列として取得
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<Uuid> for ItemId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for ItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ItemId {
    type Err = uuid::Error;

    /// 文字列から `ItemId` を作成
    ///
    /// # Errors
    ///
    /// UUID として無効な文字列が渡された場合はエラーを返します
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// 学習セッション ID 値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    /// 新しい `SessionId` を作成
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 内部のUUIDを取得
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SessionId {
    type Err = uuid::Error;

    /// 文字列から `SessionId` を作成
    ///
    /// # Errors
    ///
    /// UUID として無効な文字列が渡された場合はエラーを返します
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// 語彙エントリー ID 値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntryId(Uuid);

impl EntryId {
    /// 新しい `EntryId` を作成
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 内部のUUIDを取得
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// バイト配列として取得
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<Uuid> for EntryId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for EntryId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for EntryId {
    type Err = uuid::Error;

    /// 文字列から `EntryId` を作成
    ///
    /// # Errors
    ///
    /// UUID として無効な文字列が渡された場合はエラーを返します
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// イベント ID 値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    /// 新しい `EventId` を作成
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 内部のUUIDを取得
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for EventId {
    type Err = uuid::Error;

    /// 文字列から `EventId` を作成
    ///
    /// # Errors
    ///
    /// UUID として無効な文字列が渡された場合はエラーを返します
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_id_should_generate_unique_ids() {
        let id1 = UserId::new();
        let id2 = UserId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn user_id_should_parse_from_string() -> Result<(), Box<dyn std::error::Error>> {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let user_id = UserId::from_str(uuid_str)?;
        assert_eq!(user_id.to_string(), uuid_str);
        Ok(())
    }

    #[test]
    fn user_id_should_fail_on_invalid_string() {
        let invalid_str = "not-a-uuid";
        let result = UserId::from_str(invalid_str);
        assert!(result.is_err());
    }

    #[test]
    fn item_id_should_generate_unique_ids() {
        let id1 = ItemId::new();
        let id2 = ItemId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn session_id_should_be_serializable() -> Result<(), Box<dyn std::error::Error>> {
        let session_id = SessionId::new();
        let json = serde_json::to_string(&session_id)?;
        let deserialized: SessionId = serde_json::from_str(&json)?;
        assert_eq!(session_id, deserialized);
        Ok(())
    }

    #[test]
    fn event_id_should_implement_display() {
        let event_id = EventId::new();
        let display_str = event_id.to_string();
        assert!(!display_str.is_empty());
        assert_eq!(display_str.len(), 36); // UUID文字列の長さ
    }

    #[test]
    fn all_id_types_should_have_default() {
        let _user_id = UserId::default();
        let _item_id = ItemId::default();
        let _entry_id = EntryId::default();
        let _session_id = SessionId::default();
        let _event_id = EventId::default();
        // デフォルトインスタンスが作成できることを確認
    }

    #[test]
    fn entry_id_should_generate_unique_ids() {
        let id1 = EntryId::new();
        let id2 = EntryId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn entry_id_should_parse_from_string() -> Result<(), Box<dyn std::error::Error>> {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let entry_id = EntryId::from_str(uuid_str)?;
        assert_eq!(entry_id.to_string(), uuid_str);
        Ok(())
    }

    #[test]
    fn id_should_expose_inner_uuid() {
        let user_id = UserId::new();
        let uuid = user_id.as_uuid();
        assert_eq!(user_id.to_string(), uuid.to_string());
    }
}
