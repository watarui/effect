//! 検索サービス

use async_trait::async_trait;
use tracing::{debug, info};
use uuid::Uuid;

use crate::{
    domain::{AutocompleteItem, SearchFacets, SearchQuery, SearchResult, VocabularySearchItem},
    error::{Result, SearchError},
    ports::{
        inbound::SearchUseCase,
        outbound::{SearchEngineRepository, SearchLogRepository},
    },
};

/// 検索サービス
pub struct SearchService<S, L>
where
    S: SearchEngineRepository,
    L: SearchLogRepository,
{
    search_engine: S,
    search_logger: Option<L>,
}

impl<S, L> SearchService<S, L>
where
    S: SearchEngineRepository,
    L: SearchLogRepository,
{
    pub fn new(search_engine: S, search_logger: Option<L>) -> Self {
        Self {
            search_engine,
            search_logger,
        }
    }
}

#[async_trait]
impl<S, L> SearchUseCase for SearchService<S, L>
where
    S: SearchEngineRepository + Send + Sync,
    L: SearchLogRepository + Send + Sync,
{
    async fn search_items(
        &self,
        query: SearchQuery,
    ) -> Result<(SearchResult<VocabularySearchItem>, Option<SearchFacets>)> {
        debug!("Searching for: {}", query.query);

        // クエリの検証
        if query.query.trim().is_empty() {
            return Err(SearchError::InvalidQuery(
                "Search query cannot be empty".to_string(),
            ));
        }

        // 検索実行
        let result = self.search_engine.search(&query).await?;

        // 検索ログを記録（非同期、エラーは無視）
        // Note: ログの記録は別タスクで行うが、SearchLogRepository が Send ではないため
        // 現時点では同期的に実行
        if let Some(logger) = &self.search_logger {
            let _ = logger
                .log_search(&query.query, result.0.total_results)
                .await;
        }

        info!(
            "Search completed: {} results for '{}'",
            result.0.total_results, query.query
        );

        Ok(result)
    }

    async fn get_autocomplete(&self, prefix: &str, limit: usize) -> Result<Vec<AutocompleteItem>> {
        debug!("Getting autocomplete for prefix: {}", prefix);

        if prefix.trim().is_empty() {
            return Ok(Vec::new());
        }

        let suggestions = self.search_engine.suggest(prefix, limit).await?;

        debug!("Found {} suggestions", suggestions.len());
        Ok(suggestions)
    }

    async fn find_similar_items(
        &self,
        item_id: Uuid,
        limit: usize,
    ) -> Result<Vec<VocabularySearchItem>> {
        debug!("Finding similar items to: {}", item_id);

        let similar = self.search_engine.find_similar(item_id, limit).await?;

        info!("Found {} similar items", similar.len());
        Ok(similar)
    }

    async fn get_popular_searches(&self, limit: usize) -> Result<Vec<String>> {
        if let Some(logger) = &self.search_logger {
            logger.get_popular_queries(limit).await
        } else {
            Ok(Vec::new())
        }
    }
}
