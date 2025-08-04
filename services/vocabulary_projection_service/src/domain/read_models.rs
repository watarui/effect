//! Read Models
//!
//! クエリ用の読み取り専用モデル

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 定義の構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionData {
    pub id:                  Uuid,
    pub part_of_speech:      String,
    pub meaning:             String,
    pub meaning_translation: Option<String>,
    pub domain:              Option<String>,
    pub register:            Option<String>,
    pub examples:            Vec<ExampleData>,
}

/// 例文の構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleData {
    pub id:                  Uuid,
    pub example_text:        String,
    pub example_translation: Option<String>,
}

/// コロケーションの構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollocationData {
    pub definition_id:    Uuid,
    pub collocation_type: String,
    pub pattern:          String,
    pub example:          Option<String>,
}

/// 語彙項目の Read Model（非正規化されたビュー）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VocabularyItemView {
    /// 基本情報
    pub item_id:        Uuid,
    pub entry_id:       Uuid,
    pub spelling:       String,
    pub disambiguation: String,

    /// 発音情報
    pub pronunciation:       Option<String>,
    pub phonetic_respelling: Option<String>,
    pub audio_url:           Option<String>,

    /// 分類情報
    pub register:   Option<String>,
    pub cefr_level: Option<String>,

    /// 集約データ（JSON形式）
    pub definitions:  sqlx::types::Json<Vec<DefinitionData>>,
    pub synonyms:     Option<sqlx::types::Json<std::collections::HashMap<String, Vec<String>>>>,
    pub antonyms:     Option<sqlx::types::Json<std::collections::HashMap<String, Vec<String>>>>,
    pub collocations: Option<sqlx::types::Json<Vec<CollocationData>>>,

    /// 統計情報
    pub definition_count: i32,
    pub example_count:    i32,
    pub quality_score:    Option<f32>,

    /// メタデータ
    pub status:           String,
    pub created_by_type:  String,
    pub created_by_id:    Option<Uuid>,
    pub created_at:       DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
    pub last_modified_by: Uuid,
    pub version:          i32,
}

/// プロジェクション状態
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectionState {
    pub projection_name:          String,
    pub last_processed_event_id:  Option<Uuid>,
    pub last_processed_timestamp: Option<DateTime<Utc>>,
    pub event_store_position:     Option<i64>,
    pub status:                   String,
    pub error_count:              i32,
    pub last_error:               Option<String>,
    pub updated_at:               DateTime<Utc>,
}
