//! アウトバウンドポート（リポジトリ）

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

/// 検索エンジンリポジトリ
#[async_trait]
pub trait SearchEngineRepository: Send + Sync {
    /// ドキュメントを検索
    async fn search(
        &self,
        query: &SearchQuery,
    ) -> Result<(SearchResult<VocabularySearchItem>, Option<SearchFacets>)>;

    /// オートコンプリート候補を取得
    async fn suggest(&self, prefix: &str, limit: usize) -> Result<Vec<AutocompleteItem>>;

    /// 類似ドキュメントを検索
    async fn find_similar(&self, item_id: Uuid, limit: usize) -> Result<Vec<VocabularySearchItem>>;

    /// ドキュメントをインデックスに追加
    async fn index_document(&self, document: &VocabularySearchItem) -> Result<()>;

    /// 複数ドキュメントをバッチでインデックス
    async fn batch_index(&self, documents: &[VocabularySearchItem]) -> Result<()>;

    /// ドキュメントを削除
    async fn delete_document(&self, item_id: Uuid) -> Result<()>;

    /// インデックスをクリア
    async fn clear_index(&self) -> Result<()>;

    /// インデックス統計を取得
    async fn get_statistics(&self) -> Result<IndexStatistics>;

    /// インデックス設定を取得
    async fn get_settings(&self) -> Result<IndexSettings>;

    /// インデックス設定を更新
    async fn update_settings(&self, settings: &IndexSettings) -> Result<()>;

    /// ヘルスチェック
    async fn health_check(&self) -> Result<()>;
}

/// データソースリポジトリ（PostgreSQL から読み取り）
#[async_trait]
pub trait DataSourceRepository: Send + Sync {
    /// 全ての語彙アイテムを取得（インデックス再構築用）
    async fn get_all_items(&self) -> Result<Vec<VocabularySearchItem>>;

    /// 更新されたアイテムを取得
    async fn get_updated_items(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<VocabularySearchItem>>;

    /// アイテムIDで取得
    async fn get_item_by_id(&self, item_id: Uuid) -> Result<Option<VocabularySearchItem>>;
}

/// 検索ログリポジトリ
#[async_trait]
pub trait SearchLogRepository: Send + Sync {
    /// 検索クエリをログに記録
    async fn log_search(&self, query: &str, results_count: usize) -> Result<()>;

    /// 人気のある検索クエリを取得
    async fn get_popular_queries(&self, limit: usize) -> Result<Vec<String>>;

    /// 検索統計を取得
    async fn get_search_statistics(&self) -> Result<SearchStatistics>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchStatistics {
    pub total_searches:        u64,
    pub unique_queries:        u64,
    pub avg_results_per_query: f64,
    pub top_queries:           Vec<(String, u64)>,
    pub search_trends:         Vec<SearchTrend>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchTrend {
    pub query:          String,
    pub count_increase: i64,
    pub percentage:     f64,
}
