//! gRPC サーバー実装

use std::net::SocketAddr;

use algorithm_service::{
    infrastructure::grpc::service::AlgorithmServiceImpl,
    proto::effect::services::algorithm::algorithm_service_server::AlgorithmServiceServer,
};
use tonic::transport::Server;
use tracing::info;

use crate::config::ServiceConfig;

/// gRPC サーバーを起動
///
/// # Errors
///
/// サーバーの起動に失敗した場合、エラーを返します
pub async fn start(config: ServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse()?;

    info!("Algorithm Service listening on {}", addr);

    // データベース接続プールの作成
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    info!("Connected to database");

    // サービスの作成
    let service = AlgorithmServiceImpl::new(db_pool);
    let algorithm_server = AlgorithmServiceServer::new(service);

    // サーバーの起動
    Server::builder()
        .add_service(algorithm_server)
        .serve(addr)
        .await?;

    info!("Algorithm Service shutting down...");

    Ok(())
}
