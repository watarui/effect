use mockall::predicate::eq;
use uuid::Uuid;

use super::*;
use crate::{
    application::commands::test_helpers::mocks::{
        MockEntryRepository,
        MockEventStore,
        MockItemRepository,
    },
    domain::{CreateVocabularyItem, DomainEvent, EntryId},
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
    let mut mock_entry_repo = MockEntryRepository::new();
    let mock_item_repo = MockItemRepository::new();
    let mock_event_store = MockEventStore::new();

    let entry_id = Uuid::new_v4();
    let command = CreateVocabularyItem {
        entry_id,
        spelling: "".to_string(), // 空のスペリングは無効
        disambiguation: None,
    };

    // exists は呼ばれるが、その後のvalidationでエラーになる
    mock_entry_repo
        .expect_exists()
        .with(eq(EntryId::from_uuid(entry_id)))
        .times(1)
        .returning(|_| Ok(true));

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
