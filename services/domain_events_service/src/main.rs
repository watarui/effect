//! Domain Events Service - イベントスキーマの中央管理サービス

use tracing::info;

mod config;
mod grpc;
mod registry;
mod schemas;
mod validator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // トレーシング初期化
    shared_telemetry::init_telemetry("domain_events_service", None)?;

    info!("===========================================");
    info!("Domain Events Service - 起動中");
    info!("責務: イベントスキーマの中央管理");
    info!("===========================================");
    info!("");
    info!("Schema Registry パターンの実装");
    info!("- 全イベントタイプの定義管理");
    info!("- スキーマバージョニング");
    info!("- イベント検証サービス");
    info!("- Event Store Service との連携");
    info!("");
    info!("===========================================");

    // 設定読み込み
    let config = config::load()?;
    info!("Configuration loaded");

    // データベース接続
    let db_config = shared_database::Config {
        url:                  config.database_url.clone(),
        max_connections:      10,
        min_connections:      2,
        connect_timeout_secs: 30,
        idle_timeout_secs:    600,
    };
    let pool = shared_database::create_pool(&db_config).await?;
    info!("Database connected");

    // マイグレーション実行
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations completed");

    // スキーマレジストリ初期化
    let registry = registry::Registry::new(pool.clone(), config.registry.clone());
    info!("Schema registry initialized");

    // バリデーター初期化
    let validator = validator::Validator::new(registry.clone());
    info!("Event validator initialized");

    // gRPC サーバー起動
    info!("Starting gRPC server on port {}", config.grpc.port);
    grpc::start_server(config, registry, validator).await?;

    Ok(())
}
