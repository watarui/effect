//! エラー変換用の拡張トレイト

use crate::domain::error::SearchError;

/// Meilisearch エラーを SearchError に変換するための拡張トレイト
pub trait MeilisearchErrorExt<T> {
    fn map_index_err(self) -> Result<T, SearchError>;
    fn map_search_err(self) -> Result<T, SearchError>;
}

impl<T, E> MeilisearchErrorExt<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn map_index_err(self) -> Result<T, SearchError> {
        self.map_err(|e| SearchError::Index(e.to_string()))
    }

    fn map_search_err(self) -> Result<T, SearchError> {
        self.map_err(|e| SearchError::SearchEngine(e.to_string()))
    }
}
