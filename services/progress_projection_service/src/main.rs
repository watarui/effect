use progress_projection_service::{config, error::Result, processor};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt::init();

    info!("===========================================");
    info!("Progress Projection Service - 起動中");
    info!("責務: イベントを Read Model に投影");
    info!("状態: 未実装（設計済み）");
    info!("===========================================");
    info!("");
    info!("純粋な CQRS/Event Sourcing の Projection");
    info!("- Event Store からのイベント読み取り");
    info!("- Read Model への投影処理");
    info!("- 投影の整合性保証");
    info!("- リトライとエラーハンドリング");
    info!("");
    info!("詳細: docs/tactical/contexts/progress/");
    info!("===========================================");

    // 設定読み込み
    let config = config::Config::from_env()?;

    // イベントプロセッサー起動
    processor::run(config).await?;

    Ok(())
}
