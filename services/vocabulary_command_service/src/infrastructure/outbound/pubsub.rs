//! Pub/Sub 実装

use async_trait::async_trait;
use shared_error::DomainResult;
use shared_vocabulary_context::events::{
    EntryCreated,
    FieldUpdated,
    ItemCreated,
    ItemPublished,
    VocabularyEvent,
    vocabulary_event,
};

use crate::{domain::events::VocabularyDomainEvent, ports::outbound::EventBus};

/// Google Pub/Sub 実装（モック）
pub struct PubSubEventBus {
    // TODO: Pub/Sub クライアントを追加
    // client: google_cloud_pubsub::client::Client,
    topic_name: String,
}

impl Default for PubSubEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl PubSubEventBus {
    /// 新しい EventBus を作成
    pub fn new() -> Self {
        Self {
            topic_name: "vocabulary-events".to_string(),
        }
    }

    /// ドメインイベントを Proto メッセージに変換
    fn to_proto_event(&self, event: &VocabularyDomainEvent) -> DomainResult<VocabularyEvent> {
        match event {
            VocabularyDomainEvent::EntryCreated {
                entry_id,
                word,
                occurred_at,
            } => {
                let metadata = shared_vocabulary_context::create_event_metadata(
                    *entry_id,
                    "VocabularyEntry",
                    *occurred_at,
                );
                Ok(VocabularyEvent {
                    event: Some(vocabulary_event::Event::EntryCreated(EntryCreated {
                        metadata: Some(metadata),
                        entry_id: entry_id.to_string(),
                        spelling: word.clone(),
                    })),
                })
            },
            VocabularyDomainEvent::ItemCreated {
                item_id,
                entry_id,
                word,
                definitions: _,
                part_of_speech,
                register,
                domain,
                created_by,
                occurred_at,
            } => {
                let metadata = shared_vocabulary_context::create_event_metadata(
                    *item_id,
                    "VocabularyItem",
                    *occurred_at,
                );
                // 簡易的な disambiguation 生成
                let disambiguation = format!("{part_of_speech} ({register}, {domain})");
                Ok(VocabularyEvent {
                    event: Some(vocabulary_event::Event::ItemCreated(ItemCreated {
                        metadata: Some(metadata),
                        item_id: item_id.to_string(),
                        entry_id: entry_id.to_string(),
                        spelling: word.clone(),
                        disambiguation,
                        created_by: created_by.to_string(),
                    })),
                })
            },
            VocabularyDomainEvent::ItemUpdated {
                item_id,
                field_name,
                old_value,
                new_value,
                updated_by,
                occurred_at,
            } => {
                let metadata = shared_vocabulary_context::create_event_metadata(
                    *item_id,
                    "VocabularyItem",
                    *occurred_at,
                );
                Ok(VocabularyEvent {
                    event: Some(vocabulary_event::Event::FieldUpdated(FieldUpdated {
                        metadata:       Some(metadata),
                        item_id:        item_id.to_string(),
                        field_path:     field_name.clone(),
                        old_value_json: old_value.to_string(),
                        new_value_json: new_value.to_string(),
                        updated_by:     updated_by.to_string(),
                        version:        0, // TODO: バージョン管理の実装
                    })),
                })
            },
            VocabularyDomainEvent::ItemPublished {
                item_id,
                published_by: _,
                occurred_at,
            } => {
                let metadata = shared_vocabulary_context::create_event_metadata(
                    *item_id,
                    "VocabularyItem",
                    *occurred_at,
                );
                Ok(VocabularyEvent {
                    event: Some(vocabulary_event::Event::ItemPublished(ItemPublished {
                        metadata: Some(metadata),
                        item_id:  item_id.to_string(),
                    })),
                })
            },
        }
    }
}

#[async_trait]
impl EventBus for PubSubEventBus {
    async fn publish(&self, events: Vec<VocabularyDomainEvent>) -> DomainResult<()> {
        for event in events {
            // イベントを Proto メッセージに変換
            let proto_event = self.to_proto_event(&event)?;

            // TODO: 実際の Pub/Sub への発行
            // 現在はログ出力のみ
            tracing::info!(
                topic = %self.topic_name,
                event_type = %event.event_type(),
                "Publishing event to Pub/Sub (mock)"
            );
            tracing::debug!("Event details: {:?}", proto_event);
        }
        Ok(())
    }
}
