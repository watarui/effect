use std::{net::SocketAddr, sync::Arc};

use sqlx::PgPool;
use tonic::transport::Server;
use tracing::info;

use crate::{
    application::commands::{
        CreateVocabularyItemHandler,
        DeleteVocabularyItemHandler,
        UpdateVocabularyItemHandler,
    },
    config::Config,
    error::Result,
    infrastructure::{
        event_store::PostgresEventStore,
        grpc::service::{
            VocabularyCommandServiceImpl,
            proto::vocabulary_command_service_server::VocabularyCommandServiceServer,
        },
        repositories::{PostgresVocabularyEntryRepository, PostgresVocabularyItemRepository},
    },
};

/// gRPC サーバーを起動
pub async fn start_grpc_server(config: Config) -> Result<()> {
    // データベース接続プールを作成
    let db_pool = PgPool::connect(&config.database.url)
        .await
        .map_err(crate::error::Error::Database)?;

    let event_store_pool = PgPool::connect(&config.event_store.url)
        .await
        .map_err(crate::error::Error::Database)?;

    // リポジトリとイベントストアを初期化
    let entry_repo = PostgresVocabularyEntryRepository::new(db_pool.clone());
    let item_repo = PostgresVocabularyItemRepository::new(db_pool.clone());
    let event_store = PostgresEventStore::new(event_store_pool);

    // コマンドハンドラーを初期化
    let create_handler = Arc::new(CreateVocabularyItemHandler::new(
        entry_repo.clone(),
        item_repo.clone(),
        event_store.clone(),
    ));

    let update_handler = Arc::new(UpdateVocabularyItemHandler::new(
        item_repo.clone(),
        event_store.clone(),
    ));

    let delete_handler = Arc::new(DeleteVocabularyItemHandler::new(
        entry_repo,
        item_repo,
        event_store,
    ));

    // gRPC サービスを作成
    let grpc_service =
        VocabularyCommandServiceImpl::new(create_handler, update_handler, delete_handler);

    // gRPC サーバーアドレス
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .map_err(|e| crate::error::Error::Config(format!("Invalid server address: {}", e)))?;

    info!("Starting gRPC server on {}", addr);

    // gRPC サーバーを起動
    Server::builder()
        .add_service(VocabularyCommandServiceServer::new(grpc_service))
        .serve(addr)
        .await
        .map_err(|e| crate::error::Error::Internal(format!("gRPC server error: {}", e)))?;

    Ok(())
}
