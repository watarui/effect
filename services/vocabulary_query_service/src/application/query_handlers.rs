//! クエリハンドラー
//!
//! 各種クエリの処理を実装

use std::sync::Arc;

use shared_error::{DomainError, DomainResult};
use uuid::Uuid;

use crate::{
    domain::read_models::VocabularyItemView,
    ports::outbound::{CacheService, ReadModelRepository},
};

/// 項目取得ハンドラー
pub struct GetItemHandler {
    repository: Arc<dyn ReadModelRepository>,
    cache:      Arc<dyn CacheService>,
}

impl GetItemHandler {
    /// 新しいハンドラーを作成
    pub fn new(repository: Arc<dyn ReadModelRepository>, cache: Arc<dyn CacheService>) -> Self {
        Self { repository, cache }
    }

    /// 項目を取得
    pub async fn handle(&self, item_id: Uuid) -> DomainResult<VocabularyItemView> {
        // キャッシュから取得を試みる
        let cache_key = format!("item:{item_id}");
        if let Ok(Some(json)) = self.cache.get_json(&cache_key).await
            && let Ok(item) = serde_json::from_str::<VocabularyItemView>(&json)
        {
            return Ok(item);
        }

        // データベースから取得
        let item = self
            .repository
            .get_item(item_id)
            .await?
            .ok_or_else(|| DomainError::not_found("VocabularyItem", item_id))?;

        // キャッシュに保存
        if let Ok(json) = serde_json::to_string(&item) {
            let _ = self
                .cache
                .set_json(&cache_key, &json, std::time::Duration::from_secs(300))
                .await;
        }

        Ok(item)
    }
}
