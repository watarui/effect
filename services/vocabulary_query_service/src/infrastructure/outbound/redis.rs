//! Redis キャッシュ実装
//!
//! キャッシュレイヤーの実装

use std::time::Duration;

use async_trait::async_trait;
use redis::{AsyncCommands, aio::ConnectionManager};
use shared_error::{DomainError, DomainResult};

use crate::ports::outbound::CacheService;

/// Redis キャッシュサービス
pub struct RedisCacheService {
    connection: ConnectionManager,
}

impl RedisCacheService {
    /// 新しいキャッシュサービスを作成
    pub fn new(connection: ConnectionManager) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl CacheService for RedisCacheService {
    async fn get_json(&self, key: &str) -> DomainResult<Option<String>> {
        let mut conn = self.connection.clone();
        let data: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(data)
    }

    async fn set_json(&self, key: &str, json: &str, ttl: Duration) -> DomainResult<()> {
        let mut conn = self.connection.clone();
        let _: () = conn
            .set_ex(key, json, ttl.as_secs())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> DomainResult<()> {
        let mut conn = self.connection.clone();
        let _: () = conn
            .del(key)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}
