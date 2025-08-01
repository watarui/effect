//! User Service メインエントリーポイント

use std::sync::Arc;

use tonic::transport::Server;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;
use user_service::{
    adapters::{
        inbound::grpc::{
            converters::proto::services::user::user_service_server::UserServiceServer,
            user_service::UserServiceImpl,
        },
        outbound::{
            auth::mock::Provider as MockAuthProvider,
            event::memory::InMemoryPublisher,
            repository::memory::InMemoryRepository,
        },
    },
    application::use_cases::UseCaseImpl,
    config::Config,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ロギング設定
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| format!("Failed to set default subscriber: {e}"))?;

    // 設定読み込み
    let config = Config::load().map_err(|e| format!("Failed to load configuration: {e}"))?;

    let grpc_port = config.server.port;

    // 依存関係の初期化
    let repository = Arc::new(InMemoryRepository::new());
    let event_publisher = Arc::new(InMemoryPublisher::new());
    let auth_provider = Arc::new(MockAuthProvider::new());

    // ユースケースの初期化
    let use_case = Arc::new(UseCaseImpl::new(repository, event_publisher, auth_provider));

    // gRPC サービスの初期化
    let grpc_service = UserServiceImpl::new(use_case);
    let grpc_server = UserServiceServer::new(grpc_service);

    let addr = format!("0.0.0.0:{grpc_port}").parse()?;
    info!("User Service starting on {}", addr);

    // サーバー起動
    Server::builder()
        .add_service(grpc_server)
        .serve(addr)
        .await?;

    Ok(())
}
