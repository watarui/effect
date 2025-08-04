//! 検索サービスのエラー型

use shared_error::DomainError;
use thiserror::Error;

use super::value_objects::{FuzzinessError, PaginationError, SearchQueryError, SearchScoreError};

/// 検索サービスの Result 型
pub type SearchResult<T> = Result<T, SearchError>;

/// 検索サービスのエラー
#[derive(Debug, Error)]
pub enum SearchError {
    /// ドメインエラー
    #[error(transparent)]
    Domain(#[from] DomainError),

    /// 検索クエリエラー
    #[error(transparent)]
    Query(#[from] SearchQueryError),

    /// 検索スコアエラー
    #[error(transparent)]
    Score(#[from] SearchScoreError),

    /// ページネーションエラー
    #[error(transparent)]
    Pagination(#[from] PaginationError),

    /// ファジネスエラー
    #[error(transparent)]
    Fuzziness(#[from] FuzzinessError),

    /// 検索エンジンエラー
    #[error("Search engine error: {0}")]
    SearchEngine(String),

    /// キャッシュエラー
    #[error("Cache error: {0}")]
    Cache(String),

    /// シリアライゼーションエラー
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// インデックスエラー
    #[error("Index error: {0}")]
    Index(String),

    /// クエリ分析エラー
    #[error("Query analysis error: {0}")]
    QueryAnalysis(String),

    /// 内部エラー
    #[error("Internal error: {0}")]
    Internal(String),
}

// エラー変換の実装
impl From<serde_json::Error> for SearchError {
    fn from(err: serde_json::Error) -> Self {
        SearchError::Serialization(err.to_string())
    }
}

impl From<redis::RedisError> for SearchError {
    fn from(err: redis::RedisError) -> Self {
        SearchError::Cache(err.to_string())
    }
}

// SearchError に map_err メソッドを追加するためのヘルパー
impl SearchError {
    /// 任意のエラーを内部エラーに変換
    pub fn from_error<E: std::error::Error>(err: E) -> Self {
        SearchError::Internal(err.to_string())
    }

    /// 検索エンジンエラーを作成
    pub fn search_engine<E: std::error::Error>(err: E) -> Self {
        SearchError::SearchEngine(err.to_string())
    }

    /// インデックスエラーを作成
    pub fn index<E: std::error::Error>(err: E) -> Self {
        SearchError::Index(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let query_error = SearchQueryError::Empty;
        let search_error: SearchError = query_error.into();
        assert!(matches!(search_error, SearchError::Query(_)));
    }
}
