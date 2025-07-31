//! Learning Service
//!
//! 学習セッションの管理と学習フローの制御を提供するマイクロサービス

use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

mod config;
mod server;

use config::ServiceConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログ初期化
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .json()
        .init();

    info!("Starting Learning Service...");

    // 設定読み込み
    let config = match ServiceConfig::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        },
    };
    info!("Configuration loaded");

    // gRPCサーバー起動
    server::start(config).await?;

    Ok(())
}
