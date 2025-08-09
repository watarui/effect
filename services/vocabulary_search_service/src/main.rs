use anyhow::Result;
use tracing::info;

mod config;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt::init();

    info!("===========================================");
    info!("Vocabulary Search Service - 起動中");
    info!("責務: 全文検索と自動補完");
    info!("状態: 未実装（設計済み）");
    info!("===========================================");
    info!("");
    info!("Meilisearch を活用した高度な検索機能");
    info!("- Typo 許容検索");
    info!("- 部分一致検索");
    info!("- フィルタリング（status, CEFR レベル等）");
    info!("- 高速な自動補完");
    info!("");
    info!("詳細: docs/tactical/contexts/vocabulary/");
    info!("===========================================");

    // 設定読み込み
    let config = config::Config::from_env()?;

    // サーバー起動
    server::run(config).await?;

    Ok(())
}
