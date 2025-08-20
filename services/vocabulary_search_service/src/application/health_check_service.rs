//! ヘルスチェックサービス

use async_trait::async_trait;
use tracing::{error, info};

use crate::{
    error::Result,
    ports::{
        inbound::{HealthCheckUseCase, HealthStatus, IndexStatus, SearchEngineStatus},
        outbound::SearchEngineRepository,
    },
};

/// ヘルスチェックサービス
pub struct HealthCheckService<S>
where
    S: SearchEngineRepository,
{
    search_engine: S,
}

impl<S> HealthCheckService<S>
where
    S: SearchEngineRepository,
{
    pub fn new(search_engine: S) -> Self {
        Self { search_engine }
    }
}

#[async_trait]
impl<S> HealthCheckUseCase for HealthCheckService<S>
where
    S: SearchEngineRepository + Send + Sync,
{
    async fn check_health(&self) -> Result<HealthStatus> {
        // Meilisearch 接続チェック
        let meilisearch_status = match self.search_engine.health_check().await {
            Ok(_) => {
                info!("Meilisearch connection is healthy");
                SearchEngineStatus::Connected
            },
            Err(e) => {
                error!("Meilisearch health check failed: {}", e);
                SearchEngineStatus::Error(e.to_string())
            },
        };

        // インデックス状態チェック
        let index_status = match self.search_engine.get_statistics().await {
            Ok(stats) => {
                if stats.is_indexing {
                    IndexStatus::Indexing
                } else if stats.total_documents == 0 {
                    IndexStatus::NotInitialized
                } else {
                    IndexStatus::Ready
                }
            },
            Err(e) => {
                error!("Failed to get index statistics: {}", e);
                IndexStatus::Error(e.to_string())
            },
        };

        let is_healthy = matches!(meilisearch_status, SearchEngineStatus::Connected)
            && matches!(index_status, IndexStatus::Ready | IndexStatus::Indexing);

        Ok(HealthStatus {
            is_healthy,
            meilisearch_status,
            index_status,
        })
    }
}
