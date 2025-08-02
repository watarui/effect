//! 語彙エントリーエンティティ
//!
//! 軽量な語彙エントリー（検索・一覧表示用）

use chrono::{DateTime, Utc};
use common_types::EntryId;
use domain_events::CefrLevel;
use infrastructure::repository::{Entity, entity::SoftDeletable};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::value_objects::{
    domain::Domain,
    part_of_speech::PartOfSpeech,
    register::Register,
};

/// ドメインエラー
#[derive(Error, Debug, PartialEq, Eq)]
pub enum EntryError {
    /// 単語が空の場合
    #[error("Word cannot be empty")]
    EmptyWord,
    /// 主要定義が空の場合
    #[error("Primary definition cannot be empty")]
    EmptyPrimaryDefinition,
    /// 単語が長すぎる場合
    #[error("Word is too long (max: {max}, actual: {actual})")]
    WordTooLong {
        /// 最大文字数
        max:    usize,
        /// 実際の文字数
        actual: usize,
    },
    /// 主要定義が長すぎる場合
    #[error("Primary definition is too long (max: {max}, actual: {actual})")]
    PrimaryDefinitionTooLong {
        /// 最大文字数
        max:    usize,
        /// 実際の文字数
        actual: usize,
    },
}

/// 語彙エントリー（軽量版）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabularyEntry {
    id:                 EntryId,
    word:               String,
    primary_definition: String,
    part_of_speech:     PartOfSpeech,
    cefr_level:         Option<CefrLevel>,
    register:           Register,
    domain:             Domain,
    version:            u64,
    created_at:         DateTime<Utc>,
    updated_at:         DateTime<Utc>,
    deleted_at:         Option<DateTime<Utc>>,
}

impl VocabularyEntry {
    const MAX_WORD_LENGTH: usize = 100;
    const MAX_DEFINITION_LENGTH: usize = 200;

    /// 新しい語彙エントリーを作成
    ///
    /// # Errors
    ///
    /// - 単語または定義が空の場合
    /// - 単語または定義が最大長を超える場合
    /// - CEFR レベルが無効な場合
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        word: &str,
        primary_definition: &str,
        part_of_speech: PartOfSpeech,
        cefr_level: Option<CefrLevel>,
        register: Register,
        domain: Domain,
    ) -> Result<Self, EntryError> {
        let trimmed_word = word.trim();
        let trimmed_definition = primary_definition.trim();

        // バリデーション
        if trimmed_word.is_empty() {
            return Err(EntryError::EmptyWord);
        }
        if trimmed_definition.is_empty() {
            return Err(EntryError::EmptyPrimaryDefinition);
        }
        if trimmed_word.len() > Self::MAX_WORD_LENGTH {
            return Err(EntryError::WordTooLong {
                max:    Self::MAX_WORD_LENGTH,
                actual: trimmed_word.len(),
            });
        }
        if trimmed_definition.len() > Self::MAX_DEFINITION_LENGTH {
            return Err(EntryError::PrimaryDefinitionTooLong {
                max:    Self::MAX_DEFINITION_LENGTH,
                actual: trimmed_definition.len(),
            });
        }

        let now = Utc::now();
        Ok(Self {
            id: EntryId::new(),
            word: trimmed_word.to_string(),
            primary_definition: trimmed_definition.to_string(),
            part_of_speech,
            cefr_level,
            register,
            domain,
            version: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    /// ID を取得
    #[must_use]
    pub const fn id(&self) -> &EntryId {
        &self.id
    }

    /// 単語を取得
    #[must_use]
    pub fn word(&self) -> &str {
        &self.word
    }

    /// 主要定義を取得
    #[must_use]
    pub fn primary_definition(&self) -> &str {
        &self.primary_definition
    }

    /// 品詞を取得
    #[must_use]
    pub const fn part_of_speech(&self) -> &PartOfSpeech {
        &self.part_of_speech
    }

    /// CEFR レベルを取得
    #[must_use]
    pub const fn cefr_level(&self) -> Option<CefrLevel> {
        self.cefr_level
    }

    /// レジスターを取得
    #[must_use]
    pub const fn register(&self) -> Register {
        self.register
    }

    /// ドメインを取得
    #[must_use]
    pub const fn domain(&self) -> &Domain {
        &self.domain
    }

    /// CEFR レベルを更新
    pub fn update_cefr_level(&mut self, level: Option<CefrLevel>) {
        self.cefr_level = level;
        self.touch();
    }

    /// データベースから復元（バリデーションなし）
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn from_database(
        id: EntryId,
        word: String,
        primary_definition: String,
        part_of_speech: PartOfSpeech,
        cefr_level: Option<CefrLevel>,
        register: Register,
        domain: Domain,
        version: u64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            word,
            primary_definition,
            part_of_speech,
            cefr_level,
            register,
            domain,
            version,
            created_at,
            updated_at,
            deleted_at,
        }
    }
}

impl Entity for VocabularyEntry {
    type Id = EntryId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn id_as_bytes(&self) -> Vec<u8> {
        self.id.as_bytes().to_vec()
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

impl SoftDeletable for VocabularyEntry {
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
    fn vocabulary_entry_should_be_created_with_valid_data() {
        let entry = VocabularyEntry::new(
            "ephemeral",
            "lasting for a very short time",
            PartOfSpeech::Noun(NounType::Countable),
            Some(CefrLevel::C1),
            Register::Formal,
            Domain::General,
        )
        .unwrap();

        assert_eq!(entry.word(), "ephemeral");
        assert_eq!(entry.primary_definition(), "lasting for a very short time");
        assert_eq!(entry.cefr_level(), Some(CefrLevel::C1));
        assert_eq!(entry.register(), Register::Formal);
        assert_eq!(entry.version(), 1);
    }

    #[test]
    fn vocabulary_entry_should_validate_empty_word() {
        let result = VocabularyEntry::new(
            "",
            "definition",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
        );

        assert!(matches!(result, Err(EntryError::EmptyWord)));
    }

    #[test]
    fn vocabulary_entry_should_validate_empty_definition() {
        let result = VocabularyEntry::new(
            "word",
            "",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
        );

        assert!(matches!(result, Err(EntryError::EmptyPrimaryDefinition)));
    }

    #[test]
    fn vocabulary_entry_should_validate_word_length() {
        let long_word = "a".repeat(101);
        let result = VocabularyEntry::new(
            &long_word,
            "definition",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
        );

        assert!(matches!(
            result,
            Err(EntryError::WordTooLong {
                max:    100,
                actual: 101,
            })
        ));
    }

    #[test]
    fn vocabulary_entry_should_implement_entity_trait() {
        let entry = VocabularyEntry::new(
            "test",
            "test definition",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
        )
        .unwrap();

        assert!(!entry.id_as_bytes().is_empty());
        assert_eq!(entry.version(), 1);
        assert!(entry.created_at() <= Utc::now());
        assert!(entry.updated_at() <= Utc::now());
    }

    #[test]
    fn vocabulary_entry_should_implement_soft_deletable() {
        let mut entry = VocabularyEntry::new(
            "test",
            "test definition",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
        )
        .unwrap();

        assert!(entry.deleted_at().is_none());

        entry.soft_delete();
        assert!(entry.deleted_at().is_some());

        entry.restore();
        assert!(entry.deleted_at().is_none());
    }
}
