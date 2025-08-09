use std::net::SocketAddr;

use axum::{Json, Router, routing::get};
use serde_json::json;
use tracing::info;

use crate::config::Config;

pub async fn run(config: Config) -> crate::error::Result<()> {
    // ルーター構築
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(index));

    // サーバーアドレス
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Progress Command Service listening on {}", addr);

    // サーバー起動
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "progress_command_service",
        "implementation": "pending"
    }))
}

async fn index() -> Json<serde_json::Value> {
    Json(json!({
        "service": "Progress Command Service",
        "version": "0.1.0",
        "status": "未実装",
        "responsibility": "イベント受信と永続化",
        "description": "純粋な CQRS/Event Sourcing の Write 側",
        "documentation": "docs/tactical/contexts/progress/architecture.md"
    }))
}
