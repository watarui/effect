//! ドメインイベントの定義

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event Store から取得したイベント
#[derive(Debug, Clone)]
pub struct StoredEvent {
    pub position:          i64,
    pub event_id:          Uuid,
    pub aggregate_id:      Uuid,
    pub aggregate_version: i64,
    pub event_type:        String,
    pub event_data:        String,
    pub occurred_at:       DateTime<Utc>,
}

/// イベントのメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id:     Uuid,
    pub aggregate_id: Uuid,
    pub occurred_at:  DateTime<Utc>,
    pub version:      i64,
}

/// AI エンリッチメントデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedData {
    pub part_of_speech:    Option<String>,
    pub definition:        Option<String>,
    pub ipa_pronunciation: Option<String>,
    pub cefr_level:        Option<String>,
    pub frequency_rank:    Option<i32>,
}
