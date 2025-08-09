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
    info!("Progress Query Service - 起動中");
    info!("責務: 読み取りモデルの提供（GraphQL API）");
    info!("状態: 未実装（設計済み）");
    info!("===========================================");
    info!("");
    info!("純粋な CQRS/Event Sourcing の Read 側");
    info!("- GraphQL スキーマ定義");
    info!("- Read Model からのデータ取得");
    info!("- 効率的なクエリ実行");
    info!("- DataLoader による N+1 問題対策");
    info!("");
    info!("詳細: docs/tactical/contexts/progress/");
    info!("===========================================");

    // 設定読み込み
    let config = config::Config::from_env()?;

    // サーバー起動
    server::run(config).await?;

    Ok(())
}
