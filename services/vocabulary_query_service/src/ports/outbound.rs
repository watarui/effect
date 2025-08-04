//! アウトバウンドポート
//!
//! 外部サービスとの接続インターフェース

use std::time::Duration;

use async_trait::async_trait;
use shared_error::DomainResult;
use uuid::Uuid;

use crate::domain::read_models::{VocabularyEntryView, VocabularyItemView, VocabularyStats};

/// Read Model リポジトリ
#[async_trait]
pub trait ReadModelRepository: Send + Sync {
    /// 項目を取得
    async fn get_item(&self, item_id: Uuid) -> DomainResult<Option<VocabularyItemView>>;

    /// エントリーを取得
    async fn get_entry(&self, entry_id: Uuid) -> DomainResult<Option<VocabularyEntryView>>;

    /// 統計情報を取得
    async fn get_stats(&self) -> DomainResult<VocabularyStats>;
}

/// キャッシュサービス
#[async_trait]
pub trait CacheService: Send + Sync {
    /// キャッシュから取得（JSON として）
    async fn get_json(&self, key: &str) -> DomainResult<Option<String>>;

    /// キャッシュに保存（JSON として）
    async fn set_json(&self, key: &str, json: &str, ttl: Duration) -> DomainResult<()>;

    /// キャッシュから削除
    async fn delete(&self, key: &str) -> DomainResult<()>;
}
