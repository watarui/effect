//! インバウンドポート（ユースケースインターフェース）

use async_trait::async_trait;

use crate::{
    domain::error::SearchError,
    proto::{
        GetRelatedItemsRequest,
        GetRelatedItemsResponse,
        GetSuggestionsRequest,
        GetSuggestionsResponse,
        SearchItemsRequest,
        SearchItemsResponse,
        SearchWithFacetsRequest,
        SearchWithFacetsResponse,
    },
};

/// 検索ハンドラー trait
#[async_trait]
pub trait SearchHandler: Send + Sync {
    type Request;
    type Response;

    /// リクエストを処理
    async fn handle(&self, request: Self::Request) -> Result<Self::Response, SearchError>;
}

/// 検索項目ハンドラー
#[async_trait]
pub trait SearchItemsHandler:
    SearchHandler<Request = SearchItemsRequest, Response = SearchItemsResponse>
{
}

/// サジェストハンドラー
#[async_trait]
pub trait GetSuggestionsHandler:
    SearchHandler<Request = GetSuggestionsRequest, Response = GetSuggestionsResponse>
{
}

/// ファセット検索ハンドラー
#[async_trait]
pub trait SearchWithFacetsHandler:
    SearchHandler<Request = SearchWithFacetsRequest, Response = SearchWithFacetsResponse>
{
}

/// 関連項目ハンドラー
#[async_trait]
pub trait GetRelatedItemsHandler:
    SearchHandler<Request = GetRelatedItemsRequest, Response = GetRelatedItemsResponse>
{
}
