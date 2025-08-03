//! 語彙項目エンティティ
//!
//! 完全な語彙項目（集約ルート）

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common_types::ItemId;
use domain_events::CefrLevel;
use serde::{Deserialize, Serialize};
use shared_repository::{Entity, EntitySoftDeletable as SoftDeletable};
use thiserror::Error;

use crate::domain::{
    entities::vocabulary_entry::{EntryError, VocabularyEntry},
    value_objects::{
        definition::{Collocation, Definition, Error as DefinitionError, Example},
        domain::Domain,
        part_of_speech::PartOfSpeech,
        register::Register,
    },
};

/// ドメインエラー
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ItemError {
    /// 語彙エントリーエラー
    #[error("Vocabulary entry error: {0}")]
    VocabularyEntry(#[from] EntryError),
    /// 定義エラー
    #[error("Definition error: {0}")]
    Definition(#[from] DefinitionError),
    /// 定義が空の場合
    #[error("At least one definition is required")]
    NoDefinitions,
    /// 定義数が上限を超えた場合
    #[error("Cannot add more than {max} definitions")]
    TooManyDefinitions {
        /// 最大定義数
        max: usize,
    },
    /// 例文数が上限を超えた場合
    #[error("Cannot add more than {max} examples")]
    TooManyExamples {
        /// 最大例文数
        max: usize,
    },
    /// 指定されたインデックスの定義が存在しない場合
    #[error("Definition with index {index} not found")]
    DefinitionNotFound {
        /// インデックス
        index: usize,
    },
}

/// 語彙項目（完全版・集約ルート）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabularyItem {
    id:            ItemId,
    entry:         VocabularyEntry,
    definitions:   Vec<Definition>,
    examples:      Vec<Example>,
    collocations:  Vec<Collocation>,
    pronunciation: Option<String>,
    synonyms:      Vec<String>,
    antonyms:      Vec<String>,
    metadata:      HashMap<String, String>,
    version:       u64,
    created_at:    DateTime<Utc>,
    updated_at:    DateTime<Utc>,
    deleted_at:    Option<DateTime<Utc>>,
}

impl VocabularyItem {
    const MAX_DEFINITIONS: usize = 10;
    const MAX_EXAMPLES: usize = 20;

    /// 新しい語彙項目を作成
    ///
    /// # Errors
    ///
    /// - 語彙エントリーの作成に失敗した場合
    /// - 定義が空の場合
    /// - 定義の作成に失敗した場合
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        word: &str,
        part_of_speech: PartOfSpeech,
        cefr_level: Option<CefrLevel>,
        register: Register,
        domain: Domain,
        definitions: Vec<&str>,
    ) -> Result<Self, ItemError> {
        // 定義が空の場合はエラー
        if definitions.is_empty() {
            return Err(ItemError::NoDefinitions);
        }

        // 最初の定義を primary として扱う
        let primary_definition = definitions[0];

        // 語彙エントリーを作成
        let entry = VocabularyEntry::new(
            word,
            primary_definition,
            part_of_speech,
            cefr_level,
            register,
            domain,
        )?;

        // 定義リストを作成
        let mut definition_list = Vec::new();
        for def_text in definitions {
            if definition_list.len() >= Self::MAX_DEFINITIONS {
                return Err(ItemError::TooManyDefinitions {
                    max: Self::MAX_DEFINITIONS,
                });
            }
            definition_list.push(Definition::new(def_text)?);
        }

        let now = Utc::now();
        Ok(Self {
            id: ItemId::new(),
            entry,
            definitions: definition_list,
            examples: Vec::new(),
            collocations: Vec::new(),
            pronunciation: None,
            synonyms: Vec::new(),
            antonyms: Vec::new(),
            metadata: HashMap::new(),
            version: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    /// ID を取得
    #[must_use]
    pub const fn id(&self) -> &ItemId {
        &self.id
    }

    /// 語彙エントリーを取得
    #[must_use]
    pub const fn entry(&self) -> &VocabularyEntry {
        &self.entry
    }

    /// 単語を取得
    #[must_use]
    pub fn word(&self) -> &str {
        self.entry.word()
    }

    /// 定義を取得
    #[must_use]
    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    /// 例文を取得
    #[must_use]
    pub fn examples(&self) -> &[Example] {
        &self.examples
    }

    /// コロケーションを取得
    #[must_use]
    pub fn collocations(&self) -> &[Collocation] {
        &self.collocations
    }

    /// 発音を取得
    #[must_use]
    pub fn pronunciation(&self) -> Option<&str> {
        self.pronunciation.as_deref()
    }

    /// 類義語を取得
    #[must_use]
    pub fn synonyms(&self) -> &[String] {
        &self.synonyms
    }

    /// 反意語を取得
    #[must_use]
    pub fn antonyms(&self) -> &[String] {
        &self.antonyms
    }

    /// 定義を追加
    ///
    /// # Errors
    ///
    /// - 定義が最大数を超える場合
    /// - 定義の作成に失敗した場合
    pub fn add_definition(&mut self, text: &str) -> Result<(), ItemError> {
        if self.definitions.len() >= Self::MAX_DEFINITIONS {
            return Err(ItemError::TooManyDefinitions {
                max: Self::MAX_DEFINITIONS,
            });
        }

        let definition = Definition::new(text)?;
        self.definitions.push(definition);
        self.touch();
        Ok(())
    }

    /// 例文を追加
    ///
    /// # Errors
    ///
    /// - 例文が最大数を超える場合
    /// - 例文の作成に失敗した場合
    pub fn add_example(
        &mut self,
        sentence: &str,
        translation: Option<&str>,
    ) -> Result<(), ItemError> {
        if self.examples.len() >= Self::MAX_EXAMPLES {
            return Err(ItemError::TooManyExamples {
                max: Self::MAX_EXAMPLES,
            });
        }

        let example = Example::new(sentence, translation)?;
        self.examples.push(example);
        self.touch();
        Ok(())
    }

    /// コロケーションを追加
    pub fn add_collocation(&mut self, pattern: &str, examples: Vec<&str>) {
        let collocation = Collocation::new(pattern, examples);
        self.collocations.push(collocation);
        self.touch();
    }

    /// 発音を設定
    pub fn set_pronunciation(&mut self, pronunciation: Option<&str>) {
        self.pronunciation = pronunciation.map(str::to_string);
        self.touch();
    }

    /// 類義語を追加
    pub fn add_synonym(&mut self, synonym: &str) {
        if !self.synonyms.iter().any(|s| s == synonym) {
            self.synonyms.push(synonym.to_string());
            self.touch();
        }
    }

    /// 反意語を追加
    pub fn add_antonym(&mut self, antonym: &str) {
        if !self.antonyms.iter().any(|a| a == antonym) {
            self.antonyms.push(antonym.to_string());
            self.touch();
        }
    }

    /// メタデータを設定
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
        self.touch();
    }

    /// CEFR レベルを更新
    pub fn update_cefr_level(&mut self, level: Option<CefrLevel>) {
        self.entry.update_cefr_level(level);
        self.touch();
    }

    /// データベースから復元（バリデーションなし）
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn from_database(
        id: ItemId,
        entry: VocabularyEntry,
        definitions: Vec<Definition>,
        examples: Vec<Example>,
        collocations: Vec<Collocation>,
        pronunciation: Option<String>,
        synonyms: Vec<String>,
        antonyms: Vec<String>,
        metadata: HashMap<String, String>,
        version: u64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            entry,
            definitions,
            examples,
            collocations,
            pronunciation,
            synonyms,
            antonyms,
            metadata,
            version,
            created_at,
            updated_at,
            deleted_at,
        }
    }
}

impl Entity for VocabularyItem {
    type Id = ItemId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl SoftDeletable for VocabularyItem {
    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    fn soft_delete(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.touch();
    }

    fn restore(&mut self) {
        self.deleted_at = None;
        self.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{
        domain::Domain,
        part_of_speech::{NounType, PartOfSpeech},
        register::Register,
    };

    #[test]
    fn vocabulary_item_should_be_created_with_valid_data() {
        let item = VocabularyItem::new(
            "ephemeral",
            PartOfSpeech::Noun(NounType::Countable),
            Some(CefrLevel::C1),
            Register::Formal,
            Domain::General,
            vec!["lasting for a very short time", "temporary and fleeting"],
        )
        .unwrap();

        assert_eq!(item.word(), "ephemeral");
        assert_eq!(item.definitions().len(), 2);
        assert!(item.examples().is_empty());
        assert_eq!(item.version(), 1);
    }

    #[test]
    fn vocabulary_item_should_manage_definitions() {
        let mut item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
            vec!["primary definition"],
        )
        .unwrap();

        assert_eq!(item.definitions().len(), 1);

        item.add_definition("secondary definition").unwrap();
        assert_eq!(item.definitions().len(), 2);
    }

    #[test]
    fn vocabulary_item_should_limit_definitions() {
        let mut item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
            vec!["primary definition"],
        )
        .unwrap();

        // 最大数まで追加
        for i in 1..VocabularyItem::MAX_DEFINITIONS {
            item.add_definition(&format!("definition {i}")).unwrap();
        }

        // 最大数を超えるとエラー
        let result = item.add_definition("too many");
        assert!(matches!(
            result,
            Err(ItemError::TooManyDefinitions { max: 10 })
        ));
    }

    #[test]
    fn vocabulary_item_should_manage_examples() {
        let mut item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
            vec!["primary definition"],
        )
        .unwrap();

        item.add_example("This is a test sentence.", Some("これはテスト文です。"))
            .unwrap();

        assert_eq!(item.examples().len(), 1);
        assert_eq!(item.examples()[0].sentence(), "This is a test sentence.");
    }

    #[test]
    fn vocabulary_item_should_manage_synonyms_and_antonyms() {
        let mut item = VocabularyItem::new(
            "big",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
            vec!["of considerable size"],
        )
        .unwrap();

        item.add_synonym("large");
        item.add_synonym("huge");
        item.add_antonym("small");

        assert_eq!(item.synonyms().len(), 2);
        assert_eq!(item.antonyms().len(), 1);

        // 重複は追加されない
        item.add_synonym("large");
        assert_eq!(item.synonyms().len(), 2);
    }

    #[test]
    fn vocabulary_item_should_implement_entity_trait() {
        let item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
            vec!["primary definition"],
        )
        .unwrap();

        assert_ne!(item.id(), &ItemId::default());
        assert_eq!(item.version(), 1);
        assert!(item.created_at() <= Utc::now());
        assert!(item.updated_at() <= Utc::now());
    }

    #[test]
    fn vocabulary_item_should_implement_soft_deletable() {
        let mut item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
            vec!["primary definition"],
        )
        .unwrap();

        assert!(item.deleted_at().is_none());

        item.soft_delete();
        assert!(item.deleted_at().is_some());

        item.restore();
        assert!(item.deleted_at().is_none());
    }
}
