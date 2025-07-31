//! gRPC サーバー実装

use std::net::SocketAddr;

use tracing::info;

use crate::config::ServiceConfig;

/// gRPC サーバーを起動
pub async fn start(config: ServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse()?;

    info!("Vocabulary Service listening on {}", addr);

    // TODO: gRPC サーバーの実装
    // 現時点では、シグナルを待つだけ
    tokio::signal::ctrl_c().await?;

    info!("Vocabulary Service shutting down...");

    Ok(())
}
