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
        // 値オブジェクトの生成
        let spelling =
            Spelling::new(command.spelling.clone()).map_err(crate::error::Error::Validation)?;
        let disambiguation = Disambiguation::new(command.disambiguation.clone())
            .map_err(crate::error::Error::Validation)?;

        // エントリの取得または作成
        use crate::domain::VocabularyEntry;

        let entry = if command.entry_id == uuid::Uuid::nil() {
            // entry_id が nil の場合、spelling で検索または新規作成
            if let Some(existing) = self.entry_repository.find_by_spelling(&spelling).await? {
                existing
            } else {
                // 新規エントリーを作成
                let new_entry = VocabularyEntry::create(spelling.clone());
                self.entry_repository.save(&new_entry).await?;
                new_entry
            }
        } else {
            // entry_id が指定されている場合、存在確認
            let entry_id = EntryId::from_uuid(command.entry_id);
            if !self.entry_repository.exists(&entry_id).await? {
                return Err(crate::error::Error::NotFound(format!(
                    "Entry not found: {}",
                    command.entry_id
                )));
            }
            // エントリーを取得
            self.entry_repository
                .find_by_id(&entry_id)
                .await?
                .ok_or_else(|| {
                    crate::error::Error::NotFound(format!("Entry not found: {}", command.entry_id))
                })?
        };

        // 集約の生成
        let item = VocabularyItem::create(entry.entry_id, spelling, disambiguation);

        // リポジトリに保存
        self.item_repository.save(&item).await?;

        // イベントの生成と保存
        let event = DomainEvent::VocabularyItemCreated(VocabularyItemCreated {
            metadata:       EventMetadata::new(*item.item_id.as_uuid(), item.version.value()),
            item_id:        *item.item_id.as_uuid(),
            entry_id:       *entry.entry_id.as_uuid(),
            spelling:       command.spelling,
            disambiguation: command.disambiguation,
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
        application::commands::test_helpers::mocks::{
            MockEntryRepository,
            MockEventStore,
            MockItemRepository,
        },
        domain::{DomainEvent, EntryId, Spelling, VocabularyEntry},
    };

    #[tokio::test]
    async fn test_create_vocabulary_item_success() {
        // Arrange
        let mut mock_entry_repo = MockEntryRepository::new();
        let mut mock_item_repo = MockItemRepository::new();
        let mut mock_event_store = MockEventStore::new();

        let entry_id = Uuid::new_v4();
        let command = CreateVocabularyItem {
            entry_id,
            spelling: "apple".to_string(),
            disambiguation: Some("fruit".to_string()),
        };

        // リポジトリのモック設定
        mock_entry_repo
            .expect_exists()
            .with(eq(EntryId::from_uuid(entry_id)))
            .times(1)
            .returning(|_| Ok(true));

        let mut entry = VocabularyEntry::create(Spelling::new("apple".to_string()).unwrap());
        entry.entry_id = EntryId::from_uuid(entry_id); // entry_idを正しく設定
        let entry_for_mock = entry.clone();
        mock_entry_repo
            .expect_find_by_id()
            .with(eq(EntryId::from_uuid(entry_id)))
            .times(1)
            .returning(move |_| Ok(Some(entry_for_mock.clone())));

        mock_item_repo.expect_save().times(1).returning(|_| Ok(()));

        // イベントストアのモック設定
        mock_event_store
            .expect_append_event()
            .times(1)
            .returning(|_| Ok(()));

        let handler =
            CreateVocabularyItemHandler::new(mock_entry_repo, mock_item_repo, mock_event_store);

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.spelling.as_str(), "apple");
        assert_eq!(item.disambiguation.as_option(), Some("fruit"));
        assert_eq!(item.entry_id, EntryId::from_uuid(entry_id));
    }

    #[tokio::test]
    async fn test_create_vocabulary_item_entry_not_found() {
        // Arrange
        let mut mock_entry_repo = MockEntryRepository::new();
        let mock_item_repo = MockItemRepository::new();
        let mock_event_store = MockEventStore::new();

        let entry_id = Uuid::new_v4();
        let command = CreateVocabularyItem {
            entry_id,
            spelling: "apple".to_string(),
            disambiguation: None,
        };

        // エントリが見つからない
        mock_entry_repo
            .expect_exists()
            .with(eq(EntryId::from_uuid(entry_id)))
            .times(1)
            .returning(|_| Ok(false));

        let handler =
            CreateVocabularyItemHandler::new(mock_entry_repo, mock_item_repo, mock_event_store);

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::Error::NotFound(msg) => {
                assert!(msg.contains("Entry not found"));
            },
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_create_vocabulary_item_invalid_spelling() {
        // Arrange
        let mock_entry_repo = MockEntryRepository::new();
        let mock_item_repo = MockItemRepository::new();
        let mock_event_store = MockEventStore::new();

        let entry_id = Uuid::new_v4();
        let command = CreateVocabularyItem {
            entry_id,
            spelling: "".to_string(), // 空のスペリングは無効
            disambiguation: None,
        };

        // スペリングバリデーションで失敗するため、exists は呼ばれない

        let handler =
            CreateVocabularyItemHandler::new(mock_entry_repo, mock_item_repo, mock_event_store);

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::Error::Validation(msg) => {
                assert!(msg.contains("cannot be empty"));
            },
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_vocabulary_item_with_empty_disambiguation() {
        // Arrange
        let mut mock_entry_repo = MockEntryRepository::new();
        let mut mock_item_repo = MockItemRepository::new();
        let mut mock_event_store = MockEventStore::new();

        let entry_id = Uuid::new_v4();
        let command = CreateVocabularyItem {
            entry_id,
            spelling: "run".to_string(),
            disambiguation: Some("  ".to_string()), // 空白のみは None として扱われる
        };

        mock_entry_repo
            .expect_exists()
            .with(eq(EntryId::from_uuid(entry_id)))
            .times(1)
            .returning(|_| Ok(true));

        let mut entry = VocabularyEntry::create(Spelling::new("run".to_string()).unwrap());
        entry.entry_id = EntryId::from_uuid(entry_id);
        let entry_for_mock = entry.clone();
        mock_entry_repo
            .expect_find_by_id()
            .with(eq(EntryId::from_uuid(entry_id)))
            .times(1)
            .returning(move |_| Ok(Some(entry_for_mock.clone())));

        mock_item_repo.expect_save().times(1).returning(|item| {
            // 空白のみの disambiguation は None になることを確認
            assert!(item.disambiguation.is_none());
            Ok(())
        });

        mock_event_store
            .expect_append_event()
            .times(1)
            .returning(|event| {
                // イベントでは元の値（空白文字列）が保存されることに注意
                if let DomainEvent::VocabularyItemCreated(e) = event {
                    // コマンドからイベントに渡される値は変更されない
                    assert_eq!(e.disambiguation, Some("  ".to_string()));
                }
                Ok(())
            });

        let handler =
            CreateVocabularyItemHandler::new(mock_entry_repo, mock_item_repo, mock_event_store);

        // Act
        let result = handler.handle(command).await;

        // Assert
        assert!(result.is_ok());
        let item = result.unwrap();
        assert!(item.disambiguation.is_none());
    }
}
