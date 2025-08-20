//! 語彙クエリサービスの実装

use async_trait::async_trait;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    domain::{
        Cursor,
        PageSize,
        PagedResult,
        SearchQuery,
        SortOptions,
        VocabularyEntry,
        VocabularyFilter,
        VocabularyItem,
        VocabularyStatistics,
    },
    error::{QueryError, Result},
    ports::{
        inbound::VocabularyQueryUseCase,
        outbound::{CacheRepository, ReadModelRepository},
    },
};

/// 語彙クエリサービス
pub struct VocabularyQueryService<R, C>
where
    R: ReadModelRepository,
    C: CacheRepository,
{
    repository: R,
    cache:      Option<C>,
}

impl<R, C> VocabularyQueryService<R, C>
where
    R: ReadModelRepository,
    C: CacheRepository,
{
    pub fn new(repository: R, cache: Option<C>) -> Self {
        Self { repository, cache }
    }

    /// キャッシュキーを生成
    fn cache_key(&self, prefix: &str, id: &str) -> String {
        format!("vocabulary:{}:{}", prefix, id)
    }

    /// キャッシュから取得を試みる
    async fn try_get_from_cache<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        if let Some(cache) = &self.cache {
            match cache.get(key).await {
                Ok(Some(data)) => match serde_json::from_slice(&data) {
                    Ok(value) => {
                        debug!("Cache hit for key: {}", key);
                        return Some(value);
                    },
                    Err(e) => {
                        error!("Failed to deserialize cache data: {}", e);
                    },
                },
                Ok(None) => {
                    debug!("Cache miss for key: {}", key);
                },
                Err(e) => {
                    error!("Cache error: {}", e);
                },
            }
        }
        None
    }

    /// キャッシュに保存
    async fn save_to_cache<T>(&self, key: &str, value: &T, ttl: u64)
    where
        T: serde::Serialize,
    {
        if let Some(cache) = &self.cache {
            match serde_json::to_vec(value) {
                Ok(data) => {
                    if let Err(e) = cache.set(key, data, ttl).await {
                        error!("Failed to save to cache: {}", e);
                    } else {
                        debug!("Saved to cache with key: {}", key);
                    }
                },
                Err(e) => {
                    error!("Failed to serialize for cache: {}", e);
                },
            }
        }
    }
}

#[async_trait]
impl<R, C> VocabularyQueryUseCase for VocabularyQueryService<R, C>
where
    R: ReadModelRepository + Send + Sync,
    C: CacheRepository + Send + Sync,
{
    async fn get_entry_by_id(&self, entry_id: Uuid) -> Result<Option<VocabularyEntry>> {
        let cache_key = self.cache_key("entry", &entry_id.to_string());

        // キャッシュから取得を試みる
        if let Some(entry) = self.try_get_from_cache(&cache_key).await {
            return Ok(Some(entry));
        }

        // データベースから取得
        let entry = self.repository.find_entry_by_id(entry_id).await?;

        // キャッシュに保存（5分間）
        if let Some(ref e) = entry {
            self.save_to_cache(&cache_key, e, 300).await;
        }

        Ok(entry)
    }

    async fn get_entry_by_spelling(&self, spelling: &str) -> Result<Option<VocabularyEntry>> {
        let cache_key = self.cache_key("entry:spelling", spelling);

        if let Some(entry) = self.try_get_from_cache(&cache_key).await {
            return Ok(Some(entry));
        }

        let entry = self.repository.find_entry_by_spelling(spelling).await?;

        if let Some(ref e) = entry {
            self.save_to_cache(&cache_key, e, 300).await;
        }

        Ok(entry)
    }

    async fn list_entries(
        &self,
        filter: Option<VocabularyFilter>,
        sort: Option<SortOptions>,
        cursor: Option<Cursor>,
        page_size: PageSize,
    ) -> Result<PagedResult<VocabularyEntry>> {
        info!("Listing entries with filter: {:?}", filter);
        self.repository
            .find_entries(filter, sort, cursor, page_size)
            .await
    }

    async fn get_item_by_id(&self, item_id: Uuid) -> Result<Option<VocabularyItem>> {
        let cache_key = self.cache_key("item", &item_id.to_string());

        if let Some(item) = self.try_get_from_cache(&cache_key).await {
            return Ok(Some(item));
        }

        let mut item = self.repository.find_item_by_id(item_id).await?;

        // 例文も取得
        if let Some(ref mut item) = item {
            item.examples = self.repository.find_examples_by_item_id(item_id).await?;
            self.save_to_cache(&cache_key, item, 300).await;
        }

        Ok(item)
    }

    async fn list_items_by_entry(
        &self,
        entry_id: Uuid,
        include_deleted: bool,
    ) -> Result<Vec<VocabularyItem>> {
        let mut items = self
            .repository
            .find_items_by_entry_id(entry_id, include_deleted)
            .await?;

        // 各アイテムの例文を取得
        for item in &mut items {
            item.examples = self
                .repository
                .find_examples_by_item_id(item.item_id)
                .await?;
        }

        Ok(items)
    }

    async fn list_items(
        &self,
        filter: Option<VocabularyFilter>,
        sort: Option<SortOptions>,
        cursor: Option<Cursor>,
        page_size: PageSize,
    ) -> Result<PagedResult<VocabularyItem>> {
        info!("Listing items with filter: {:?}", filter);
        self.repository
            .find_items(filter, sort, cursor, page_size)
            .await
    }

    async fn search(
        &self,
        query: SearchQuery,
        filter: Option<VocabularyFilter>,
        cursor: Option<Cursor>,
        page_size: PageSize,
    ) -> Result<PagedResult<VocabularyItem>> {
        if !query.is_valid() {
            return Err(QueryError::InvalidInput(
                "Search query must be at least 2 characters".to_string(),
            ));
        }

        info!("Searching for: {}", query.term);
        self.repository
            .search_items(&query.term, filter, cursor, page_size)
            .await
    }

    async fn get_statistics(&self) -> Result<VocabularyStatistics> {
        let cache_key = "vocabulary:statistics";

        if let Some(stats) = self.try_get_from_cache(cache_key).await {
            return Ok(stats);
        }

        let stats = self.repository.get_statistics().await?;

        // 統計情報は1時間キャッシュ
        self.save_to_cache(cache_key, &stats, 3600).await;

        Ok(stats)
    }
}
