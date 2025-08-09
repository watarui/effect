use anyhow::Result;
use tracing::info;

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt::init();

    info!("===========================================");
    info!("Vocabulary Projection Service - 起動中");
    info!("責務: Event Store から Read Model への投影");
    info!("状態: 未実装（設計済み）");
    info!("===========================================");
    info!("");
    info!("イベントソーシングの投影処理を担当");
    info!("- ドメインイベントの購読");
    info!("- Read Model（非正規化ビュー）の更新");
    info!("- 投影状態の管理");
    info!("");
    info!("詳細: docs/tactical/contexts/vocabulary/");
    info!("===========================================");

    // 設定読み込み
    let _config = config::Config::from_env()?;

    // イベント購読ループ（未実装）
    info!("イベント購読待機中...");
    tokio::signal::ctrl_c().await?;
    info!("シャットダウン");

    Ok(())
}
