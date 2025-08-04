//! Read Model の定義
//!
//! クエリ用に最適化されたビュー

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 語彙項目ビュー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyItemView {
    pub item_id:          Uuid,
    pub entry_id:         Uuid,
    pub spelling:         String,
    pub disambiguation:   String,
    pub definition_count: i32,
    pub example_count:    i32,
    pub quality_score:    Option<f32>,
    pub status:           String,
    pub created_at:       DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
    pub version:          i64,
}

/// 語彙エントリービュー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyEntryView {
    pub entry_id:       Uuid,
    pub spelling:       String,
    pub part_of_speech: String,
    pub item_count:     i32,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

/// 統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyStats {
    pub total_entries:  i64,
    pub total_items:    i64,
    pub total_examples: i64,
    pub last_updated:   DateTime<Utc>,
}
