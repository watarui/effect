//! 集約ルート
//!
//! Vocabulary ドメインの集約を定義

use chrono::{DateTime, Utc};
use shared_error::DomainResult;
use uuid::Uuid;

use super::{
    entities::VocabularyItem,
    events::VocabularyDomainEvent,
    value_objects::{Domain as VocabularyDomain, PartOfSpeech, Register},
};

/// 語彙エントリ集約
#[derive(Debug, Clone)]
pub struct VocabularyEntry {
    /// エントリID
    id:             Uuid,
    /// 単語
    word:           String,
    /// 関連する項目
    items:          Vec<VocabularyItem>,
    /// バージョン（楽観的ロック用）
    version:        u32,
    /// 作成日時
    created_at:     DateTime<Utc>,
    /// 更新日時
    updated_at:     DateTime<Utc>,
    /// 保留中のイベント
    pending_events: Vec<VocabularyDomainEvent>,
}

impl VocabularyEntry {
    /// 新しい語彙エントリを作成
    pub fn create(word: String) -> DomainResult<Self> {
        let now = Utc::now();
        let id = Uuid::new_v4();

        let mut entry = Self {
            id,
            word: word.clone(),
            items: Vec::new(),
            version: 0,
            created_at: now,
            updated_at: now,
            pending_events: Vec::new(),
        };

        // イベントを記録
        entry.record_event(VocabularyDomainEvent::EntryCreated {
            entry_id: id,
            word,
            occurred_at: now,
        });

        Ok(entry)
    }

    /// 語彙項目を追加
    pub fn add_item(
        &mut self,
        definitions: Vec<String>,
        part_of_speech: PartOfSpeech,
        register: Register,
        domain: VocabularyDomain,
        created_by: Uuid,
    ) -> DomainResult<Uuid> {
        let item = VocabularyItem::new(
            self.id,
            self.word.clone(),
            definitions.clone(),
            part_of_speech.clone(),
            register.clone(),
            domain.clone(),
            created_by,
        )?;

        let item_id = item.id();
        self.items.push(item);
        self.updated_at = Utc::now();

        // イベントを記録
        self.record_event(VocabularyDomainEvent::ItemCreated {
            item_id,
            entry_id: self.id,
            word: self.word.clone(),
            definitions,
            part_of_speech,
            register,
            domain,
            created_by,
            occurred_at: self.updated_at,
        });

        Ok(item_id)
    }

    /// イベントを記録
    fn record_event(&mut self, event: VocabularyDomainEvent) {
        self.pending_events.push(event);
    }

    /// 保留中のイベントを取得してクリア
    pub fn take_events(&mut self) -> Vec<VocabularyDomainEvent> {
        std::mem::take(&mut self.pending_events)
    }

    // Getters
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn items(&self) -> &[VocabularyItem] {
        &self.items
    }

    pub fn has_items(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
