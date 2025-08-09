use anyhow::Result;
use tracing::info;

mod config;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt::init();

    info!("===========================================");
    info!("Vocabulary Query Service - 起動中");
    info!("責務: 高速な読み取り処理");
    info!("状態: 未実装（設計済み）");
    info!("===========================================");
    info!("");
    info!("CQRS + Event Sourcing の Read 側を担当");
    info!("- Read Model からの語彙情報取得");
    info!("- Redis キャッシュによる高速化");
    info!("- GraphQL への応答");
    info!("");
    info!("詳細: docs/tactical/contexts/vocabulary/");
    info!("===========================================");

    // 設定読み込み
    let config = config::Config::from_env()?;

    // サーバー起動
    server::run(config).await?;

    Ok(())
}
