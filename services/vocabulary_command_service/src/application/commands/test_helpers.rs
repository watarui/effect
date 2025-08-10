#[cfg(test)]
pub mod mocks {
    use async_trait::async_trait;
    use mockall::mock;
    use uuid::Uuid;

    use crate::{
        domain::{DomainEvent, EntryId, ItemId, VocabularyEntry, VocabularyItem},
        error::Result,
        ports::{
            event_store::{AggregateSnapshot, EventStore},
            repositories::{VocabularyEntryRepository, VocabularyItemRepository},
        },
    };

    // VocabularyEntryRepository のモック
    mock! {
        pub EntryRepository {}

        #[async_trait]
        impl VocabularyEntryRepository for EntryRepository {
            async fn find_by_id(&self, entry_id: &EntryId) -> Result<Option<VocabularyEntry>>;
            async fn exists(&self, entry_id: &EntryId) -> Result<bool>;
            async fn save(&self, entry: &VocabularyEntry) -> Result<()>;
            async fn find_by_spelling(&self, spelling: &str) -> Result<Option<VocabularyEntry>>;
        }
    }

    // VocabularyItemRepository のモック
    mock! {
        pub ItemRepository {}

        #[async_trait]
        impl VocabularyItemRepository for ItemRepository {
            async fn find_by_id(&self, item_id: &ItemId) -> Result<Option<VocabularyItem>>;
            async fn save(&self, item: &VocabularyItem) -> Result<()>;
            async fn find_by_entry_id(&self, entry_id: &EntryId) -> Result<Vec<VocabularyItem>>;
            async fn find_primary_by_entry_id(&self, entry_id: &EntryId) -> Result<Option<VocabularyItem>>;
        }
    }

    // EventStore のモック
    mock! {
        pub EventStore {}

        #[async_trait]
        impl EventStore for EventStore {
            async fn append_event(&self, event: DomainEvent) -> Result<()>;
            async fn get_events_by_aggregate_id(&self, aggregate_id: Uuid) -> Result<Vec<DomainEvent>>;
            async fn get_events_since_version(&self, aggregate_id: Uuid, version: i64) -> Result<Vec<DomainEvent>>;
            async fn get_events_by_type(&self, event_type: &str, limit: Option<usize>) -> Result<Vec<DomainEvent>>;
            async fn get_events_in_range(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Result<Vec<DomainEvent>>;
            async fn get_latest_snapshot(&self, aggregate_id: Uuid) -> Result<Option<AggregateSnapshot>>;
            async fn save_snapshot(&self, snapshot: AggregateSnapshot) -> Result<()>;
        }
    }
}
