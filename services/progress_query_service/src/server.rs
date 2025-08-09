use std::net::SocketAddr;

use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_axum::GraphQL;
use axum::{Json, Router, routing::get};
use serde_json::json;
use tracing::info;

use crate::config::Config;

// GraphQL Query Root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn api_version(&self) -> &str {
        "0.1.0"
    }

    async fn service_status(&self) -> &str {
        "Progress Query Service - 未実装"
    }
}

pub async fn run(config: Config) -> crate::error::Result<()> {
    // GraphQL スキーマ構築
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    // ルーター構築
    let app = Router::new()
        .route(
            "/graphql",
            get(graphql_playground).post_service(GraphQL::new(schema)),
        )
        .route("/health", get(health_check))
        .route("/", get(index));

    // サーバーアドレス
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Progress Query Service listening on {}", addr);

    // サーバー起動
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "progress_query_service",
        "implementation": "pending"
    }))
}

async fn index() -> Json<serde_json::Value> {
    Json(json!({
        "service": "Progress Query Service",
        "version": "0.1.0",
        "status": "未実装",
        "responsibility": "GraphQL API による読み取りモデル提供",
        "description": "純粋な CQRS/Event Sourcing の Read 側",
        "endpoints": {
            "graphql": "/graphql",
            "health": "/health"
        },
        "documentation": "docs/tactical/contexts/progress/architecture.md"
    }))
}
