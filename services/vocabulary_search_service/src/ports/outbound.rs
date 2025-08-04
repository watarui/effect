//! アウトバウンドポート（依存関係のインターフェース）

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::{
    error::SearchError,
    search_models::{AnalyzedQuery, MeilisearchQuery, SearchResult, VocabularySearchDocument},
    value_objects::Pagination,
};

/// 検索エンジンインターフェース
#[async_trait]
pub trait SearchEngine: Send + Sync {
    type Document;

    /// ドキュメントを検索
    async fn search(
        &self,
        query: MeilisearchQuery,
        pagination: Pagination,
    ) -> Result<SearchResult<Self::Document>, SearchError>;

    /// ドキュメントをインデックス
    async fn index_document(&self, document: Self::Document) -> Result<(), SearchError>;

    /// ドキュメントを更新
    async fn update_document(&self, document: Self::Document) -> Result<(), SearchError>;

    /// ドキュメントを削除
    async fn delete_document(&self, doc_id: &str) -> Result<(), SearchError>;

    /// 複数のドキュメントを削除
    async fn delete_documents(&self, doc_ids: &[String]) -> Result<(), SearchError>;

    /// インデックスをクリア
    async fn clear_index(&self) -> Result<(), SearchError>;
}

/// クエリ分析インターフェース
#[async_trait]
pub trait QueryAnalyzer: Send + Sync {
    /// クエリを分析
    async fn analyze(&self, query: &str) -> Result<AnalyzedQuery, SearchError>;
}

/// キャッシュサービスインターフェース
#[async_trait]
pub trait CacheService: Send + Sync {
    /// キャッシュから値を取得
    async fn get<T>(&self, key: &str) -> Result<Option<T>, SearchError>
    where
        T: for<'de> Deserialize<'de>;

    /// キャッシュに値を設定
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), SearchError>
    where
        T: Serialize + Sync;

    /// キャッシュから削除
    async fn delete(&self, key: &str) -> Result<(), SearchError>;

    /// パターンに一致するキーをすべて削除
    async fn clear_pattern(&self, pattern: &str) -> Result<(), SearchError>;
}

/// リードモデルリポジトリインターフェース
#[async_trait]
pub trait ReadModelRepository: Send + Sync {
    /// 関連項目を取得
    async fn get_related_items(
        &self,
        item_id: &str,
        relation_type: RelationType,
        limit: usize,
    ) -> Result<Vec<RelatedItem>, SearchError>;

    /// 人気のある項目を取得
    async fn get_popular_items(
        &self,
        limit: usize,
    ) -> Result<Vec<VocabularySearchDocument>, SearchError>;

    /// 最近追加された項目を取得
    async fn get_recent_items(
        &self,
        limit: usize,
    ) -> Result<Vec<VocabularySearchDocument>, SearchError>;
}

/// 関連タイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    Synonyms,
    Antonyms,
    SimilarUsage,
    SameDomain,
    SameLevel,
}

/// 関連項目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedItem {
    pub item_id:        String,
    pub spelling:       String,
    pub disambiguation: String,
    pub relation_score: f32,
    pub relation_type:  RelationType,
}
