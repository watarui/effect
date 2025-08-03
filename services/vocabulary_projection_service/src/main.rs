//! Vocabulary Projection Service
//!
//! CQRS+ES パターンにおける Read Model を構築するサービス。
//! Event Store からのイベントを購読し、プロジェクションを生成・更新する。

use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // テレメトリの初期化
    let _tracer = shared_telemetry::init_telemetry("vocabulary_projection_service", None)?;

    info!("Starting Vocabulary Projection Service");

    // TODO: 以下を実装
    // 1. 設定の読み込み
    // 2. PostgreSQL への接続（Read Model 保存用）
    // 3. Event Store への接続（イベント読み取り用）
    // 4. Google Pub/Sub への接続（サブスクライバー）
    // 5. イベントハンドラーの起動
    // 6. プロジェクション更新ループの開始

    info!("Vocabulary Projection Service is running");

    // シグナルハンドリング
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Vocabulary Projection Service");

    Ok(())
}
