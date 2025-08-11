//! Algorithm Service
//!
//! 学習アルゴリズム管理を提供するマイクロサービス
//!
//! # 概要
//!
//! このサービスは SM-2
//! アルゴリズムを実装し、効果的な学習スケジューリングを提供します。
//! gRPC を通じて他のマイクロサービスと通信し、
//! 学習進度の計算と最適化を行います。

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

    info!("Starting Algorithm Service...");

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
