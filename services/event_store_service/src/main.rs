//! Event Store Service - イベントストアの中央管理サービス

use tracing::info;

mod config;
mod event_bus;
mod grpc;
mod repository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // トレーシング初期化
    shared_telemetry::init_telemetry("event_store_service", None)?;

    info!("Starting Event Store Service");

    // 設定読み込み
    let config = config::load()?;

    // データベース接続
    let db_config = shared_database::Config {
        url:                  config.database_url.clone(),
        max_connections:      10,
        min_connections:      2,
        connect_timeout_secs: 30,
        idle_timeout_secs:    600,
    };
    let pool = shared_database::create_pool(&db_config).await?;

    // マイグレーション実行
    sqlx::migrate!("./migrations").run(&pool).await?;

    // リポジトリ作成
    let repository = repository::PostgresEventStore::new(pool.clone());

    // Event Bus 初期化
    let event_bus = event_bus::EventBus::new(config.event_bus.clone()).await?;
    info!("Event Bus (Pub/Sub) initialized");

    // gRPC サーバー起動
    grpc::start_server(config, repository, event_bus).await?;

    Ok(())
}
