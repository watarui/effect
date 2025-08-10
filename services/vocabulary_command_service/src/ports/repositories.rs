use async_trait::async_trait;

use crate::{
    domain::{EntryId, ItemId, VocabularyEntry, VocabularyItem},
    error::Result,
};

/// VocabularyEntry のリポジトリトレイト
#[async_trait]
pub trait VocabularyEntryRepository: Send + Sync {
    /// ID でエントリを検索
    async fn find_by_id(&self, entry_id: &EntryId) -> Result<Option<VocabularyEntry>>;

    /// エントリの存在確認
    async fn exists(&self, entry_id: &EntryId) -> Result<bool>;

    /// エントリを保存
    async fn save(&self, entry: &VocabularyEntry) -> Result<()>;

    /// スペリングでエントリを検索
    async fn find_by_spelling(&self, spelling: &str) -> Result<Option<VocabularyEntry>>;
}

/// VocabularyItem のリポジトリトレイト
#[async_trait]
pub trait VocabularyItemRepository: Send + Sync {
    /// ID でアイテムを検索
    async fn find_by_id(&self, item_id: &ItemId) -> Result<Option<VocabularyItem>>;

    /// アイテムを保存
    async fn save(&self, item: &VocabularyItem) -> Result<()>;

    /// エントリID でアイテムを検索
    async fn find_by_entry_id(&self, entry_id: &EntryId) -> Result<Vec<VocabularyItem>>;

    /// 主要アイテムを取得
    async fn find_primary_by_entry_id(&self, entry_id: &EntryId) -> Result<Option<VocabularyItem>>;
}

/// 統合リポジトリトレイト（トランザクション管理用）
#[async_trait]
pub trait VocabularyRepository: VocabularyEntryRepository + VocabularyItemRepository {
    /// トランザクション開始
    async fn begin_transaction(&self) -> Result<()>;

    /// トランザクションコミット
    async fn commit(&self) -> Result<()>;

    /// トランザクションロールバック
    async fn rollback(&self) -> Result<()>;
}
