//! 入力ポート（ユースケースインターフェース）

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::{
        Cursor,
        PageSize,
        PagedResult,
        SearchQuery,
        SortOptions,
        VocabularyEntry,
        VocabularyFilter,
        VocabularyItem,
        VocabularyStatistics,
    },
    error::Result,
};

/// 語彙クエリユースケース
#[async_trait]
pub trait VocabularyQueryUseCase: Send + Sync {
    /// エントリを ID で取得
    async fn get_entry_by_id(&self, entry_id: Uuid) -> Result<Option<VocabularyEntry>>;

    /// エントリをスペリングで取得
    async fn get_entry_by_spelling(&self, spelling: &str) -> Result<Option<VocabularyEntry>>;

    /// エントリ一覧を取得（ページネーション付き）
    async fn list_entries(
        &self,
        filter: Option<VocabularyFilter>,
        sort: Option<SortOptions>,
        cursor: Option<Cursor>,
        page_size: PageSize,
    ) -> Result<PagedResult<VocabularyEntry>>;

    /// アイテムを ID で取得
    async fn get_item_by_id(&self, item_id: Uuid) -> Result<Option<VocabularyItem>>;

    /// エントリのアイテム一覧を取得
    async fn list_items_by_entry(
        &self,
        entry_id: Uuid,
        include_deleted: bool,
    ) -> Result<Vec<VocabularyItem>>;

    /// アイテム一覧を取得（ページネーション付き）
    async fn list_items(
        &self,
        filter: Option<VocabularyFilter>,
        sort: Option<SortOptions>,
        cursor: Option<Cursor>,
        page_size: PageSize,
    ) -> Result<PagedResult<VocabularyItem>>;

    /// 検索（エントリとアイテムの両方）
    async fn search(
        &self,
        query: SearchQuery,
        filter: Option<VocabularyFilter>,
        cursor: Option<Cursor>,
        page_size: PageSize,
    ) -> Result<PagedResult<VocabularyItem>>;

    /// 統計情報を取得
    async fn get_statistics(&self) -> Result<VocabularyStatistics>;
}

/// ヘルスチェックユースケース
#[async_trait]
pub trait HealthCheckUseCase: Send + Sync {
    /// サービスの健全性をチェック
    async fn check_health(&self) -> Result<HealthStatus>;
}

/// ヘルスステータス
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy:      bool,
    pub database_status: DatabaseStatus,
    pub message:         Option<String>,
}

#[derive(Debug, Clone)]
pub enum DatabaseStatus {
    Connected,
    Disconnected,
    Error(String),
}
