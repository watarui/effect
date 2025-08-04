//! Vocabulary Query Service
//!
//! CQRS+ES パターンにおける Read Model を提供するサービス。
//! 基本的な読み取り操作とシンプルなフィルタリングを行う。

use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // テレメトリの初期化
    let _tracer = shared_telemetry::init_telemetry("vocabulary_query_service", None)?;

    info!("Starting Vocabulary Query Service");

    // TODO: 以下を実装
    // 1. 設定の読み込み
    // 2. PostgreSQL Read DB への接続
    // 3. Redis への接続
    // 4. gRPC サーバーの起動

    info!("Vocabulary Query Service is running");

    // シグナルハンドリング
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Vocabulary Query Service");

    Ok(())
}
