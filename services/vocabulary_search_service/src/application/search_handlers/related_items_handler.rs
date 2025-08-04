//! 関連項目ハンドラー

use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;

use crate::{
    domain::error::SearchError,
    ports::{
        inbound::{GetRelatedItemsHandler, SearchHandler},
        outbound::{CacheService, ReadModelRepository, RelationType as DomainRelationType},
    },
    proto::{GetRelatedItemsRequest, GetRelatedItemsResponse, RelatedItem, RelationType},
};

/// 関連項目ハンドラー実装
pub struct RelatedItemsHandlerImpl<R, C>
where
    R: ReadModelRepository,
    C: CacheService,
{
    repository: Arc<R>,
    cache:      Arc<C>,
}

impl<R, C> RelatedItemsHandlerImpl<R, C>
where
    R: ReadModelRepository,
    C: CacheService,
{
    /// 新しいハンドラーを作成
    pub fn new(repository: Arc<R>, cache: Arc<C>) -> Self {
        Self { repository, cache }
    }

    /// Proto の RelationType を Domain の RelationType に変換
    fn convert_relation_type(&self, proto_type: i32) -> DomainRelationType {
        match RelationType::try_from(proto_type).unwrap_or(RelationType::Synonyms) {
            RelationType::Synonyms => DomainRelationType::Synonyms,
            RelationType::Antonyms => DomainRelationType::Antonyms,
            RelationType::SimilarUsage => DomainRelationType::SimilarUsage,
            RelationType::SameDomain => DomainRelationType::SameDomain,
            RelationType::SameLevel => DomainRelationType::SameLevel,
        }
    }
}

#[async_trait]
impl<R, C> SearchHandler for RelatedItemsHandlerImpl<R, C>
where
    R: ReadModelRepository,
    C: CacheService,
{
    type Request = GetRelatedItemsRequest;
    type Response = GetRelatedItemsResponse;

    async fn handle(&self, request: Self::Request) -> Result<Self::Response, SearchError> {
        info!(
            "Handling related items request for item: {} with type: {}",
            request.item_id, request.relation_type
        );

        // キャッシュチェック
        let cache_key = format!(
            "related:{}:{}:{}",
            request.item_id, request.relation_type, request.limit
        );

        if let Ok(Some(cached)) = self.cache.get::<GetRelatedItemsResponse>(&cache_key).await {
            return Ok(cached);
        }

        // リポジトリから関連項目を取得
        let domain_relation_type = self.convert_relation_type(request.relation_type);
        let related_items = self
            .repository
            .get_related_items(
                &request.item_id,
                domain_relation_type,
                request.limit as usize,
            )
            .await?;

        // レスポンス形式に変換
        let items = related_items
            .into_iter()
            .map(|item| RelatedItem {
                item_id:        item.item_id,
                spelling:       item.spelling,
                disambiguation: Some(item.disambiguation),
                relation_score: item.relation_score,
                relation_type:  request.relation_type,
            })
            .collect();

        let response = GetRelatedItemsResponse { items };

        // キャッシュに保存
        let _ = self
            .cache
            .set(
                &cache_key,
                &response,
                Some(std::time::Duration::from_secs(3600)),
            )
            .await;

        Ok(response)
    }
}

#[async_trait]
impl<R, C> GetRelatedItemsHandler for RelatedItemsHandlerImpl<R, C>
where
    R: ReadModelRepository,
    C: CacheService,
{
}
