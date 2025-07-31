//! User Service
//!
//! ユーザー管理を提供するマイクロサービス

mod config;
mod server;

use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログ初期化
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .json()
        .init();

    info!("Starting User Service...");

    // 設定読み込み
    let config = match config::ServiceConfig::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        },
    };
    info!(
        "Running in {:?} mode on port {}",
        config.environment, config.port
    );

    // サーバー起動
    server::start(config).await?;

    Ok(())
}
