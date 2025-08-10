use crate::{
    domain::{
        CreateVocabularyItem,
        Disambiguation,
        DomainEvent,
        EntryId,
        EventMetadata,
        Spelling,
        VocabularyItem,
        VocabularyItemCreated,
    },
    error::Result,
    ports::{
        event_store::EventStore,
        repositories::{VocabularyEntryRepository, VocabularyItemRepository},
    },
};

/// CreateVocabularyItem コマンドハンドラー
pub struct CreateVocabularyItemHandler<ER, IR, ES>
where
    ER: VocabularyEntryRepository,
    IR: VocabularyItemRepository,
    ES: EventStore,
{
    entry_repository: ER,
    item_repository:  IR,
    event_store:      ES,
}

impl<ER, IR, ES> CreateVocabularyItemHandler<ER, IR, ES>
where
    ER: VocabularyEntryRepository,
    IR: VocabularyItemRepository,
    ES: EventStore,
{
    pub fn new(entry_repository: ER, item_repository: IR, event_store: ES) -> Self {
        Self {
            entry_repository,
            item_repository,
            event_store,
        }
    }

    pub async fn handle(&self, command: CreateVocabularyItem) -> Result<VocabularyItem> {
        // エントリの存在確認
        let entry_id = EntryId::from_uuid(command.entry_id);
        if !self.entry_repository.exists(&entry_id).await? {
            return Err(crate::error::Error::NotFound(format!(
                "Entry not found: {}",
                command.entry_id
            )));
        }

        // 値オブジェクトの生成
        let spelling =
            Spelling::new(command.spelling.clone()).map_err(crate::error::Error::Validation)?;
        let disambiguation = Disambiguation::new(command.disambiguation.clone())
            .map_err(crate::error::Error::Validation)?;

        // 集約の生成
        let item = VocabularyItem::create(entry_id, spelling, disambiguation);

        // リポジトリに保存
        self.item_repository.save(&item).await?;

        // イベントの生成と保存
        let event = DomainEvent::VocabularyItemCreated(VocabularyItemCreated {
            metadata:       EventMetadata::new(*item.item_id.as_uuid(), item.version.value()),
            item_id:        *item.item_id.as_uuid(),
            entry_id:       *item.entry_id.as_uuid(),
            spelling:       command.spelling,
            disambiguation: command.disambiguation,
        });
        self.event_store.append_event(event).await?;

        Ok(item)
    }
}

#[cfg(test)]
#[path = "create_vocabulary_item_test.rs"]
mod tests;
