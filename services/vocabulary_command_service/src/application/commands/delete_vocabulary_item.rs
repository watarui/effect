use crate::{
    domain::{DeleteVocabularyItem, DomainEvent, EventMetadata, ItemId, VocabularyItemDeleted},
    error::Result,
    ports::{
        event_store::EventStore,
        repositories::{VocabularyEntryRepository, VocabularyItemRepository},
    },
};

/// DeleteVocabularyItem コマンドハンドラー
pub struct DeleteVocabularyItemHandler<ER, IR, ES>
where
    ER: VocabularyEntryRepository,
    IR: VocabularyItemRepository,
    ES: EventStore,
{
    _entry_repository: ER,
    item_repository:   IR,
    event_store:       ES,
}

impl<ER, IR, ES> DeleteVocabularyItemHandler<ER, IR, ES>
where
    ER: VocabularyEntryRepository,
    IR: VocabularyItemRepository,
    ES: EventStore,
{
    pub fn new(entry_repository: ER, item_repository: IR, event_store: ES) -> Self {
        Self {
            _entry_repository: entry_repository,
            item_repository,
            event_store,
        }
    }

    pub async fn handle(&self, command: DeleteVocabularyItem) -> Result<()> {
        // アイテムの存在確認
        let item_id = ItemId::from_uuid(command.item_id);
        let item = self
            .item_repository
            .find_by_id(&item_id)
            .await?
            .ok_or_else(|| {
                crate::error::Error::NotFound(format!("Item not found: {}", command.item_id))
            })?;

        // すでに削除済みのチェック
        if item.is_deleted {
            return Err(crate::error::Error::Conflict(
                "Item is already deleted".to_string(),
            ));
        }

        // 削除マーク
        let mut updated_item = item.clone();
        updated_item.mark_as_deleted()?;

        // アイテムを保存
        self.item_repository.save(&updated_item).await?;

        // イベントを発行
        let event = DomainEvent::VocabularyItemDeleted(VocabularyItemDeleted {
            metadata:   EventMetadata::new(command.item_id, updated_item.version.value()),
            item_id:    command.item_id,
            deleted_by: command.deleted_by,
        });
        self.event_store.append_event(event).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use uuid::Uuid;

    use super::*;
    use crate::{
        application::commands::test_helpers::mocks::{
            MockEntryRepository,
            MockEventStore,
            MockItemRepository,
        },
        domain::{Disambiguation, EntryId, ItemId, Spelling, VocabularyItem},
        error::Error,
    };

    #[tokio::test]
    async fn test_delete_existing_item() {
        // Arrange
        let item_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let mut item_repository = MockItemRepository::new();
        let entry_repository = MockEntryRepository::new();
        let mut event_store = MockEventStore::new();

        // 既存のアイテムを設定
        let existing_item = VocabularyItem::create(
            EntryId::from_uuid(entry_id),
            Spelling::new("test".to_string()).unwrap(),
            Disambiguation::new(Some("test meaning".to_string())).unwrap(),
        );

        item_repository
            .expect_find_by_id()
            .with(eq(ItemId::from_uuid(item_id)))
            .times(1)
            .returning(move |_| Ok(Some(existing_item.clone())));

        item_repository.expect_save().times(1).returning(|_| Ok(()));

        // イベント保存の期待値を設定
        event_store
            .expect_append_event()
            .times(1)
            .returning(|_| Ok(()));

        let handler =
            DeleteVocabularyItemHandler::new(entry_repository, item_repository, event_store);

        let command = DeleteVocabularyItem {
            item_id,
            deleted_by: user_id,
        };

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_non_existent_item() {
        // Arrange
        let item_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let mut item_repository = MockItemRepository::new();
        let entry_repository = MockEntryRepository::new();
        let event_store = MockEventStore::new();

        item_repository
            .expect_find_by_id()
            .with(eq(ItemId::from_uuid(item_id)))
            .times(1)
            .returning(|_| Ok(None));

        let handler =
            DeleteVocabularyItemHandler::new(entry_repository, item_repository, event_store);

        let command = DeleteVocabularyItem {
            item_id,
            deleted_by: user_id,
        };

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::NotFound(msg) => {
                assert!(msg.contains("Item not found"));
            },
            _ => panic!("Expected NotFound error"),
        }
    }
}
