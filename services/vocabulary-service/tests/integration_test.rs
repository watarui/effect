//! Integration tests for vocabulary service
//!
//! `PostgreSQL` リポジトリの統合テスト

use domain_events::CefrLevel;
use infrastructure::repository::Entity;
use vocabulary_service::domain::{
    entities::vocabulary_item::VocabularyItem,
    value_objects::{
        domain::Domain,
        part_of_speech::{AdjectiveType, NounType, PartOfSpeech},
        register::Register,
    },
};

#[tokio::test]
async fn test_vocabulary_item_creation() {
    // ドメインレベルでのテスト
    let item = VocabularyItem::new(
        "ephemeral",
        PartOfSpeech::Adjective(AdjectiveType::Both),
        Some(CefrLevel::C1),
        Register::Formal,
        Domain::General,
        vec!["lasting for a very short time", "temporary and fleeting"],
    )
    .expect("Failed to create vocabulary item");

    assert_eq!(item.word(), "ephemeral");
    assert_eq!(
        item.entry().primary_definition(),
        "lasting for a very short time"
    );
    assert_eq!(item.definitions().len(), 2); // primary + additional
    assert_eq!(item.version(), 1);
}

#[tokio::test]
async fn test_vocabulary_entry_validation() {
    // 空の単語でエラー
    let result = VocabularyItem::new(
        "",
        PartOfSpeech::Noun(NounType::Countable),
        None,
        Register::Neutral,
        Domain::General,
        vec!["some meaning"],
    );
    assert!(result.is_err());

    // 空の定義でエラー
    let result = VocabularyItem::new(
        "word",
        PartOfSpeech::Noun(NounType::Countable),
        None,
        Register::Neutral,
        Domain::General,
        vec![""],
    );
    assert!(result.is_err());
}

#[tokio::test]
async fn test_vocabulary_item_methods() {
    let mut item = VocabularyItem::new(
        "test",
        PartOfSpeech::Noun(NounType::Countable),
        Some(CefrLevel::B1),
        Register::Neutral,
        Domain::General,
        vec!["test definition"],
    )
    .unwrap();

    // 例文を追加
    item.add_example("This is a test sentence.", Some("これはテスト文です。"))
        .unwrap();

    assert_eq!(item.examples().len(), 1);

    // 類義語を追加
    item.add_synonym("sample");
    item.add_synonym("example");
    assert_eq!(item.synonyms().len(), 2);

    // 重複は追加されない
    item.add_synonym("sample");
    assert_eq!(item.synonyms().len(), 2);

    // メタデータを設定
    item.set_metadata("source", "dictionary");
}

// データベーステストは実際のDBが必要なため、環境変数でスキップ可能にする
#[tokio::test]
async fn test_repository_basic_operations() {
    let Ok(_) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("Skipping database test: TEST_DATABASE_URL not set");
        return;
    };

    // 実際のデータベースが利用可能な場合のみテストを実行
    // このテストは完全なリポジトリ実装が必要
    eprintln!("Repository implementation not yet complete, skipping test");
}
