//! 基本的な検索項目ハンドラー

#![allow(clippy::map_flatten)]

use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use tracing::{debug, info};

use crate::{
    domain::{
        error::SearchError,
        search_models::{HighlightConfig, MeilisearchQuery, VocabularySearchDocument},
        value_objects::{Pagination, SearchQuery},
    },
    ports::{
        inbound::{SearchHandler, SearchItemsHandler},
        outbound::{CacheService, QueryAnalyzer, SearchEngine},
    },
    proto::{
        MatchExplanation,
        SearchItemsRequest,
        SearchItemsResponse,
        SearchMode,
        SearchResultItem,
        SpellingSuggestion,
    },
};

/// 検索項目ハンドラー実装
pub struct SearchItemsHandlerImpl<E, A, C>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    A: QueryAnalyzer,
    C: CacheService,
{
    search_engine: Arc<E>,
    analyzer:      Arc<A>,
    cache:         Arc<C>,
}

impl<E, A, C> SearchItemsHandlerImpl<E, A, C>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    A: QueryAnalyzer,
    C: CacheService,
{
    /// 新しいハンドラーを作成
    pub fn new(search_engine: Arc<E>, analyzer: Arc<A>, cache: Arc<C>) -> Self {
        Self {
            search_engine,
            analyzer,
            cache,
        }
    }

    /// 検索クエリを構築
    fn build_search_query(
        &self,
        analyzed: &crate::domain::search_models::AnalyzedQuery,
        filters: &Option<crate::proto::SearchFilters>,
        options: &Option<crate::proto::SearchOptions>,
    ) -> Result<MeilisearchQuery, SearchError> {
        let mode = options
            .as_ref()
            .and_then(|o| SearchMode::try_from(o.mode).ok())
            .unwrap_or(SearchMode::Fuzzy);

        // 検索モードに応じたクエリ文字列の構築
        let query_string = match mode {
            SearchMode::Exact => format!("\"{}\"", analyzed.normalized_query),
            SearchMode::Fuzzy => analyzed.normalized_query.clone(),
            SearchMode::Phrase => format!("\"{}\"", analyzed.normalized_query),
            SearchMode::Wildcard => format!("{}*", analyzed.normalized_query),
            SearchMode::Semantic => analyzed.normalized_query.clone(),
        };

        // フィルタ構築
        let mut filter_conditions = Vec::new();

        if let Some(filters) = filters {
            if !filters.part_of_speech.is_empty() {
                let pos_filter = filters
                    .part_of_speech
                    .iter()
                    .map(|pos| format!("part_of_speech = \"{pos}\""))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                filter_conditions.push(format!("({pos_filter})"));
            }

            if !filters.cefr_levels.is_empty() {
                let level_filter = filters
                    .cefr_levels
                    .iter()
                    .map(|level| format!("cefr_level = \"{level}\""))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                filter_conditions.push(format!("({level_filter})"));
            }

            if filters.ai_generated_only {
                filter_conditions.push("is_ai_generated = true".to_string());
            }
        }

        let filter = if filter_conditions.is_empty() {
            None
        } else {
            Some(filter_conditions.join(" AND "))
        };

        // ハイライト設定
        let highlight = options.as_ref().and_then(|o| {
            if o.highlight_tag.is_empty() {
                None
            } else {
                Some(HighlightConfig {
                    attributes: vec![
                        "spelling".to_string(),
                        "definitions".to_string(),
                        "examples".to_string(),
                    ],
                    pre_tag:    format!("<{}>", o.highlight_tag),
                    post_tag:   format!("</{}>", o.highlight_tag),
                })
            }
        });

        // ソート設定
        let sort = options.as_ref().and_then(|o| {
            o.sort_by
                .as_ref()
                .map(|s| {
                    let field = match s.field {
                        0 => return None, // RELEVANCE はデフォルト
                        1 => "spelling",
                        2 => "created_at",
                        3 => "updated_at",
                        4 => "popularity_score",
                        5 => "quality_score",
                        _ => return None,
                    };

                    let direction = if s.descending { "desc" } else { "asc" };
                    Some(vec![format!("{}:{}", field, direction)])
                })
                .flatten()
        });

        Ok(MeilisearchQuery {
            query_string,
            filter,
            highlight,
            sort,
        })
    }

    /// 検索結果をレスポンス形式に変換
    fn map_to_result_items(
        &self,
        hits: Vec<crate::domain::search_models::SearchHit<VocabularySearchDocument>>,
    ) -> Vec<SearchResultItem> {
        hits.into_iter()
            .map(|hit| SearchResultItem {
                item_id:        hit.document.item_id,
                entry_id:       hit.document.entry_id,
                spelling:       hit.document.spelling,
                disambiguation: Some(hit.document.disambiguation),
                score:          hit.score,
                highlights:     hit
                    .highlights
                    .into_iter()
                    .map(|(k, v)| (k, vec![v]))
                    .collect(),
                explanation:    Some(MatchExplanation {
                    field_matches: vec![],
                    total_score:   hit.score,
                }),
            })
            .collect()
    }

    /// スペリングサジェストを取得
    async fn get_spelling_suggestions(
        &self,
        _query: &str,
    ) -> Result<Vec<SpellingSuggestion>, SearchError> {
        // TODO: 実装
        Ok(vec![])
    }
}

#[async_trait]
impl<E, A, C> SearchHandler for SearchItemsHandlerImpl<E, A, C>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    A: QueryAnalyzer,
    C: CacheService,
{
    type Request = SearchItemsRequest;
    type Response = SearchItemsResponse;

    async fn handle(&self, request: Self::Request) -> Result<Self::Response, SearchError> {
        let start = Instant::now();

        info!("Handling search request: {}", request.query);

        // 1. クエリ検証
        let search_query = SearchQuery::new(&request.query)?;

        // 2. キャッシュチェック
        let cache_key = format!("search:{}", serde_json::to_string(&request)?);
        if let Ok(Some(cached)) = self.cache.get::<SearchItemsResponse>(&cache_key).await {
            debug!("Returning cached result");
            return Ok(cached);
        }

        // 3. クエリ分析
        let analyzed_query = self.analyzer.analyze(search_query.as_str()).await?;

        // 4. 検索クエリ構築
        let search_query =
            self.build_search_query(&analyzed_query, &request.filters, &request.options)?;

        // 5. ページネーション設定
        let pagination = if let Some(p) = request.pagination {
            Pagination::new(p.offset, p.limit)?
        } else {
            Pagination::default()
        };

        // 6. 検索実行
        let search_result = self.search_engine.search(search_query, pagination).await?;

        // 7. スペルチェック（結果がない場合）
        let suggestions = if search_result.total_hits == 0 {
            self.get_spelling_suggestions(&request.query).await?
        } else {
            vec![]
        };

        // 8. レスポンス構築
        let response = SearchItemsResponse {
            items: self.map_to_result_items(search_result.hits),
            total_hits: search_result.total_hits as u64,
            max_score: search_result.max_score,
            took_ms: start.elapsed().as_millis() as u32,
            suggestions,
        };

        // 9. キャッシュに保存
        let _ = self
            .cache
            .set(
                &cache_key,
                &response,
                Some(std::time::Duration::from_secs(300)),
            )
            .await;

        info!(
            "Search completed in {}ms with {} results",
            response.took_ms, response.total_hits
        );

        Ok(response)
    }
}

#[async_trait]
impl<E, A, C> SearchItemsHandler for SearchItemsHandlerImpl<E, A, C>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    A: QueryAnalyzer,
    C: CacheService,
{
}
