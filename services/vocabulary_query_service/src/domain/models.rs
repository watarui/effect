//! クエリ用ドメインモデル

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 語彙エントリのクエリモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyEntry {
    pub entry_id:        Uuid,
    pub spelling:        String,
    pub primary_item_id: Option<Uuid>,
    pub item_count:      i32,
    #[serde(default)]
    pub items:           Vec<VocabularyItem>,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

/// 語彙エントリの DB レコード
#[derive(Debug, Clone, FromRow)]
pub struct VocabularyEntryRow {
    pub entry_id:        Uuid,
    pub spelling:        String,
    pub primary_item_id: Option<Uuid>,
    pub item_count:      i32,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

impl From<VocabularyEntryRow> for VocabularyEntry {
    fn from(row: VocabularyEntryRow) -> Self {
        Self {
            entry_id:        row.entry_id,
            spelling:        row.spelling,
            primary_item_id: row.primary_item_id,
            item_count:      row.item_count,
            items:           Vec::new(),
            created_at:      row.created_at,
            updated_at:      row.updated_at,
        }
    }
}

/// 語彙項目のクエリモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyItem {
    pub item_id:           Uuid,
    pub entry_id:          Uuid,
    pub spelling:          String,
    pub disambiguation:    Option<String>,
    pub part_of_speech:    Option<String>,
    pub definition:        Option<String>,
    pub ipa_pronunciation: Option<String>,
    pub cefr_level:        Option<String>,
    pub frequency_rank:    Option<i32>,
    pub is_published:      bool,
    pub is_deleted:        bool,
    pub example_count:     i32,
    #[serde(default)]
    pub examples:          Vec<VocabularyExample>,
    pub created_at:        DateTime<Utc>,
    pub updated_at:        DateTime<Utc>,
}

/// 語彙項目の DB レコード
#[derive(Debug, Clone, FromRow)]
pub struct VocabularyItemRow {
    pub item_id:           Uuid,
    pub entry_id:          Uuid,
    pub spelling:          String,
    pub disambiguation:    Option<String>,
    pub part_of_speech:    Option<String>,
    pub definition:        Option<String>,
    pub ipa_pronunciation: Option<String>,
    pub cefr_level:        Option<String>,
    pub frequency_rank:    Option<i32>,
    pub is_published:      bool,
    pub is_deleted:        bool,
    pub example_count:     i32,
    pub created_at:        DateTime<Utc>,
    pub updated_at:        DateTime<Utc>,
}

impl From<VocabularyItemRow> for VocabularyItem {
    fn from(row: VocabularyItemRow) -> Self {
        Self {
            item_id:           row.item_id,
            entry_id:          row.entry_id,
            spelling:          row.spelling,
            disambiguation:    row.disambiguation,
            part_of_speech:    row.part_of_speech,
            definition:        row.definition,
            ipa_pronunciation: row.ipa_pronunciation,
            cefr_level:        row.cefr_level,
            frequency_rank:    row.frequency_rank,
            is_published:      row.is_published,
            is_deleted:        row.is_deleted,
            example_count:     row.example_count,
            examples:          Vec::new(),
            created_at:        row.created_at,
            updated_at:        row.updated_at,
        }
    }
}

/// 例文のクエリモデル
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VocabularyExample {
    pub example_id:  Uuid,
    pub item_id:     Uuid,
    pub example:     String,
    pub translation: Option<String>,
    pub added_by:    Option<Uuid>,
    pub created_at:  DateTime<Utc>,
}

/// ページネーション情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub has_next_page:     bool,
    pub has_previous_page: bool,
    pub start_cursor:      Option<String>,
    pub end_cursor:        Option<String>,
    pub total_count:       Option<i64>,
}

/// ページネーション付き結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult<T> {
    pub items:     Vec<T>,
    pub page_info: PageInfo,
}

/// フィルター条件
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VocabularyFilter {
    pub search_term:    Option<String>,
    pub part_of_speech: Option<String>,
    pub cefr_level:     Option<String>,
    pub is_published:   Option<bool>,
    pub has_definition: Option<bool>,
    pub has_examples:   Option<bool>,
    pub min_frequency:  Option<i32>,
    pub max_frequency:  Option<i32>,
}

/// ソート条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    #[serde(rename = "ASC")]
    Ascending,
    #[serde(rename = "DESC")]
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortField {
    Spelling,
    FrequencyRank,
    CefrLevel,
    CreatedAt,
    UpdatedAt,
    ExampleCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOptions {
    pub field: SortField,
    pub order: SortOrder,
}

/// 統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyStatistics {
    pub total_entries:   i64,
    pub total_items:     i64,
    pub total_examples:  i64,
    pub published_items: i64,
    pub items_by_pos:    std::collections::HashMap<String, i64>,
    pub items_by_cefr:   std::collections::HashMap<String, i64>,
}
