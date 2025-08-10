use crate::{
    domain::{
        Disambiguation,
        DomainEvent,
        EventMetadata,
        ItemId,
        UpdateVocabularyItem,
        VocabularyItem,
        VocabularyItemDisambiguationUpdated,
    },
    error::Result,
    ports::{event_store::EventStore, repositories::VocabularyItemRepository},
};

/// UpdateVocabularyItem コマンドハンドラー
pub struct UpdateVocabularyItemHandler<R, E>
where
    R: VocabularyItemRepository,
    E: EventStore,
{
    repository:  R,
    event_store: E,
}

impl<R, E> UpdateVocabularyItemHandler<R, E>
where
    R: VocabularyItemRepository,
    E: EventStore,
{
    pub fn new(repository: R, event_store: E) -> Self {
        Self {
            repository,
            event_store,
        }
    }

    pub async fn handle(&self, command: UpdateVocabularyItem) -> Result<VocabularyItem> {
        // アイテムの取得
        let item_id = ItemId::from_uuid(command.item_id);
        let mut item = self.repository.find_by_id(&item_id).await?.ok_or_else(|| {
            crate::error::Error::NotFound(format!("Item not found: {}", command.item_id))
        })?;

        // バージョンチェック（楽観的ロック）
        if item.version.value() != command.version {
            return Err(crate::error::Error::Conflict(format!(
                "Version mismatch. Current: {}, Expected: {}",
                item.version.value(),
                command.version
            )));
        }

        // 値オブジェクトの生成
        let new_disambiguation = Disambiguation::new(command.disambiguation.clone())
            .map_err(crate::error::Error::Validation)?;

        // 古い値を保存
        let old_disambiguation = item.disambiguation.as_option().map(|s| s.to_string());

        // 集約の更新
        item.update_disambiguation(new_disambiguation)?;

        // リポジトリに保存
        self.repository.save(&item).await?;

        // イベントの生成と保存
        let event =
            DomainEvent::VocabularyItemDisambiguationUpdated(VocabularyItemDisambiguationUpdated {
                metadata: EventMetadata::new(*item.item_id.as_uuid(), item.version.value()),
                item_id: *item.item_id.as_uuid(),
                old_disambiguation,
                new_disambiguation: command.disambiguation,
            });
        self.event_store.append_event(event).await?;

        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use uuid::Uuid;

    use super::*;
    use crate::{
        application::commands::test_helpers::mocks::{MockEventStore, MockItemRepository},
        domain::{EntryId, Spelling},
    };

    fn create_test_item() -> VocabularyItem {
        let entry_id = EntryId::new();
        let spelling = Spelling::new("test".to_string()).unwrap();
        let disambiguation = Disambiguation::new(Some("original".to_string())).unwrap();
        VocabularyItem::create(entry_id, spelling, disambiguation)
    }

    #[tokio::test]
    async fn test_update_vocabulary_item_success() {
        // Arrange
        let mut mock_repo = MockItemRepository::new();
        let mut mock_event_store = MockEventStore::new();

        let item = create_test_item();
        let item_id = *item.item_id.as_uuid();
        let version = item.version.value();
        let item_clone = item.clone();

        let command = UpdateVocabularyItem {
            item_id,
            disambiguation: Some("updated".to_string()),
            version,
        };

        mock_repo
            .expect_find_by_id()
            .with(eq(ItemId::from_uuid(item_id)))
            .times(1)
            .returning(move |_| Ok(Some(item_clone.clone())));

        mock_repo.expect_save().times(1).returning(|item| {
            assert_eq!(item.disambiguation.as_option(), Some("updated"));
            assert_eq!(item.version.value(), 2); // バージョンがインクリメントされている
            Ok(())
        });

        mock_event_store
            .expect_append_event()
            .times(1)
            .returning(|event| {
                if let DomainEvent::VocabularyItemDisambiguationUpdated(e) = event {
                    assert_eq!(e.old_disambiguation, Some("original".to_string()));
                    assert_eq!(e.new_disambiguation, Some("updated".to_string()));
                }
                Ok(())
            });

        let handler = UpdateVocabularyItemHandler::new(mock_repo, mock_event_store);

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_ok());
        let updated_item = result.unwrap();
        assert_eq!(updated_item.disambiguation.as_option(), Some("updated"));
        assert_eq!(updated_item.version.value(), 2);
    }

    #[tokio::test]
    async fn test_update_vocabulary_item_not_found() {
        // Arrange
        let mut mock_repo = MockItemRepository::new();
        let mock_event_store = MockEventStore::new();

        let item_id = Uuid::new_v4();
        let command = UpdateVocabularyItem {
            item_id,
            disambiguation: Some("updated".to_string()),
            version: 1,
        };

        mock_repo
            .expect_find_by_id()
            .with(eq(ItemId::from_uuid(item_id)))
            .times(1)
            .returning(|_| Ok(None));

        let handler = UpdateVocabularyItemHandler::new(mock_repo, mock_event_store);

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::Error::NotFound(msg) => {
                assert!(msg.contains("Item not found"));
            },
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_update_vocabulary_item_version_conflict() {
        // Arrange
        let mut mock_repo = MockItemRepository::new();
        let mock_event_store = MockEventStore::new();

        let item = create_test_item();
        let item_id = *item.item_id.as_uuid();
        let item_clone = item.clone();

        let command = UpdateVocabularyItem {
            item_id,
            disambiguation: Some("updated".to_string()),
            version: 2, // 間違ったバージョン（実際は1）
        };

        mock_repo
            .expect_find_by_id()
            .with(eq(ItemId::from_uuid(item_id)))
            .times(1)
            .returning(move |_| Ok(Some(item_clone.clone())));

        let handler = UpdateVocabularyItemHandler::new(mock_repo, mock_event_store);

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::Error::Conflict(msg) => {
                assert!(msg.contains("Version mismatch"));
            },
            _ => panic!("Expected Conflict error"),
        }
    }

    #[tokio::test]
    async fn test_update_published_item_fails() {
        // Arrange
        let mut mock_repo = MockItemRepository::new();
        let mock_event_store = MockEventStore::new();

        let mut item = create_test_item();
        item.publish().unwrap(); // 公開済みにする
        let item_id = *item.item_id.as_uuid();
        let version = item.version.value();
        let item_clone = item.clone();

        let command = UpdateVocabularyItem {
            item_id,
            disambiguation: Some("updated".to_string()),
            version,
        };

        mock_repo
            .expect_find_by_id()
            .with(eq(ItemId::from_uuid(item_id)))
            .times(1)
            .returning(move |_| Ok(Some(item_clone.clone())));

        let handler = UpdateVocabularyItemHandler::new(mock_repo, mock_event_store);

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::Error::Domain(msg) => {
                assert!(msg.contains("Cannot update disambiguation for published items"));
            },
            _ => panic!("Expected Domain error"),
        }
    }

    #[tokio::test]
    async fn test_clear_disambiguation() {
        // Arrange
        let mut mock_repo = MockItemRepository::new();
        let mut mock_event_store = MockEventStore::new();

        let item = create_test_item();
        let item_id = *item.item_id.as_uuid();
        let version = item.version.value();
        let item_clone = item.clone();

        let command = UpdateVocabularyItem {
            item_id,
            disambiguation: None, // クリア
            version,
        };

        mock_repo
            .expect_find_by_id()
            .with(eq(ItemId::from_uuid(item_id)))
            .times(1)
            .returning(move |_| Ok(Some(item_clone.clone())));

        mock_repo.expect_save().times(1).returning(|item| {
            assert!(item.disambiguation.is_none());
            Ok(())
        });

        mock_event_store
            .expect_append_event()
            .times(1)
            .returning(|event| {
                if let DomainEvent::VocabularyItemDisambiguationUpdated(e) = event {
                    assert_eq!(e.old_disambiguation, Some("original".to_string()));
                    assert_eq!(e.new_disambiguation, None);
                }
                Ok(())
            });

        let handler = UpdateVocabularyItemHandler::new(mock_repo, mock_event_store);

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_ok());
        let updated_item = result.unwrap();
        assert!(updated_item.disambiguation.is_none());
    }
}
