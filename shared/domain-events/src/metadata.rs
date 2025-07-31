//! イベントメタデータ
//!
//! このモジュールは全てのドメインイベントに共通のメタデータ構造を含みます。

use chrono::Utc;
use common_types::{EventId, Timestamp};
use serde::{Deserialize, Serialize};

/// 全てのドメインイベントに共通のメタデータ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventMetadata {
    /// イベント ID
    pub event_id:    EventId,
    /// 発生日時
    pub occurred_at: Timestamp,
    /// バージョン
    pub version:     String,
}

impl EventMetadata {
    /// 新しいイベントメタデータを作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            event_id:    EventId::new(),
            occurred_at: Utc::now(),
            version:     "1.0".to_string(),
        }
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new()
    }
}
