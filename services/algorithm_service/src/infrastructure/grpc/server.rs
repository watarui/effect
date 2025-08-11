use std::net::SocketAddr;

use tonic::transport::Server;
use tracing::info;

use super::service::AlgorithmServiceImpl;
use crate::proto::effect::services::algorithm::algorithm_service_server::AlgorithmServiceServer;

/// gRPC サーバーを起動
///
/// # Errors
///
/// サーバーの起動に失敗した場合、エラーを返します
pub async fn start(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Algorithm Service gRPC server on {}", addr);

    // データベース接続（TODO: 実際のデータベース設定を追加）
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://effect:effect_password@localhost:5432/effect_test".to_string()
    });
    let db_pool = sqlx::PgPool::connect(&database_url).await?;

    // サービスの作成
    let service = AlgorithmServiceImpl::new(db_pool);
    let algorithm_server = AlgorithmServiceServer::new(service);

    // サーバーの起動
    Server::builder()
        .add_service(algorithm_server)
        .serve(addr)
        .await?;

    Ok(())
}
