//! サジェストハンドラー

use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;

use crate::{
    domain::{
        error::SearchError,
        search_models::{MeilisearchQuery, VocabularySearchDocument},
        value_objects::Pagination,
    },
    ports::{
        inbound::{GetSuggestionsHandler, SearchHandler},
        outbound::{CacheService, SearchEngine},
    },
    proto::{GetSuggestionsRequest, GetSuggestionsResponse, Suggestion, SuggestionType},
};

/// サジェストハンドラー実装
pub struct SuggestionsHandlerImpl<E, C>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    C: CacheService,
{
    search_engine: Arc<E>,
    cache:         Arc<C>,
}

impl<E, C> SuggestionsHandlerImpl<E, C>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    C: CacheService,
{
    /// 新しいハンドラーを作成
    pub fn new(search_engine: Arc<E>, cache: Arc<C>) -> Self {
        Self {
            search_engine,
            cache,
        }
    }
}

#[async_trait]
impl<E, C> SearchHandler for SuggestionsHandlerImpl<E, C>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    C: CacheService,
{
    type Request = GetSuggestionsRequest;
    type Response = GetSuggestionsResponse;

    async fn handle(&self, request: Self::Request) -> Result<Self::Response, SearchError> {
        info!(
            "Handling suggestions request for prefix: {}",
            request.prefix
        );

        // キャッシュチェック
        let cache_key = format!(
            "suggest:{}:{}:{}",
            request.prefix, request.limit, request.r#type
        );
        if let Ok(Some(cached)) = self.cache.get::<GetSuggestionsResponse>(&cache_key).await {
            return Ok(cached);
        }

        let suggestion_type =
            SuggestionType::try_from(request.r#type).unwrap_or(SuggestionType::Spelling);

        // 検索フィールドを決定
        let _search_field = match suggestion_type {
            SuggestionType::Spelling => "spelling",
            SuggestionType::Definition => "definitions",
            SuggestionType::Example => "examples",
        };

        // プレフィックス検索クエリを構築
        let query = MeilisearchQuery {
            query_string: format!("{}*", request.prefix),
            filter:       None,
            highlight:    None,
            sort:         Some(vec!["popularity_score:desc".to_string()]),
        };

        let pagination = Pagination::new(0, request.limit)?;

        // 検索実行
        let result = self.search_engine.search(query, pagination).await?;

        // 結果をサジェストに変換
        let suggestions = result
            .hits
            .into_iter()
            .map(|hit| {
                let text = match suggestion_type {
                    SuggestionType::Spelling => hit.document.spelling.clone(),
                    SuggestionType::Definition => hit
                        .document
                        .definitions
                        .first()
                        .cloned()
                        .unwrap_or_default(),
                    SuggestionType::Example => {
                        hit.document.examples.first().cloned().unwrap_or_default()
                    },
                };

                Suggestion {
                    text:         text.clone(),
                    display_text: text,
                    score:        hit.score,
                    r#type:       request.r#type,
                }
            })
            .collect();

        let response = GetSuggestionsResponse { suggestions };

        // キャッシュに保存
        let _ = self
            .cache
            .set(
                &cache_key,
                &response,
                Some(std::time::Duration::from_secs(600)),
            )
            .await;

        Ok(response)
    }
}

#[async_trait]
impl<E, C> GetSuggestionsHandler for SuggestionsHandlerImpl<E, C>
where
    E: SearchEngine<Document = VocabularySearchDocument>,
    C: CacheService,
{
}
