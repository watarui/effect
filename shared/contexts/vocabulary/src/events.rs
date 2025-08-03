//! Vocabulary Context のドメインイベント
//!
//! このモジュールは Vocabulary Context 内のドメインイベントを定義します。
//! Proto ファイルから生成されたコードを使用し、
//! 必要に応じて拡張トレイトを実装します。

use shared_kernel::{DomainEvent, EventMetadata, IntegrationEvent};

// Proto 生成コードを含める
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/effect.events.vocabulary.rs"));
}

// Proto 型を再エクスポート
pub use proto::*;

// DomainEvent トレイトの実装（Proto 生成型用）
impl DomainEvent for VocabularyEvent {
    fn event_type(&self) -> &str {
        match &self.event {
            Some(vocabulary_event::Event::EntryCreated(_)) => "VocabularyEntryCreated",
            Some(vocabulary_event::Event::ItemCreated(_)) => "VocabularyItemCreated",
            Some(vocabulary_event::Event::FieldUpdated(_)) => "VocabularyFieldUpdated",
            Some(vocabulary_event::Event::AiGenerationRequested(_)) => {
                "VocabularyAiGenerationRequested"
            },
            Some(vocabulary_event::Event::AiGenerationCompleted(_)) => {
                "VocabularyAiGenerationCompleted"
            },
            Some(vocabulary_event::Event::AiGenerationFailed(_)) => "VocabularyAiGenerationFailed",
            Some(vocabulary_event::Event::ItemPublished(_)) => "VocabularyItemPublished",
            Some(vocabulary_event::Event::UpdateConflicted(_)) => "VocabularyUpdateConflicted",
            None => "VocabularyEventUnknown",
        }
    }

    fn metadata(&self) -> &EventMetadata {
        // Proto の EventMetadata を shared_kernel の EventMetadata に変換する必要がある
        // 一時的にパニックを返す（後で実装）
        todo!("Convert proto EventMetadata to shared_kernel EventMetadata")
    }
}

/// 統合イベントの定義
/// 他のコンテキストに公開されるイベント
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum VocabularyIntegrationEvent {
    /// 語彙項目が公開された
    ItemPublished {
        event_id:              String,
        occurred_at:           chrono::DateTime<chrono::Utc>,
        item_id:               String,
        spelling:              String,
        part_of_speech:        String,
        cefr_level:            Option<String>,
        difficulty_estimate:   f32,
        content_quality_score: f32,
    },
    /// 語彙項目が更新された
    ItemUpdated {
        event_id:           String,
        occurred_at:        chrono::DateTime<chrono::Utc>,
        item_id:            String,
        updated_fields:     Vec<String>,
        difficulty_changed: bool,
        new_difficulty:     Option<f32>,
    },
    /// AI 生成が要求された
    AiGenerationRequested {
        event_id:    String,
        occurred_at: chrono::DateTime<chrono::Utc>,
        request_id:  String,
        item_id:     String,
        spelling:    String,
        priority:    String,
    },
}

impl IntegrationEvent for VocabularyIntegrationEvent {
    fn event_type(&self) -> &str {
        match self {
            Self::ItemPublished { .. } => "VocabularyItemPublished",
            Self::ItemUpdated { .. } => "VocabularyItemUpdated",
            Self::AiGenerationRequested { .. } => "VocabularyAiGenerationRequested",
        }
    }

    fn source_context(&self) -> &str {
        "vocabulary"
    }

    fn event_id(&self) -> &str {
        match self {
            Self::ItemPublished { event_id, .. }
            | Self::ItemUpdated { event_id, .. }
            | Self::AiGenerationRequested { event_id, .. } => event_id,
        }
    }

    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::ItemPublished { occurred_at, .. }
            | Self::ItemUpdated { occurred_at, .. }
            | Self::AiGenerationRequested { occurred_at, .. } => *occurred_at,
        }
    }
}
