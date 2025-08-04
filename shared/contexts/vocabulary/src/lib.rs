//! Vocabulary Context 共有ライブラリ
//!
//! Vocabulary Context 内の各マイクロサービス間で共有される
//! ドメインモデル、イベント、コマンド、クエリを定義

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod commands;
pub mod domain;
pub mod events;
pub mod queries;

// Re-export commonly used items
pub use commands::*;
pub use domain::{VocabularyEntry, VocabularyItem};
pub use events::*;
pub use queries::*;

/// イベントメタデータを作成するヘルパー関数
pub fn create_event_metadata(
    aggregate_id: Uuid,
    _aggregate_type: &str,
    occurred_at: DateTime<Utc>,
) -> shared_kernel::proto::effect::common::EventMetadata {
    shared_kernel::proto::effect::common::EventMetadata {
        event_id:          Uuid::new_v4().to_string(),
        aggregate_id:      aggregate_id.to_string(),
        occurred_at:       Some(prost_types::Timestamp {
            seconds: occurred_at.timestamp(),
            nanos:   occurred_at.timestamp_subsec_nanos() as i32,
        }),
        version:           0,
        caused_by_user_id: None,
        correlation_id:    None,
        causation_id:      None,
        trace_context:     None,
        command_id:        None,
        source:            Some("vocabulary_command_service".to_string()),
        schema_version:    Some(1),
    }
}
