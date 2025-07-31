//! Event Processor アプリケーション
//!
//! ドメインイベントを処理し、各コンテキストに配信するアプリケーション

use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログ初期化
    tracing_subscriber::fmt::init();

    info!("Event Processor starting...");

    // TODO: イベントバスの初期化
    // TODO: イベントハンドラーの登録
    // TODO: イベント処理ループの開始

    info!("Event Processor started");

    // シグナルハンドリング
    tokio::signal::ctrl_c().await?;

    info!("Event Processor shutting down...");

    Ok(())
}
