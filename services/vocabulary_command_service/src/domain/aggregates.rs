use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::value_objects::{Disambiguation, EntryId, ItemId, Spelling, Version, VocabularyStatus},
    error::{Error, Result},
};

/// VocabularyEntry 集約（見出し語）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyEntry {
    pub entry_id:   EntryId,
    pub spelling:   Spelling,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version:    Version,
}

impl VocabularyEntry {
    /// 新しいエントリを作成
    pub fn create(spelling: Spelling) -> Self {
        let now = Utc::now();
        Self {
            entry_id: EntryId::new(),
            spelling,
            created_at: now,
            updated_at: now,
            version: Version::initial(),
        }
    }

    /// スペリングを更新
    pub fn update_spelling(&mut self, spelling: Spelling) {
        self.spelling = spelling;
        self.updated_at = Utc::now();
        self.version = self.version.increment();
    }
}

/// VocabularyItem 集約（語彙項目）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyItem {
    pub item_id:        ItemId,
    pub entry_id:       EntryId,
    pub spelling:       Spelling,
    pub disambiguation: Disambiguation,
    pub is_primary:     bool,
    pub status:         VocabularyStatus,
    pub is_deleted:     bool,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
    pub version:        Version,
}

impl VocabularyItem {
    /// 新しい語彙項目を作成
    pub fn create(entry_id: EntryId, spelling: Spelling, disambiguation: Disambiguation) -> Self {
        let now = Utc::now();
        Self {
            item_id: ItemId::new(),
            entry_id,
            spelling,
            disambiguation,
            is_primary: false,
            status: VocabularyStatus::Draft,
            is_deleted: false,
            created_at: now,
            updated_at: now,
            version: Version::initial(),
        }
    }

    /// 主要項目として設定
    pub fn set_as_primary(&mut self) -> Result<()> {
        if self.status != VocabularyStatus::Published {
            return Err(Error::Domain(
                "Only published items can be set as primary".to_string(),
            ));
        }
        self.is_primary = true;
        self.updated_at = Utc::now();
        self.version = self.version.increment();
        Ok(())
    }

    /// 主要項目設定を解除
    pub fn unset_primary(&mut self) {
        self.is_primary = false;
        self.updated_at = Utc::now();
        self.version = self.version.increment();
    }

    /// 公開する
    pub fn publish(&mut self) -> Result<()> {
        match self.status {
            VocabularyStatus::Draft => {
                self.status = VocabularyStatus::Published;
                self.updated_at = Utc::now();
                self.version = self.version.increment();
                Ok(())
            },
            VocabularyStatus::PendingAI => Err(Error::Domain(
                "Cannot publish item while AI enrichment is pending".to_string(),
            )),
            VocabularyStatus::Published => {
                Err(Error::Domain("Item is already published".to_string()))
            },
        }
    }

    /// AI エンリッチメントをリクエスト
    pub fn request_ai_enrichment(&mut self) -> Result<()> {
        match self.status {
            VocabularyStatus::Draft => {
                self.status = VocabularyStatus::PendingAI;
                self.updated_at = Utc::now();
                self.version = self.version.increment();
                Ok(())
            },
            VocabularyStatus::PendingAI => Err(Error::Domain(
                "AI enrichment is already pending".to_string(),
            )),
            VocabularyStatus::Published => Err(Error::Domain(
                "Cannot request AI enrichment for published items".to_string(),
            )),
        }
    }

    /// AI エンリッチメント完了
    pub fn complete_ai_enrichment(&mut self) -> Result<()> {
        match self.status {
            VocabularyStatus::PendingAI => {
                self.status = VocabularyStatus::Draft;
                self.updated_at = Utc::now();
                self.version = self.version.increment();
                Ok(())
            },
            _ => Err(Error::Domain(
                "Item is not pending AI enrichment".to_string(),
            )),
        }
    }

    /// 曖昧性解消を更新
    pub fn update_disambiguation(&mut self, disambiguation: Disambiguation) -> Result<()> {
        if self.status == VocabularyStatus::Published {
            return Err(Error::Domain(
                "Cannot update disambiguation for published items".to_string(),
            ));
        }
        self.disambiguation = disambiguation;
        self.updated_at = Utc::now();
        self.version = self.version.increment();
        Ok(())
    }

    /// アイテムを削除（ソフトデリート）
    pub fn mark_as_deleted(&mut self) -> Result<()> {
        if self.is_deleted {
            return Err(Error::Conflict("Item is already deleted".to_string()));
        }
        self.is_deleted = true;
        self.updated_at = Utc::now();
        self.version = self.version.increment();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vocabulary_entry() {
        let spelling = Spelling::new("apple".to_string()).unwrap();
        let entry = VocabularyEntry::create(spelling);

        assert_eq!(entry.spelling.as_str(), "apple");
        assert_eq!(entry.version.value(), 1);
    }

    #[test]
    fn test_create_vocabulary_item() {
        let entry_id = EntryId::new();
        let spelling = Spelling::new("apple".to_string()).unwrap();
        let disambiguation = Disambiguation::new(Some("fruit".to_string())).unwrap();

        let item = VocabularyItem::create(entry_id, spelling, disambiguation);

        assert_eq!(item.spelling.as_str(), "apple");
        assert_eq!(item.disambiguation.as_option(), Some("fruit"));
        assert!(!item.is_primary);
        assert_eq!(item.status, VocabularyStatus::Draft);
        assert_eq!(item.version.value(), 1);
    }

    #[test]
    fn test_publish_item() {
        let entry_id = EntryId::new();
        let spelling = Spelling::new("apple".to_string()).unwrap();
        let disambiguation = Disambiguation::new(None).unwrap();

        let mut item = VocabularyItem::create(entry_id, spelling, disambiguation);

        // Draft から Published へ
        assert!(item.publish().is_ok());
        assert_eq!(item.status, VocabularyStatus::Published);
        assert_eq!(item.version.value(), 2);

        // すでに Published の場合はエラー
        assert!(item.publish().is_err());
    }

    #[test]
    fn test_set_as_primary() {
        let entry_id = EntryId::new();
        let spelling = Spelling::new("apple".to_string()).unwrap();
        let disambiguation = Disambiguation::new(None).unwrap();

        let mut item = VocabularyItem::create(entry_id, spelling, disambiguation);

        // Draft の状態では主要項目に設定できない
        assert!(item.set_as_primary().is_err());

        // Published にしてから設定
        item.publish().unwrap();
        assert!(item.set_as_primary().is_ok());
        assert!(item.is_primary);
        assert_eq!(item.version.value(), 3);
    }

    #[test]
    fn test_ai_enrichment_flow() {
        let entry_id = EntryId::new();
        let spelling = Spelling::new("apple".to_string()).unwrap();
        let disambiguation = Disambiguation::new(None).unwrap();

        let mut item = VocabularyItem::create(entry_id, spelling, disambiguation);

        // AI エンリッチメントをリクエスト
        assert!(item.request_ai_enrichment().is_ok());
        assert_eq!(item.status, VocabularyStatus::PendingAI);

        // Pending 中は公開できない
        assert!(item.publish().is_err());

        // AI エンリッチメント完了
        assert!(item.complete_ai_enrichment().is_ok());
        assert_eq!(item.status, VocabularyStatus::Draft);

        // 完了後は公開可能
        assert!(item.publish().is_ok());
    }
}
