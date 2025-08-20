//! インデックス管理サービス

use async_trait::async_trait;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    domain::{IndexSettings, IndexStatistics, VocabularySearchItem},
    error::Result,
    ports::{
        inbound::IndexManagementUseCase,
        outbound::{DataSourceRepository, SearchEngineRepository},
    },
};

/// インデックス管理サービス
pub struct IndexManagementService<S, D>
where
    S: SearchEngineRepository,
    D: DataSourceRepository,
{
    search_engine: S,
    data_source:   D,
}

impl<S, D> IndexManagementService<S, D>
where
    S: SearchEngineRepository,
    D: DataSourceRepository,
{
    pub fn new(search_engine: S, data_source: D) -> Self {
        Self {
            search_engine,
            data_source,
        }
    }
}

#[async_trait]
impl<S, D> IndexManagementUseCase for IndexManagementService<S, D>
where
    S: SearchEngineRepository + Send + Sync,
    D: DataSourceRepository + Send + Sync,
{
    async fn rebuild_index(&self) -> Result<()> {
        info!("Starting index rebuild");

        // 既存のインデックスをクリア
        self.search_engine.clear_index().await?;
        info!("Cleared existing index");

        // データソースから全アイテムを取得
        let items = self.data_source.get_all_items().await?;
        info!("Fetched {} items from data source", items.len());

        if items.is_empty() {
            warn!("No items to index");
            return Ok(());
        }

        // バッチサイズ（Meilisearch の推奨値）
        const BATCH_SIZE: usize = 1000;

        // バッチでインデックス
        for (i, chunk) in items.chunks(BATCH_SIZE).enumerate() {
            match self.search_engine.batch_index(chunk).await {
                Ok(_) => {
                    info!(
                        "Indexed batch {}/{} ({} items)",
                        i + 1,
                        items.len().div_ceil(BATCH_SIZE),
                        chunk.len()
                    );
                },
                Err(e) => {
                    error!("Failed to index batch {}: {}", i + 1, e);
                    return Err(e);
                },
            }
        }

        info!("Index rebuild completed successfully");
        Ok(())
    }

    async fn get_index_statistics(&self) -> Result<IndexStatistics> {
        self.search_engine.get_statistics().await
    }

    async fn get_index_settings(&self) -> Result<IndexSettings> {
        self.search_engine.get_settings().await
    }

    async fn update_index_settings(&self, settings: IndexSettings) -> Result<()> {
        info!("Updating index settings");
        self.search_engine.update_settings(&settings).await?;
        info!("Index settings updated successfully");
        Ok(())
    }

    async fn index_document(&self, item: VocabularySearchItem) -> Result<()> {
        info!("Indexing document: {}", item.item_id);
        self.search_engine.index_document(&item).await?;
        info!("Document indexed successfully");
        Ok(())
    }

    async fn delete_document(&self, item_id: Uuid) -> Result<()> {
        info!("Deleting document: {}", item_id);
        self.search_engine.delete_document(item_id).await?;
        info!("Document deleted successfully");
        Ok(())
    }

    async fn batch_index_documents(&self, items: Vec<VocabularySearchItem>) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        info!("Batch indexing {} documents", items.len());
        self.search_engine.batch_index(&items).await?;
        info!("Batch indexing completed");
        Ok(())
    }
}
