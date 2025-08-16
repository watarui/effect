use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::commands::EnrichedData;

/// イベントの基本メタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id:     Uuid,
    pub aggregate_id: Uuid,
    pub occurred_at:  DateTime<Utc>,
    pub version:      i64,
}

impl EventMetadata {
    pub fn new(aggregate_id: Uuid, version: i64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            aggregate_id,
            occurred_at: Utc::now(),
            version,
        }
    }
}

/// VocabularyEntry が作成された
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyEntryCreated {
    pub metadata: EventMetadata,
    pub entry_id: Uuid,
    pub spelling: String,
}

/// VocabularyEntry のスペリングが更新された
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyEntrySpellingUpdated {
    pub metadata:     EventMetadata,
    pub entry_id:     Uuid,
    pub old_spelling: String,
    pub new_spelling: String,
}

/// VocabularyItem が作成された
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyItemCreated {
    pub metadata:       EventMetadata,
    pub item_id:        Uuid,
    pub entry_id:       Uuid,
    pub spelling:       String,
    pub disambiguation: Option<String>,
}

/// VocabularyItem の曖昧性解消が更新された
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyItemDisambiguationUpdated {
    pub metadata:           EventMetadata,
    pub item_id:            Uuid,
    pub old_disambiguation: Option<String>,
    pub new_disambiguation: Option<String>,
}

/// VocabularyItem が公開された
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyItemPublished {
    pub metadata: EventMetadata,
    pub item_id:  Uuid,
    pub entry_id: Uuid,
}

/// AI エンリッチメントがリクエストされた
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEnrichmentRequested {
    pub metadata:       EventMetadata,
    pub item_id:        Uuid,
    pub entry_id:       Uuid,
    pub spelling:       String,
    pub disambiguation: Option<String>,
}

/// AI エンリッチメントが完了した
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEnrichmentCompleted {
    pub metadata:      EventMetadata,
    pub item_id:       Uuid,
    pub enriched_data: EnrichedData,
}

/// 主要項目として設定された
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryItemSet {
    pub metadata:                 EventMetadata,
    pub entry_id:                 Uuid,
    pub item_id:                  Uuid,
    pub previous_primary_item_id: Option<Uuid>,
}

/// 主要項目から解除された
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryItemUnset {
    pub metadata: EventMetadata,
    pub entry_id: Uuid,
    pub item_id:  Uuid,
}

/// VocabularyItem が削除された
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyItemDeleted {
    pub metadata:   EventMetadata,
    pub item_id:    Uuid,
    pub deleted_by: Uuid,
}

/// VocabularyItem に例文が追加された
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleAdded {
    pub metadata:    EventMetadata,
    pub item_id:     Uuid,
    pub example:     String,
    pub translation: Option<String>,
    pub added_by:    Uuid,
}

/// すべてのドメインイベントをまとめる列挙型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    VocabularyEntryCreated(VocabularyEntryCreated),
    VocabularyEntrySpellingUpdated(VocabularyEntrySpellingUpdated),
    VocabularyItemCreated(VocabularyItemCreated),
    VocabularyItemDisambiguationUpdated(VocabularyItemDisambiguationUpdated),
    VocabularyItemPublished(VocabularyItemPublished),
    VocabularyItemDeleted(VocabularyItemDeleted),
    ExampleAdded(ExampleAdded),
    AIEnrichmentRequested(AIEnrichmentRequested),
    AIEnrichmentCompleted(AIEnrichmentCompleted),
    PrimaryItemSet(PrimaryItemSet),
    PrimaryItemUnset(PrimaryItemUnset),
}

impl DomainEvent {
    /// イベントのメタデータを取得
    pub fn metadata(&self) -> &EventMetadata {
        match self {
            DomainEvent::VocabularyEntryCreated(e) => &e.metadata,
            DomainEvent::VocabularyEntrySpellingUpdated(e) => &e.metadata,
            DomainEvent::VocabularyItemCreated(e) => &e.metadata,
            DomainEvent::VocabularyItemDisambiguationUpdated(e) => &e.metadata,
            DomainEvent::VocabularyItemPublished(e) => &e.metadata,
            DomainEvent::VocabularyItemDeleted(e) => &e.metadata,
            DomainEvent::ExampleAdded(e) => &e.metadata,
            DomainEvent::AIEnrichmentRequested(e) => &e.metadata,
            DomainEvent::AIEnrichmentCompleted(e) => &e.metadata,
            DomainEvent::PrimaryItemSet(e) => &e.metadata,
            DomainEvent::PrimaryItemUnset(e) => &e.metadata,
        }
    }

    /// イベントタイプを文字列で取得
    pub fn event_type(&self) -> &str {
        match self {
            DomainEvent::VocabularyEntryCreated(_) => "VocabularyEntryCreated",
            DomainEvent::VocabularyEntrySpellingUpdated(_) => "VocabularyEntrySpellingUpdated",
            DomainEvent::VocabularyItemCreated(_) => "VocabularyItemCreated",
            DomainEvent::VocabularyItemDisambiguationUpdated(_) => {
                "VocabularyItemDisambiguationUpdated"
            },
            DomainEvent::VocabularyItemPublished(_) => "VocabularyItemPublished",
            DomainEvent::VocabularyItemDeleted(_) => "VocabularyItemDeleted",
            DomainEvent::ExampleAdded(_) => "ExampleAdded",
            DomainEvent::AIEnrichmentRequested(_) => "AIEnrichmentRequested",
            DomainEvent::AIEnrichmentCompleted(_) => "AIEnrichmentCompleted",
            DomainEvent::PrimaryItemSet(_) => "PrimaryItemSet",
            DomainEvent::PrimaryItemUnset(_) => "PrimaryItemUnset",
        }
    }
}
