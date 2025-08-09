use anyhow::Result;
use tracing::info;

mod config;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt::init();

    info!("===========================================");
    info!("Vocabulary Service - 起動中");
    info!("状態: 未実装（再設計中）");
    info!("===========================================");
    info!("");
    info!("このサービスは現在再設計中です。");
    info!("設計が確定次第、実装を開始します。");
    info!("");
    info!("詳細: docs/tactical/contexts/vocabulary/");
    info!("===========================================");

    // 設定読み込み
    let config = config::Config::from_env()?;

    // サーバー起動（最小限の実装）
    server::run(config).await?;

    Ok(())
}
