//! 出力ポート（外部システムインターフェース）

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::{
        Cursor,
        PageSize,
        PagedResult,
        SortOptions,
        VocabularyEntry,
        VocabularyExample,
        VocabularyFilter,
        VocabularyItem,
        VocabularyStatistics,
    },
    error::Result,
};

/// Read Model リポジトリ
#[async_trait]
pub trait ReadModelRepository: Send + Sync {
    /// エントリを ID で取得
    async fn find_entry_by_id(&self, entry_id: Uuid) -> Result<Option<VocabularyEntry>>;

    /// エントリをスペリングで取得
    async fn find_entry_by_spelling(&self, spelling: &str) -> Result<Option<VocabularyEntry>>;

    /// エントリ一覧を取得
    async fn find_entries(
        &self,
        filter: Option<VocabularyFilter>,
        sort: Option<SortOptions>,
        cursor: Option<Cursor>,
        limit: PageSize,
    ) -> Result<PagedResult<VocabularyEntry>>;

    /// アイテムを ID で取得
    async fn find_item_by_id(&self, item_id: Uuid) -> Result<Option<VocabularyItem>>;

    /// エントリのアイテムを取得
    async fn find_items_by_entry_id(
        &self,
        entry_id: Uuid,
        include_deleted: bool,
    ) -> Result<Vec<VocabularyItem>>;

    /// アイテム一覧を取得
    async fn find_items(
        &self,
        filter: Option<VocabularyFilter>,
        sort: Option<SortOptions>,
        cursor: Option<Cursor>,
        limit: PageSize,
    ) -> Result<PagedResult<VocabularyItem>>;

    /// アイテムの例文を取得
    async fn find_examples_by_item_id(&self, item_id: Uuid) -> Result<Vec<VocabularyExample>>;

    /// 全文検索
    async fn search_items(
        &self,
        search_term: &str,
        filter: Option<VocabularyFilter>,
        cursor: Option<Cursor>,
        limit: PageSize,
    ) -> Result<PagedResult<VocabularyItem>>;

    /// 統計情報を取得
    async fn get_statistics(&self) -> Result<VocabularyStatistics>;

    /// データベース接続をチェック
    async fn health_check(&self) -> Result<()>;
}

/// キャッシュリポジトリ（オプション）
#[async_trait]
pub trait CacheRepository: Send + Sync {
    /// キャッシュから取得
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// キャッシュに保存
    async fn set(&self, key: &str, value: Vec<u8>, ttl_seconds: u64) -> Result<()>;

    /// キャッシュから削除
    async fn delete(&self, key: &str) -> Result<()>;

    /// パターンに一致するキーを削除
    async fn delete_pattern(&self, pattern: &str) -> Result<()>;
}
