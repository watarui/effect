//! Vocabulary Context のドメインイベント

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{CefrLevel, ItemId, UserId};

use crate::domain::{Definition, Domain, PartOfSpeech, Register};

/// 語彙項目が作成された
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyItemCreated {
    pub item_id:        ItemId,
    pub word:           String,
    pub definitions:    Vec<Definition>,
    pub part_of_speech: PartOfSpeech,
    pub cefr_level:     Option<CefrLevel>,
    pub register:       Register,
    pub domain:         Domain,
    pub created_by:     UserId,
    pub created_at:     DateTime<Utc>,
}

/// 語彙項目が更新された
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyItemUpdated {
    pub item_id:     ItemId,
    pub definitions: Vec<Definition>,
    pub cefr_level:  Option<CefrLevel>,
    pub updated_by:  UserId,
    pub updated_at:  DateTime<Utc>,
}

/// 語彙項目が削除された
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyItemDeleted {
    pub item_id:    ItemId,
    pub deleted_by: UserId,
    pub deleted_at: DateTime<Utc>,
}

/// 例文が追加された
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExampleAdded {
    pub item_id:          ItemId,
    pub definition_index: usize,
    pub example:          String,
    pub added_by:         UserId,
    pub added_at:         DateTime<Utc>,
}

/// AI による情報が生成された
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiEnrichmentCompleted {
    pub item_id:       ItemId,
    pub pronunciation: Option<String>,
    pub synonyms:      Vec<String>,
    pub antonyms:      Vec<String>,
    pub collocations:  Vec<String>,
    pub generated_at:  DateTime<Utc>,
}
