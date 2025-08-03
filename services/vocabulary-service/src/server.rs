//! gRPC サーバー実装

use std::net::SocketAddr;

use infrastructure::database::Config as DatabaseConfig;
use tracing::info;

use crate::config::ServiceConfig;

/// gRPC サーバーを起動
pub async fn start(config: ServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse()?;

    info!("Vocabulary Service listening on {}", addr);

    // データベース接続プールを作成
    let db_config = DatabaseConfig {
        url:                  config.database_url.clone(),
        max_connections:      10,
        min_connections:      2,
        connect_timeout_secs: 30,
        idle_timeout_secs:    600,
    };
    let db_pool = infrastructure::database::create_pool(&db_config).await?;

    // Redis クライアントを作成
    let cache_client = infrastructure::cache::Client::new(&config.redis_url).await?;

    // PostgreSQL リポジトリを作成
    let postgres_repo =
        vocabulary_service::adapters::outbound::repository::postgres::Repository::new(db_pool);

    // キャッシュ付きリポジトリを作成
    let cached_repo = vocabulary_service::adapters::outbound::repository::cached::Repository::new(
        postgres_repo,
        cache_client,
    );

    // アプリケーションサービスを作成
    let app_service =
        vocabulary_service::application::services::VocabularyService::new(cached_repo);

    // gRPC サービスを作成
    let _grpc_service =
        vocabulary_service::adapters::inbound::grpc::VocabularyGrpcService::new(app_service);

    // TODO: gRPC サーバーの起動
    // 現時点では、proto ファイルからの生成が必要

    info!("Vocabulary Service is ready to handle requests");

    // シグナルを待つ
    tokio::signal::ctrl_c().await?;

    info!("Vocabulary Service shutting down...");

    Ok(())
}
