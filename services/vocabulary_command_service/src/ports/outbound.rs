//! アウトバウンドポート
//!
//! 外部リソースにアクセスするためのインターフェース

use async_trait::async_trait;
use shared_error::DomainResult;
use uuid::Uuid;

use crate::domain::{aggregates::VocabularyEntry, events::VocabularyDomainEvent};

/// イベントストアのインターフェース
#[async_trait]
pub trait EventStore: Send + Sync {
    /// 集約を保存
    async fn save_aggregate(
        &self,
        aggregate_id: Uuid,
        events: Vec<VocabularyDomainEvent>,
        expected_version: Option<u32>,
    ) -> DomainResult<()>;

    /// 集約を読み込み
    async fn load_aggregate(&self, aggregate_id: Uuid) -> DomainResult<Option<VocabularyEntry>>;

    /// イベントストリームを取得
    async fn get_events(
        &self,
        aggregate_id: Uuid,
        from_version: Option<u32>,
    ) -> DomainResult<Vec<VocabularyDomainEvent>>;
}

/// イベントバスのインターフェース
#[async_trait]
pub trait EventBus: Send + Sync {
    /// イベントを発行
    async fn publish(&self, events: Vec<VocabularyDomainEvent>) -> DomainResult<()>;
}

/// 語彙リポジトリのインターフェース
#[async_trait]
pub trait VocabularyRepository: Send + Sync {
    /// エントリをIDで検索
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<VocabularyEntry>>;

    /// エントリを単語で検索
    async fn find_by_word(&self, word: &str) -> DomainResult<Option<VocabularyEntry>>;

    /// エントリを保存
    async fn save(&self, entry: &mut VocabularyEntry) -> DomainResult<()>;

    /// エントリを削除
    async fn delete(&self, id: Uuid) -> DomainResult<()>;
}
