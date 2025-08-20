//! イベントハンドラーの実装

use serde_json::Value as JsonValue;
use sqlx::{Postgres, Transaction};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{
    domain::{
        events::{EnrichedData, StoredEvent},
        projections::{
            VocabularyEntryProjection,
            VocabularyExampleProjection,
            VocabularyItemProjection,
        },
    },
    error::{ProjectionError, Result},
    ports::outbound::{ItemEnrichmentData, ReadModelRepository},
};

/// イベントハンドラー
pub struct EventHandler<R: ReadModelRepository> {
    repository: R,
}

impl<R: ReadModelRepository> EventHandler<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// イベントを処理
    pub async fn handle_event(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &StoredEvent,
    ) -> Result<()> {
        debug!(
            "Processing event: {} at position {}",
            event.event_type, event.position
        );

        match event.event_type.as_str() {
            "VocabularyEntryCreated" => self.handle_entry_created(tx, event).await,
            "VocabularyItemCreated" => self.handle_item_created(tx, event).await,
            "VocabularyItemPublished" => self.handle_item_published(tx, event).await,
            "VocabularyItemDeleted" => self.handle_item_deleted(tx, event).await,
            "ExampleAdded" => self.handle_example_added(tx, event).await,
            "AIEnrichmentCompleted" => self.handle_ai_enrichment(tx, event).await,
            "PrimaryItemSet" => self.handle_primary_item_set(tx, event).await,
            _ => {
                warn!("Unknown event type: {}", event.event_type);
                Ok(())
            },
        }
    }

    async fn handle_entry_created(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &StoredEvent,
    ) -> Result<()> {
        let data: JsonValue = serde_json::from_str(&event.event_data)?;

        let entry = VocabularyEntryProjection {
            entry_id:           event.aggregate_id,
            spelling:           data["spelling"].as_str().unwrap_or("").to_string(),
            primary_item_id:    None,
            item_count:         0,
            created_at:         event.occurred_at,
            updated_at:         event.occurred_at,
            last_event_version: event.aggregate_version,
        };

        self.repository.save_entry(tx, &entry).await
    }

    async fn handle_item_created(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &StoredEvent,
    ) -> Result<()> {
        let data: JsonValue = serde_json::from_str(&event.event_data)?;

        let item = VocabularyItemProjection {
            item_id:            data["item_id"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(Uuid::new_v4),
            entry_id:           data["entry_id"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(event.aggregate_id),
            spelling:           data["spelling"].as_str().unwrap_or("").to_string(),
            disambiguation:     data["disambiguation"].as_str().map(String::from),
            part_of_speech:     None,
            definition:         None,
            ipa_pronunciation:  None,
            cefr_level:         None,
            frequency_rank:     None,
            is_published:       false,
            is_deleted:         false,
            example_count:      0,
            created_at:         event.occurred_at,
            updated_at:         event.occurred_at,
            last_event_version: event.aggregate_version,
        };

        self.repository.save_item(tx, &item).await?;
        self.repository.update_item_count(tx, item.entry_id).await
    }

    async fn handle_item_published(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &StoredEvent,
    ) -> Result<()> {
        let data: JsonValue = serde_json::from_str(&event.event_data)?;
        let item_id = self.extract_uuid(&data, "item_id")?;

        self.repository
            .update_item_published(tx, item_id, true, event.aggregate_version)
            .await
    }

    async fn handle_item_deleted(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &StoredEvent,
    ) -> Result<()> {
        let data: JsonValue = serde_json::from_str(&event.event_data)?;
        let item_id = self.extract_uuid(&data, "item_id")?;

        self.repository
            .update_item_deleted(tx, item_id, true, event.aggregate_version)
            .await?;

        // Entry のアイテムカウントを更新
        if let Some(entry_id_str) = data["entry_id"].as_str()
            && let Ok(entry_id) = entry_id_str.parse::<Uuid>()
        {
            self.repository.update_item_count(tx, entry_id).await?;
        }

        Ok(())
    }

    async fn handle_example_added(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &StoredEvent,
    ) -> Result<()> {
        let data: JsonValue = serde_json::from_str(&event.event_data)?;
        let item_id = self.extract_uuid(&data, "item_id")?;

        let example = VocabularyExampleProjection {
            example_id: Uuid::new_v4(),
            item_id,
            example: data["example"].as_str().unwrap_or("").to_string(),
            translation: data["translation"].as_str().map(String::from),
            added_by: self
                .extract_uuid(&data, "added_by")
                .unwrap_or_else(|_| Uuid::new_v4()),
            created_at: event.occurred_at,
        };

        self.repository.add_example(tx, &example).await?;
        self.repository.increment_example_count(tx, item_id).await
    }

    async fn handle_ai_enrichment(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &StoredEvent,
    ) -> Result<()> {
        let data: JsonValue = serde_json::from_str(&event.event_data)?;
        let item_id = self.extract_uuid(&data, "item_id")?;

        let enriched: EnrichedData = serde_json::from_value(data["enriched_data"].clone())
            .map_err(ProjectionError::Serialization)?;
        let enrichment_data = ItemEnrichmentData {
            part_of_speech:    enriched.part_of_speech,
            definition:        enriched.definition,
            ipa_pronunciation: enriched.ipa_pronunciation,
            cefr_level:        enriched.cefr_level,
            frequency_rank:    enriched.frequency_rank,
        };

        self.repository
            .update_item_enrichment(tx, item_id, enrichment_data, event.aggregate_version)
            .await
    }

    async fn handle_primary_item_set(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &StoredEvent,
    ) -> Result<()> {
        let data: JsonValue = serde_json::from_str(&event.event_data)?;
        let entry_id = self.extract_uuid(&data, "entry_id")?;
        let item_id = self.extract_uuid(&data, "item_id")?;

        self.repository
            .update_entry_primary_item(tx, entry_id, Some(item_id), event.aggregate_version)
            .await
    }

    fn extract_uuid(&self, data: &JsonValue, field: &str) -> Result<Uuid> {
        data[field]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| {
                ProjectionError::EventProcessing(format!("Invalid {}: {:?}", field, data[field]))
            })
    }
}
