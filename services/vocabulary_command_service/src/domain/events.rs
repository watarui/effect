//! ドメインイベント
//!
//! Vocabulary ドメインで発生するイベント

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::value_objects::{Domain, PartOfSpeech, Register};

/// Vocabulary ドメインイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VocabularyDomainEvent {
    /// エントリ作成
    EntryCreated {
        entry_id:    Uuid,
        word:        String,
        occurred_at: DateTime<Utc>,
    },
    /// 項目作成
    ItemCreated {
        item_id:        Uuid,
        entry_id:       Uuid,
        word:           String,
        definitions:    Vec<String>,
        part_of_speech: PartOfSpeech,
        register:       Register,
        domain:         Domain,
        created_by:     Uuid,
        occurred_at:    DateTime<Utc>,
    },
    /// 項目更新
    ItemUpdated {
        item_id:     Uuid,
        field_name:  String,
        old_value:   serde_json::Value,
        new_value:   serde_json::Value,
        updated_by:  Uuid,
        occurred_at: DateTime<Utc>,
    },
    /// 項目公開
    ItemPublished {
        item_id:      Uuid,
        published_by: Uuid,
        occurred_at:  DateTime<Utc>,
    },
}

impl VocabularyDomainEvent {
    /// イベントの発生日時を取得
    pub fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            Self::EntryCreated { occurred_at, .. }
            | Self::ItemCreated { occurred_at, .. }
            | Self::ItemUpdated { occurred_at, .. }
            | Self::ItemPublished { occurred_at, .. } => *occurred_at,
        }
    }

    /// イベントIDを生成
    pub fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    /// イベントタイプを取得
    pub fn event_type(&self) -> String {
        match self {
            Self::EntryCreated { .. } => "EntryCreated".to_string(),
            Self::ItemCreated { .. } => "ItemCreated".to_string(),
            Self::ItemUpdated { .. } => "ItemUpdated".to_string(),
            Self::ItemPublished { .. } => "ItemPublished".to_string(),
        }
    }
}
