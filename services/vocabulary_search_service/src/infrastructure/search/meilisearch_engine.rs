//! Meilisearch 検索エンジンの実装

use async_trait::async_trait;
use tracing::info;

use crate::{
    domain::{
        error::SearchError,
        search_models::{MeilisearchQuery, SearchResult, VocabularySearchDocument},
        value_objects::Pagination,
    },
    ports::outbound::SearchEngine,
};

/// Meilisearch エンジン実装
pub struct MeilisearchEngine {
    _client:    meilisearch_sdk::client::Client,
    index_name: String,
}

impl MeilisearchEngine {
    /// 新しい Meilisearch エンジンを作成
    pub fn new(
        url: impl AsRef<str>,
        api_key: impl AsRef<str>,
        index_name: impl Into<String>,
    ) -> Self {
        let client = meilisearch_sdk::client::Client::new(url.as_ref(), Some(api_key.as_ref()));

        Self {
            _client:    client.expect("Failed to create Meilisearch client"),
            index_name: index_name.into(),
        }
    }
}

#[async_trait]
impl SearchEngine for MeilisearchEngine {
    type Document = VocabularySearchDocument;

    async fn search(
        &self,
        query: MeilisearchQuery,
        _pagination: Pagination,
    ) -> Result<SearchResult<Self::Document>, SearchError> {
        info!(
            "Searching index {} with query: {}",
            self.index_name, query.query_string
        );

        // TODO: Meilisearch SDK のバージョンアップ後に実装
        // 一時的なモック実装
        Ok(SearchResult {
            hits:       vec![],
            total_hits: 0,
            max_score:  0.0,
            facets:     None,
        })
    }

    async fn index_document(&self, document: Self::Document) -> Result<(), SearchError> {
        info!("Indexing document: {}", document.item_id);
        // TODO: 実装
        Ok(())
    }

    async fn update_document(&self, document: Self::Document) -> Result<(), SearchError> {
        info!("Updating document: {}", document.item_id);
        // TODO: 実装
        Ok(())
    }

    async fn delete_document(&self, doc_id: &str) -> Result<(), SearchError> {
        info!("Deleting document: {}", doc_id);
        // TODO: 実装
        Ok(())
    }

    async fn delete_documents(&self, doc_ids: &[String]) -> Result<(), SearchError> {
        info!("Deleting {} documents", doc_ids.len());
        // TODO: 実装
        Ok(())
    }

    async fn clear_index(&self) -> Result<(), SearchError> {
        info!("Clearing index: {}", self.index_name);
        // TODO: 実装
        Ok(())
    }
}
