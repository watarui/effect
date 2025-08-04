//! エンティティ
//!
//! ドメインのエンティティを定義

use chrono::{DateTime, Utc};
use shared_error::{DomainError, DomainResult};
use uuid::Uuid;

use super::value_objects::{Domain, PartOfSpeech, Register};

/// 語彙項目エンティティ
#[derive(Debug, Clone)]
pub struct VocabularyItem {
    /// 項目ID
    id:             Uuid,
    /// エントリID
    entry_id:       Uuid,
    /// 単語
    word:           String,
    /// 定義
    definitions:    Vec<String>,
    /// 品詞
    part_of_speech: PartOfSpeech,
    /// レジスター
    register:       Register,
    /// ドメイン
    domain:         Domain,
    /// 作成者
    created_by:     Uuid,
    /// 公開状態
    is_published:   bool,
    /// 作成日時
    created_at:     DateTime<Utc>,
    /// 更新日時
    updated_at:     DateTime<Utc>,
}

impl VocabularyItem {
    /// 新しい語彙項目を作成
    pub fn new(
        entry_id: Uuid,
        word: String,
        definitions: Vec<String>,
        part_of_speech: PartOfSpeech,
        register: Register,
        domain: Domain,
        created_by: Uuid,
    ) -> DomainResult<Self> {
        if definitions.is_empty() {
            return Err(DomainError::Validation(
                "At least one definition is required".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            entry_id,
            word,
            definitions,
            part_of_speech,
            register,
            domain,
            created_by,
            is_published: false,
            created_at: now,
            updated_at: now,
        })
    }

    /// 定義を更新
    pub fn update_definitions(&mut self, definitions: Vec<String>) -> DomainResult<()> {
        if definitions.is_empty() {
            return Err(DomainError::Validation(
                "At least one definition is required".to_string(),
            ));
        }
        self.definitions = definitions;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 公開
    pub fn publish(&mut self) -> DomainResult<()> {
        if self.is_published {
            return Err(DomainError::InvalidState(
                "Item is already published".to_string(),
            ));
        }
        self.is_published = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    // Getters
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn entry_id(&self) -> Uuid {
        self.entry_id
    }

    pub fn is_published(&self) -> bool {
        self.is_published
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn part_of_speech(&self) -> &PartOfSpeech {
        &self.part_of_speech
    }

    pub fn register(&self) -> &Register {
        &self.register
    }

    pub fn domain(&self) -> &Domain {
        &self.domain
    }

    pub fn created_by(&self) -> Uuid {
        self.created_by
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
