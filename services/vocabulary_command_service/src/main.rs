//! Vocabulary Command Service
//!
//! CQRS+ES パターンにおける Write Model を担当するサービス。
//! コマンドの処理、ドメインイベントの生成、Event Store への永続化を行う。

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tracing::info;
use vocabulary_command_service::{
    application::CommandHandler,
    infrastructure::outbound::{
        event_store::PostgresEventStore,
        pubsub::PubSubEventBus,
        repository::EventStoreVocabularyRepository,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // テレメトリの初期化
    let _tracer = shared_telemetry::init_telemetry("vocabulary_command_service", None)?;

    info!("Starting Vocabulary Command Service");

    // データベース接続
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://effect:effect_password@localhost:5434/effect".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Event Store 初期化
    let event_store = Arc::new(PostgresEventStore::new(pool.clone()));

    // Event Bus 初期化
    let event_bus = Arc::new(PubSubEventBus::new());

    // Repository 初期化
    let repository = Arc::new(EventStoreVocabularyRepository::new(
        event_store.clone(),
        event_bus.clone(),
        pool.clone(),
    ));

    // Command Handler 初期化
    let _command_handler = Arc::new(CommandHandler::new(event_store, event_bus, repository));

    // gRPC サーバー起動
    let addr = "[::1]:50051";

    info!("Starting gRPC server on {}", addr);

    // TODO: proto が生成されたら有効化
    // Server::builder()
    //     .add_service(vocabulary_command_service_server::VocabularyCommandServiceServer::new(grpc_service))
    //     .serve(addr)
    //     .await?;

    // シグナルハンドリング
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Vocabulary Command Service");

    Ok(())
}
