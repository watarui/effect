//! ファセット検索ハンドラー

use std::{collections::HashMap, sync::Arc, time::Instant};

use async_trait::async_trait;
use tracing::info;

use crate::{
    domain::{
        error::SearchError,
        search_models::{MeilisearchQuery, VocabularySearchDocument},
        value_objects::{Pagination, SearchQuery},
    },
    ports::{
        inbound::{SearchHandler, SearchWithFacetsHandler},
        outbound::{QueryAnalyzer, SearchEngine},
    },
    proto::{
        FacetDistribution,
        SearchResultItem,
        SearchWithFacetsRequest,
        SearchWithFacetsResponse,
    },
};

/// ファセット検索ハンドラー実装
pub struct FacetSearchHandlerImpl<E, A>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    A: QueryAnalyzer,
{
    search_engine: Arc<E>,
    analyzer:      Arc<A>,
}

impl<E, A> FacetSearchHandlerImpl<E, A>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    A: QueryAnalyzer,
{
    /// 新しいハンドラーを作成
    pub fn new(search_engine: Arc<E>, analyzer: Arc<A>) -> Self {
        Self {
            search_engine,
            analyzer,
        }
    }

    /// フィルタを構築
    fn build_filter(&self, filters: &Option<crate::proto::SearchFilters>) -> Option<String> {
        let mut conditions = Vec::new();

        if let Some(filters) = filters {
            if !filters.part_of_speech.is_empty() {
                let pos_filter = filters
                    .part_of_speech
                    .iter()
                    .map(|pos| format!("part_of_speech = \"{pos}\""))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                conditions.push(format!("({pos_filter})"));
            }

            if !filters.cefr_levels.is_empty() {
                let level_filter = filters
                    .cefr_levels
                    .iter()
                    .map(|level| format!("cefr_level = \"{level}\""))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                conditions.push(format!("({level_filter})"));
            }

            if !filters.domains.is_empty() {
                let domain_filter = filters
                    .domains
                    .iter()
                    .map(|domain| format!("domain = \"{domain}\""))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                conditions.push(format!("({domain_filter})"));
            }

            if !filters.tags.is_empty() {
                let tags_filter = filters
                    .tags
                    .iter()
                    .map(|tag| format!("tags = \"{tag}\""))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                conditions.push(format!("({tags_filter})"));
            }
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }
}

#[async_trait]
impl<E, A> SearchHandler for FacetSearchHandlerImpl<E, A>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    A: QueryAnalyzer,
{
    type Request = SearchWithFacetsRequest;
    type Response = SearchWithFacetsResponse;

    async fn handle(&self, request: Self::Request) -> Result<Self::Response, SearchError> {
        let start = Instant::now();

        info!("Handling facet search request: {}", request.query);

        // クエリ検証
        let search_query = SearchQuery::new(&request.query)?;

        // クエリ分析
        let analyzed = self.analyzer.analyze(search_query.as_str()).await?;

        // 検索クエリ構築
        let query = MeilisearchQuery {
            query_string: analyzed.normalized_query,
            filter:       self.build_filter(&request.filters),
            highlight:    None,
            sort:         None,
        };

        // ページネーション設定
        let pagination = if let Some(p) = request.pagination {
            Pagination::new(p.offset, p.limit)?
        } else {
            Pagination::default()
        };

        // 検索実行
        // TODO: Meilisearch のファセット機能を使用するように修正
        let result = self.search_engine.search(query, pagination).await?;

        // 結果をレスポンス形式に変換
        let items = result
            .hits
            .into_iter()
            .map(|hit| SearchResultItem {
                item_id:        hit.document.item_id,
                entry_id:       hit.document.entry_id,
                spelling:       hit.document.spelling,
                disambiguation: Some(hit.document.disambiguation),
                score:          hit.score,
                highlights:     HashMap::new(),
                explanation:    None,
            })
            .collect();

        // TODO: 実際のファセット集計結果を使用
        let mut facets = HashMap::new();

        if request.facets.contains(&"part_of_speech".to_string()) {
            facets.insert(
                "part_of_speech".to_string(),
                FacetDistribution {
                    values: HashMap::new(),
                },
            );
        }

        if request.facets.contains(&"cefr_level".to_string()) {
            facets.insert(
                "cefr_level".to_string(),
                FacetDistribution {
                    values: HashMap::new(),
                },
            );
        }

        let response = SearchWithFacetsResponse {
            items,
            total_hits: result.total_hits as u64,
            max_score: result.max_score,
            took_ms: start.elapsed().as_millis() as u32,
            suggestions: vec![],
            facets,
        };

        info!(
            "Facet search completed in {}ms with {} results",
            response.took_ms, response.total_hits
        );

        Ok(response)
    }
}

#[async_trait]
impl<E, A> SearchWithFacetsHandler for FacetSearchHandlerImpl<E, A>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    A: QueryAnalyzer,
{
}
