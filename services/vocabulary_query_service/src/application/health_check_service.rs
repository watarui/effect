//! ヘルスチェックサービス

use async_trait::async_trait;
use tracing::{error, info};

use crate::{
    error::Result,
    ports::{
        inbound::{DatabaseStatus, HealthCheckUseCase, HealthStatus},
        outbound::ReadModelRepository,
    },
};

/// ヘルスチェックサービス
pub struct HealthCheckService<R>
where
    R: ReadModelRepository,
{
    repository: R,
}

impl<R> HealthCheckService<R>
where
    R: ReadModelRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> HealthCheckUseCase for HealthCheckService<R>
where
    R: ReadModelRepository + Send + Sync,
{
    async fn check_health(&self) -> Result<HealthStatus> {
        let database_status = match self.repository.health_check().await {
            Ok(_) => {
                info!("Database connection is healthy");
                DatabaseStatus::Connected
            },
            Err(e) => {
                error!("Database health check failed: {}", e);
                DatabaseStatus::Error(e.to_string())
            },
        };

        let is_healthy = matches!(database_status, DatabaseStatus::Connected);

        Ok(HealthStatus {
            is_healthy,
            database_status,
            message: if is_healthy {
                None
            } else {
                Some("Database connection failed".to_string())
            },
        })
    }
}
