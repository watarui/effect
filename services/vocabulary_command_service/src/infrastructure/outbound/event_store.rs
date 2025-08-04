//! Event Store 実装

use async_trait::async_trait;
use shared_error::DomainResult;
use uuid::Uuid;

use crate::{
    domain::{aggregates::VocabularyEntry, events::VocabularyDomainEvent},
    ports::outbound::EventStore,
};

/// PostgreSQL ベースの Event Store 実装
pub struct PostgresEventStore {
    // TODO: PostgreSQL 接続プールを追加
    // pool: sqlx::PgPool,
}

impl Default for PostgresEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresEventStore {
    /// 新しい Event Store を作成
    pub fn new() -> Self {
        Self {
            // TODO: pool を注入
        }
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn save_aggregate(
        &self,
        _aggregate_id: Uuid,
        _events: Vec<VocabularyDomainEvent>,
        _expected_version: Option<u32>,
    ) -> DomainResult<()> {
        // TODO: 実装
        // 1. 楽観的ロックのチェック
        // 2. イベントを events テーブルに保存
        // 3. スナップショットの更新（必要に応じて）
        Ok(())
    }

    async fn load_aggregate(&self, _aggregate_id: Uuid) -> DomainResult<Option<VocabularyEntry>> {
        // TODO: 実装
        // 1. スナップショットを読み込み（あれば）
        // 2. スナップショット以降のイベントを読み込み
        // 3. イベントを適用して現在の状態を再構築
        Ok(None)
    }

    async fn get_events(
        &self,
        _aggregate_id: Uuid,
        _from_version: Option<u32>,
    ) -> DomainResult<Vec<VocabularyDomainEvent>> {
        // TODO: 実装
        // events テーブルから指定されたバージョン以降のイベントを取得
        Ok(Vec::new())
    }
}
