//! PostgreSQL リポジトリ実装
//!
//! Read Model の取得

use async_trait::async_trait;
use shared_error::{DomainError, DomainResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::read_models::{VocabularyEntryView, VocabularyItemView, VocabularyStats},
    ports::outbound::ReadModelRepository,
};

/// PostgreSQL Read Model リポジトリ
pub struct PostgresReadModelRepository {
    pool: PgPool,
}

impl PostgresReadModelRepository {
    /// 新しいリポジトリを作成
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReadModelRepository for PostgresReadModelRepository {
    async fn get_item(&self, item_id: Uuid) -> DomainResult<Option<VocabularyItemView>> {
        // TODO: 実装
        let _ = item_id;
        let _ = &self.pool;
        Ok(None)
    }

    async fn get_entry(&self, entry_id: Uuid) -> DomainResult<Option<VocabularyEntryView>> {
        // TODO: 実装
        let _ = entry_id;
        let _ = &self.pool;
        Ok(None)
    }

    async fn get_stats(&self) -> DomainResult<VocabularyStats> {
        // TODO: 実装
        let _ = &self.pool;
        Err(DomainError::Internal("Not implemented".to_string()))
    }
}
