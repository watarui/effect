use std::net::SocketAddr;

use axum::{Json, Router, routing::get};
use serde_json::json;
use tokio::task;
use tracing::info;
use vocabulary_command_service::{
    config::Config,
    error::{Error, Result},
    infrastructure::grpc::server::start_grpc_server,
};

pub async fn run(config: Config) -> Result<()> {
    let config_clone = config.clone();

    // HTTPヘルスチェックサーバーを別タスクで起動
    let http_task = task::spawn(async move {
        let http_port = 8080; // HTTPヘルスチェック用ポート
        run_http_server(config_clone.server.host.clone(), http_port).await
    });

    // gRPCサーバーを別タスクで起動
    let grpc_task = task::spawn(async move { start_grpc_server(config).await });

    // 両方のサーバーを並行して実行
    tokio::select! {
        result = http_task => {
            result
                .map_err(|e| Error::Internal(format!("HTTP task error: {}", e)))?
                .map_err(|e| Error::Internal(format!("HTTP server error: {}", e)))?
        }
        result = grpc_task => {
            result
                .map_err(|e| Error::Internal(format!("gRPC task error: {}", e)))?
                .map_err(|e| Error::Internal(format!("gRPC server error: {}", e)))?
        }
    }

    Ok(())
}

async fn run_http_server(host: String, port: u16) -> Result<()> {
    // ルーター構築
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(index));

    // サーバーアドレス
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| Error::Config(format!("Invalid address: {}", e)))?;

    info!("HTTP health check server listening on {}", addr);

    // サーバー起動
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| Error::Config(format!("Failed to bind address: {}", e)))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "vocabulary_command_service",
        "implementation": "pending"
    }))
}

async fn index() -> Json<serde_json::Value> {
    Json(json!({
        "service": "Vocabulary Command Service",
        "version": "0.1.0",
        "status": "未実装",
        "responsibility": "コマンド処理（書き込み）",
        "description": "CQRS + Event Sourcing の Write 側を担当",
        "documentation": "docs/tactical/contexts/vocabulary/architecture.md"
    }))
}
