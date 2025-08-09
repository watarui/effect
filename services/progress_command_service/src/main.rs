use tracing::info;

mod config;
mod error;
mod server;

use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt::init();

    info!("===========================================");
    info!("Progress Command Service - 起動中");
    info!("責務: イベント受信と Event Store への永続化");
    info!("状態: 未実装（設計済み）");
    info!("===========================================");
    info!("");
    info!("純粋な CQRS/Event Sourcing の Write 側");
    info!("- 他コンテキストからのイベント受信");
    info!("- Event Store への永続化");
    info!("- イベント順序保証");
    info!("- Pub/Sub へのイベント配信");
    info!("");
    info!("詳細: docs/tactical/contexts/progress/");
    info!("===========================================");

    // 設定読み込み
    let config = config::Config::from_env()?;

    // サーバー起動
    server::run(config).await?;

    Ok(())
}
