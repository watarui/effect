//! Vocabulary Projection Service
//!
//! CQRS+ES パターンにおける Read Model を構築するサービス。
//! Event Store からのイベントを購読し、プロジェクションを生成・更新する。

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tracing::info;
use vocabulary_projection_service::{
    application::event_handler::VocabularyEventHandler,
    infrastructure::outbound::{
        postgres::{PostgresProjectionStateRepository, PostgresReadModelRepository},
        pubsub::PubSubSubscriber,
    },
    ports::outbound::EventSubscriber,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // テレメトリの初期化
    let _tracer = shared_telemetry::init_telemetry("vocabulary_projection_service", None)?;

    info!("Starting Vocabulary Projection Service");

    // PostgreSQL 接続プールを作成
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://effect:effect_password@localhost:5434/effect_vocabulary".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // リポジトリを作成
    let read_model_repo = Arc::new(PostgresReadModelRepository::new(pool.clone()));
    let projection_repo = Arc::new(PostgresProjectionStateRepository::new(pool));

    // イベントハンドラーを作成
    let event_handler = Arc::new(VocabularyEventHandler::new(
        read_model_repo,
        projection_repo,
    ));

    // Pub/Sub サブスクライバーを作成
    let subscription_name = std::env::var("PUBSUB_SUBSCRIPTION")
        .unwrap_or_else(|_| "vocabulary-events-projection".to_string());

    let subscriber = PubSubSubscriber::new(subscription_name, event_handler).await?;

    info!("Vocabulary Projection Service is running");

    // サブスクリプションを別タスクで開始
    let subscriber = Arc::new(subscriber);
    let subscriber_task = {
        let subscriber = Arc::clone(&subscriber);
        tokio::spawn(async move {
            if let Err(e) = subscriber.subscribe().await {
                tracing::error!("Subscriber error: {}", e);
            }
        })
    };

    // シグナルハンドリング
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Vocabulary Projection Service");

    // サブスクライバーを停止
    subscriber_task.abort();

    Ok(())
}
