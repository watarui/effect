use std::{sync::Arc, time::Duration};

use sqlx::PgPool;
use tokio::time;
use tracing::{error, info};

use crate::{
    application::{EventProcessorService, ProgressEventHandler},
    config::Config,
    infrastructure::repositories::{
        PostgresEventStoreReader,
        PostgresProjectionStateStore,
        PostgresReadModelRepository,
    },
    ports::inbound::EventProcessor,
};

pub async fn run(config: Config) -> crate::error::Result<()> {
    info!("イベントプロセッサー開始");
    info!("バッチサイズ: {}", config.processor.batch_size);
    info!("ポーリング間隔: {}ms", config.processor.poll_interval_ms);

    // データベース接続プールを作成
    let event_store_pool = PgPool::connect(&config.database.event_store_url).await?;
    let read_model_pool = PgPool::connect(&config.database.read_model_url).await?;

    // リポジトリを作成
    let event_store_reader = Arc::new(PostgresEventStoreReader::new(event_store_pool.clone()));
    let state_store = Arc::new(PostgresProjectionStateStore::new(read_model_pool.clone()));
    let read_model_repository = Arc::new(PostgresReadModelRepository::new(read_model_pool.clone()));

    // イベントハンドラーを作成
    let event_handler = Arc::new(ProgressEventHandler::new(read_model_repository));

    // イベントプロセッサーを作成
    let processor = EventProcessorService::new(
        event_store_reader,
        state_store,
        event_handler,
        config.processor.batch_size,
    );

    // イベント処理ループ
    let mut interval = time::interval(Duration::from_millis(config.processor.poll_interval_ms));

    loop {
        interval.tick().await;

        match processor.process_events().await {
            Ok(_) => {},
            Err(e) => {
                error!("イベント処理エラー: {}", e);
            },
        }
    }
}
