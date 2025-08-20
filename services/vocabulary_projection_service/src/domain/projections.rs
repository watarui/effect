//! プロジェクションのドメインモデル

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Vocabulary Entry の Read Model
#[derive(Debug, Clone)]
pub struct VocabularyEntryProjection {
    pub entry_id:           Uuid,
    pub spelling:           String,
    pub primary_item_id:    Option<Uuid>,
    pub item_count:         i32,
    pub created_at:         DateTime<Utc>,
    pub updated_at:         DateTime<Utc>,
    pub last_event_version: i64,
}

/// Vocabulary Item の Read Model
#[derive(Debug, Clone)]
pub struct VocabularyItemProjection {
    pub item_id:            Uuid,
    pub entry_id:           Uuid,
    pub spelling:           String,
    pub disambiguation:     Option<String>,
    pub part_of_speech:     Option<String>,
    pub definition:         Option<String>,
    pub ipa_pronunciation:  Option<String>,
    pub cefr_level:         Option<String>,
    pub frequency_rank:     Option<i32>,
    pub is_published:       bool,
    pub is_deleted:         bool,
    pub example_count:      i32,
    pub created_at:         DateTime<Utc>,
    pub updated_at:         DateTime<Utc>,
    pub last_event_version: i64,
}

/// 例文の Read Model
#[derive(Debug, Clone)]
pub struct VocabularyExampleProjection {
    pub example_id:  Uuid,
    pub item_id:     Uuid,
    pub example:     String,
    pub translation: Option<String>,
    pub added_by:    Uuid,
    pub created_at:  DateTime<Utc>,
}

/// プロジェクション状態
#[derive(Debug, Clone)]
pub struct ProjectionState {
    pub projection_name:         String,
    pub last_processed_position: i64,
    pub last_processed_event_id: Option<Uuid>,
    pub last_processed_at:       Option<DateTime<Utc>>,
    pub error_count:             i32,
    pub last_error:              Option<String>,
    pub last_error_at:           Option<DateTime<Utc>>,
}

impl ProjectionState {
    pub fn new(name: String) -> Self {
        Self {
            projection_name:         name,
            last_processed_position: 0,
            last_processed_event_id: None,
            last_processed_at:       None,
            error_count:             0,
            last_error:              None,
            last_error_at:           None,
        }
    }

    pub fn update_position(&mut self, position: i64, event_id: Option<Uuid>) {
        self.last_processed_position = position;
        self.last_processed_event_id = event_id;
        self.last_processed_at = Some(Utc::now());
        self.error_count = 0; // 成功時はエラーカウントをリセット
    }

    pub fn record_error(&mut self, error: String) {
        self.error_count += 1;
        self.last_error = Some(error);
        self.last_error_at = Some(Utc::now());
    }
}

/// チェックポイント
#[derive(Debug, Clone)]
pub struct ProjectionCheckpoint {
    pub projection_name:  String,
    pub position:         i64,
    pub event_id:         Option<Uuid>,
    pub events_processed: i32,
    pub created_at:       DateTime<Utc>,
}

impl ProjectionCheckpoint {
    pub fn new(
        projection_name: String,
        position: i64,
        event_id: Option<Uuid>,
        events_processed: i32,
    ) -> Self {
        Self {
            projection_name,
            position,
            event_id,
            events_processed,
            created_at: Utc::now(),
        }
    }
}
