//! インバウンドポート
//!
//! クエリサービスの公開インターフェース

use async_trait::async_trait;
use shared_error::DomainResult;
use uuid::Uuid;

use crate::domain::read_models::{VocabularyEntryView, VocabularyItemView, VocabularyStats};

/// クエリサービスインターフェース
#[async_trait]
pub trait QueryService: Send + Sync {
    /// 項目を取得
    async fn get_item(&self, item_id: Uuid) -> DomainResult<VocabularyItemView>;

    /// エントリーを取得
    async fn get_entry(&self, entry_id: Uuid) -> DomainResult<VocabularyEntryView>;

    /// 統計情報を取得
    async fn get_stats(&self) -> DomainResult<VocabularyStats>;
}
