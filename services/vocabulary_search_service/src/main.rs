//! Vocabulary Search Service
//!
//! Meilisearch を活用した高度な検索機能を提供する専門サービス

use std::{sync::Arc, time::Duration};

use meilisearch_sdk::client::Client;
use redis::aio::ConnectionManager;
use tracing::{error, info};
use vocabulary_search_service::{
    application::search_handlers::{
        FacetSearchHandlerImpl,
        SearchItemsHandlerImpl,
        SuggestionsHandlerImpl,
    },
    infrastructure::{
        cache::RedisCacheService,
        search::{
            index_config::configure_meilisearch_index,
            meilisearch_engine::MeilisearchEngine,
            query_analyzer::QueryAnalyzer,
        },
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // テレメトリの初期化
    let _tracer = shared_telemetry::init_telemetry("vocabulary_search_service", None)?;

    info!("Starting Vocabulary Search Service");

    // 環境変数から設定を読み込み
    let meilisearch_url =
        std::env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
    let meilisearch_api_key =
        std::env::var("MEILISEARCH_API_KEY").unwrap_or_else(|_| "masterKey".to_string());
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let index_name = std::env::var("INDEX_NAME").unwrap_or_else(|_| "vocabulary_items".to_string());

    // Meilisearch クライアントの初期化
    let meilisearch_client = Client::new(&meilisearch_url, Some(&meilisearch_api_key))
        .expect("Failed to create Meilisearch client");

    info!("Setting up Meilisearch index: {}", index_name);
    if let Err(e) = configure_meilisearch_index(&meilisearch_client, &index_name).await {
        error!("Failed to setup Meilisearch index: {}", e);
        // インデックス設定に失敗してもサービスは継続
    }

    // Redis 接続の初期化
    let redis_client = redis::Client::open(redis_url)?;
    let redis_conn = ConnectionManager::new(redis_client).await?;
    info!("Redis connection established");

    // 検索エンジンの初期化
    let search_engine = Arc::new(MeilisearchEngine::new(
        meilisearch_url,
        meilisearch_api_key,
        index_name,
    ));

    // クエリアナライザーの初期化
    let query_analyzer = Arc::new(QueryAnalyzer::default());

    // キャッシュサービスの初期化
    let cache_service = Arc::new(RedisCacheService::new(
        redis_conn,
        Duration::from_secs(300),
        "vocabulary_search",
    ));

    // TODO: ReadModelRepository の実装が必要
    // let read_model_repository = Arc::new(PostgresReadModelRepository::new(pool));

    // ハンドラーの初期化
    let _search_items_handler = Arc::new(SearchItemsHandlerImpl::new(
        search_engine.clone(),
        query_analyzer.clone(),
        cache_service.clone(),
    ));

    let _suggestions_handler = Arc::new(SuggestionsHandlerImpl::new(
        search_engine.clone(),
        cache_service.clone(),
    ));

    let _facet_search_handler = Arc::new(FacetSearchHandlerImpl::new(
        search_engine.clone(),
        query_analyzer.clone(),
    ));

    // TODO: ReadModelRepository が実装されたら有効化
    // let related_items_handler = Arc::new(RelatedItemsHandlerImpl::new(
    //     read_model_repository,
    //     cache_service.clone(),
    // ));

    // gRPC サーバーの設定
    let addr = "[::1]:50053";
    info!("Starting gRPC server on {}", addr);

    // TODO: gRPC サーバーの実装
    // let grpc_server = VocabularySearchGrpcServer::new(
    //     search_items_handler,
    //     suggestions_handler,
    //     facet_search_handler,
    //     related_items_handler,
    // );

    // Server::builder()
    //     .add_service(VocabularySearchServiceServer::new(grpc_server))
    //     .serve(addr.parse()?)
    //     .await?;

    // 一時的にシグナル待機のみ
    info!("Vocabulary Search Service is running (without gRPC server)");

    // シグナルハンドリング
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Vocabulary Search Service");

    Ok(())
}
