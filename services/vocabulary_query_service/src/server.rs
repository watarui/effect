use std::net::SocketAddr;

use axum::{Json, Router, routing::get};
use serde_json::json;
use tracing::info;

use crate::config::Config;

pub async fn run(config: Config) -> anyhow::Result<()> {
    // ルーター構築
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(index));

    // サーバーアドレス
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Vocabulary Query Service listening on {}", addr);

    // サーバー起動
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "vocabulary_query_service",
        "implementation": "pending"
    }))
}

async fn index() -> Json<serde_json::Value> {
    Json(json!({
        "service": "Vocabulary Query Service",
        "version": "0.1.0",
        "status": "未実装",
        "responsibility": "クエリ処理（読み取り）",
        "description": "CQRS + Event Sourcing の Read 側を担当",
        "documentation": "docs/tactical/contexts/vocabulary/architecture.md"
    }))
}
