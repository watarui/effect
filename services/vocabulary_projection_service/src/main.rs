use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};
use vocabulary_projection_service::{
    application::processor::EventProcessor,
    config::Config,
    infrastructure::{
        adapters::event_store_subscriber::EventStoreSubscriber,
        repositories::{
            postgres_projection_state::PostgresProjectionStateRepository,
            postgres_read_model::PostgresReadModelRepository,
        },
    },
    ports::inbound::EventProcessorUseCase,
};

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt::init();

    info!("===========================================");
    info!("Vocabulary Projection Service - 起動中");
    info!("責務: Event Store から Read Model への投影");
    info!("アーキテクチャ: ヘキサゴナル");
    info!("===========================================");

    // 設定読み込み
    let config = Config::from_env()?;
    info!("Configuration loaded");

    // データベース接続プールを作成
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    info!("Database connection established");

    // マイグレーションを実行
    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Database migrations completed");

    // インフラストラクチャ層の実装を作成
    let event_subscriber = EventStoreSubscriber::new(config.event_store.url.clone());
    let read_repository = PostgresReadModelRepository::new(pool.clone());
    let state_repository = PostgresProjectionStateRepository::new(pool);

    // アプリケーション層のサービスを作成
    let processor =
        EventProcessor::new(config, event_subscriber, read_repository, state_repository);

    // イベント処理ループを開始
    info!("Starting event processing loop");

    // Ctrl+C ハンドラと並行して実行
    tokio::select! {
        result = processor.start_processing() => {
            if let Err(e) = result {
                error!("Event processor error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
            processor.stop_processing().await?;
        }
    }

    info!("Vocabulary Projection Service shutting down");
    Ok(())
}
