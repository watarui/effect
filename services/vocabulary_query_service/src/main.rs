//! Vocabulary Query Service
//!
//! CQRS+ES パターンにおける Read Model を提供するサービス。
//! 基本的な読み取り操作とシンプルなフィルタリングを行う。

use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::PgPool;
use tracing::info;
use vocabulary_query_service::{
    application::query_handlers::{GetEntryHandler, GetItemHandler, GetStatsHandler},
    infrastructure::{
        inbound::grpc::VocabularyQueryGrpcServer,
        outbound::{postgres::PostgresReadModelRepository, redis::RedisCacheService},
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // テレメトリの初期化
    let _tracer = shared_telemetry::init_telemetry("vocabulary_query_service", None)?;

    info!("Starting Vocabulary Query Service");

    // 設定の読み込み
    // TODO: 実際の設定読み込みを実装
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://effect:effect_password@localhost:5432/effect".to_string());
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

    // データベース接続プールの作成
    let pool = PgPool::connect(&database_url).await?;
    info!("Database connection pool created");

    // Redis 接続の作成
    let redis_client = redis::Client::open(redis_url)?;
    let redis_conn = ConnectionManager::new(redis_client).await?;
    info!("Redis connection established");

    // リポジトリとサービスの作成
    let repository = Arc::new(PostgresReadModelRepository::new(pool));
    let cache = Arc::new(RedisCacheService::new(redis_conn));

    // ハンドラーの作成
    let get_item_handler = Arc::new(GetItemHandler::new(repository.clone(), cache.clone()));
    let get_entry_handler = Arc::new(GetEntryHandler::new(repository.clone(), cache.clone()));
    let get_stats_handler = Arc::new(GetStatsHandler::new(repository, cache));

    // gRPC サーバーの作成
    let _grpc_server =
        VocabularyQueryGrpcServer::new(get_item_handler, get_entry_handler, get_stats_handler);

    // サーバーアドレスの設定
    let addr = "[::1]:50053";
    info!("gRPC server will be listening on {}", addr);

    // TODO: gRPC サーバーの起動を実装
    // 現在は一時的にシグナル待ちのみ
    info!("Vocabulary Query Service is running (without gRPC server)");

    // シグナルハンドリング
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Vocabulary Query Service");

    Ok(())
}
