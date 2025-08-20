//! インバウンドポート（ユースケース）

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::{
        AutocompleteItem,
        IndexSettings,
        IndexStatistics,
        SearchFacets,
        SearchQuery,
        SearchResult,
        VocabularySearchItem,
    },
    error::Result,
};

/// 検索ユースケース
#[async_trait]
pub trait SearchUseCase: Send + Sync {
    /// 語彙アイテムを検索
    async fn search_items(
        &self,
        query: SearchQuery,
    ) -> Result<(SearchResult<VocabularySearchItem>, Option<SearchFacets>)>;

    /// オートコンプリート候補を取得
    async fn get_autocomplete(&self, prefix: &str, limit: usize) -> Result<Vec<AutocompleteItem>>;

    /// 類似アイテムを検索
    async fn find_similar_items(
        &self,
        item_id: Uuid,
        limit: usize,
    ) -> Result<Vec<VocabularySearchItem>>;

    /// 頻出検索語を取得
    async fn get_popular_searches(&self, limit: usize) -> Result<Vec<String>>;
}

/// インデックス管理ユースケース
#[async_trait]
pub trait IndexManagementUseCase: Send + Sync {
    /// インデックスを再構築
    async fn rebuild_index(&self) -> Result<()>;

    /// インデックスの統計情報を取得
    async fn get_index_statistics(&self) -> Result<IndexStatistics>;

    /// インデックス設定を取得
    async fn get_index_settings(&self) -> Result<IndexSettings>;

    /// インデックス設定を更新
    async fn update_index_settings(&self, settings: IndexSettings) -> Result<()>;

    /// 単一ドキュメントをインデックスに追加/更新
    async fn index_document(&self, item: VocabularySearchItem) -> Result<()>;

    /// ドキュメントをインデックスから削除
    async fn delete_document(&self, item_id: Uuid) -> Result<()>;

    /// バッチでドキュメントをインデックス
    async fn batch_index_documents(&self, items: Vec<VocabularySearchItem>) -> Result<()>;
}

/// ヘルスチェックユースケース
#[async_trait]
pub trait HealthCheckUseCase: Send + Sync {
    async fn check_health(&self) -> Result<HealthStatus>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    pub is_healthy:         bool,
    pub meilisearch_status: SearchEngineStatus,
    pub index_status:       IndexStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SearchEngineStatus {
    Connected,
    Disconnected,
    Error(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum IndexStatus {
    Ready,
    Indexing,
    NotInitialized,
    Error(String),
}
