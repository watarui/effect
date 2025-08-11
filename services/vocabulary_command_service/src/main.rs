use tracing::info;
use vocabulary_command_service::{config, error::Result};

mod server;

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt::init();

    info!("===========================================");
    info!("Vocabulary Command Service - 起動中");
    info!("責務: コマンド処理とドメインロジック");
    info!("状態: 未実装（設計済み）");
    info!("===========================================");
    info!("");
    info!("CQRS + Event Sourcing の Write 側を担当");
    info!("- 語彙項目の作成・更新・削除");
    info!("- ビジネスルールの検証");
    info!("- Event Store への永続化");
    info!("");
    info!("詳細: docs/tactical/contexts/vocabulary/");
    info!("===========================================");

    // 設定読み込み
    let config = config::Config::from_env()?;

    // サーバー起動
    server::run(config).await?;

    Ok(())
}
