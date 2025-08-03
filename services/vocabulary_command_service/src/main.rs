//! Vocabulary Command Service
//!
//! CQRS+ES パターンにおける Write Model を担当するサービス。
//! コマンドの処理、ドメインイベントの生成、Event Store への永続化を行う。

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // テレメトリの初期化
    let _tracer = shared_telemetry::init_telemetry("vocabulary_command_service", None)
        .map_err(|e| anyhow::anyhow!("Failed to initialize telemetry: {}", e))?;

    info!("Starting Vocabulary Command Service");

    // TODO: 以下を実装
    // 1. 設定の読み込み
    // 2. Event Store への接続
    // 3. Pub/Sub への接続
    // 4. gRPC サーバーの起動

    info!("Vocabulary Command Service is running");

    // シグナルハンドリング
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Vocabulary Command Service");

    Ok(())
}
