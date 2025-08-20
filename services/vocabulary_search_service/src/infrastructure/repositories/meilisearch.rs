//! Meilisearch リポジトリ実装

use async_trait::async_trait;
use chrono::Utc;
use meilisearch_sdk::{client::*, indexes::*, search::Selectors, settings::Settings};
use serde_json::json;
use uuid::Uuid;

use crate::{
    domain::{
        AutocompleteItem,
        IndexSettings,
        IndexStatistics,
        ProximityPrecision,
        SearchFacets,
        SearchQuery,
        SearchResult,
        VocabularySearchItem,
    },
    error::{Result, SearchError},
    ports::outbound::SearchEngineRepository,
};

/// Meilisearch リポジトリ
pub struct MeilisearchRepository {
    client:     Client,
    index_name: String,
}

impl MeilisearchRepository {
    pub fn new(url: String, api_key: Option<String>, index_name: String) -> Self {
        let client = Client::new(url, api_key).expect("Failed to create Meilisearch client");
        Self { client, index_name }
    }

    /// インデックスを取得（なければ作成）
    async fn get_or_create_index(&self) -> Result<Index> {
        match self.client.get_index(&self.index_name).await {
            Ok(index) => Ok(index),
            Err(_) => {
                // インデックスが存在しない場合は作成
                self.client
                    .create_index(&self.index_name, Some("item_id"))
                    .await
                    .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

                let index = self
                    .client
                    .get_index(&self.index_name)
                    .await
                    .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

                // デフォルト設定を適用
                self.apply_default_settings(&index).await?;

                Ok(index)
            },
        }
    }

    /// デフォルトのインデックス設定を適用
    async fn apply_default_settings(&self, index: &Index) -> Result<()> {
        let settings = Settings::new()
            .with_searchable_attributes(vec![
                "spelling".to_string(),
                "definition".to_string(),
                "disambiguation".to_string(),
            ])
            .with_filterable_attributes(vec![
                "part_of_speech".to_string(),
                "cefr_level".to_string(),
                "frequency_rank".to_string(),
                "is_published".to_string(),
                "has_definition".to_string(),
                "has_examples".to_string(),
            ])
            .with_sortable_attributes(vec![
                "spelling".to_string(),
                "frequency_rank".to_string(),
                "cefr_level".to_string(),
                "example_count".to_string(),
                "created_at".to_string(),
                "updated_at".to_string(),
            ])
            .with_ranking_rules(vec![
                "words".to_string(),
                "typo".to_string(),
                "proximity".to_string(),
                "attribute".to_string(),
                "sort".to_string(),
                "exactness".to_string(),
            ]);

        index
            .set_settings(&settings)
            .await
            .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl SearchEngineRepository for MeilisearchRepository {
    async fn search(
        &self,
        query: &SearchQuery,
    ) -> Result<(SearchResult<VocabularySearchItem>, Option<SearchFacets>)> {
        let index = self.get_or_create_index().await?;

        // ページネーション
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20);

        // フィルターの構築
        let filter_str = if let Some(filter) = &query.filter {
            let mut filters = Vec::new();

            if let Some(pos) = &filter.part_of_speech {
                let pos_filter = pos
                    .iter()
                    .map(|p| format!("part_of_speech = \"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                filters.push(format!("({})", pos_filter));
            }

            if let Some(cefr) = &filter.cefr_level {
                let cefr_filter = cefr
                    .iter()
                    .map(|c| format!("cefr_level = \"{}\"", c))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                filters.push(format!("({})", cefr_filter));
            }

            if let Some(is_published) = filter.is_published {
                filters.push(format!("is_published = {}", is_published));
            }

            if !filters.is_empty() {
                Some(filters.join(" AND "))
            } else {
                None
            }
        } else {
            None
        };

        // ソートの構築
        let sort_order = if let Some(sort_by) = &query.sort_by {
            use crate::domain::SortBy;
            let sort_field = match sort_by {
                SortBy::Spelling => "spelling",
                SortBy::FrequencyRank => "frequency_rank",
                SortBy::CefrLevel => "cefr_level",
                SortBy::ExampleCount => "example_count",
                SortBy::CreatedAt => "created_at",
                SortBy::UpdatedAt => "updated_at",
                SortBy::Relevance => "", // デフォルト
            };

            if !sort_field.is_empty() {
                Some(
                    match query.sort_order.as_ref().unwrap_or(&Default::default()) {
                        crate::domain::SortOrder::Ascending => format!("{}:asc", sort_field),
                        crate::domain::SortOrder::Descending => format!("{}:desc", sort_field),
                    },
                )
            } else {
                None
            }
        } else {
            None
        };

        // ファセットの構築
        let facet_refs: Vec<&str> = if let Some(facets) = &query.facets {
            facets.iter().map(|s| s.as_str()).collect()
        } else {
            Vec::new()
        };

        // 検索クエリの構築と実行
        let start_time = std::time::Instant::now();

        let results = if let Some(ref sort) = sort_order {
            let sort_array = [sort.as_str()];
            let mut search = index.search();
            search.with_query(&query.query);
            search.with_limit(per_page as usize);
            search.with_offset(((page - 1) * per_page) as usize);

            if let Some(ref filter) = filter_str {
                search.with_filter(filter);
            }

            search.with_sort(&sort_array);

            if !facet_refs.is_empty() {
                search.with_facets(Selectors::Some(facet_refs.as_slice()));
            }

            search
                .execute::<VocabularySearchItem>()
                .await
                .map_err(|e| SearchError::SearchEngine(e.to_string()))?
        } else {
            let mut search = index.search();
            search.with_query(&query.query);
            search.with_limit(per_page as usize);
            search.with_offset(((page - 1) * per_page) as usize);

            if let Some(ref filter) = filter_str {
                search.with_filter(filter);
            }

            if !facet_refs.is_empty() {
                search.with_facets(Selectors::Some(facet_refs.as_slice()));
            }

            search
                .execute::<VocabularySearchItem>()
                .await
                .map_err(|e| SearchError::SearchEngine(e.to_string()))?
        };
        let processing_time = start_time.elapsed().as_millis() as u64;

        let total_results = results.estimated_total_hits.unwrap_or(0);
        let total_pages = total_results.div_ceil(per_page as usize);

        let search_result = SearchResult {
            items: results.hits.into_iter().map(|h| h.result).collect(),
            total_results,
            total_pages,
            current_page: page as usize,
            processing_time,
            query: query.query.clone(),
        };

        // TODO: ファセットの処理
        Ok((search_result, None))
    }

    async fn suggest(&self, prefix: &str, limit: usize) -> Result<Vec<AutocompleteItem>> {
        let index = self.get_or_create_index().await?;

        let results = index
            .search()
            .with_query(prefix)
            .with_limit(limit)
            .with_attributes_to_retrieve(Selectors::Some(&["spelling", "frequency_rank"]))
            .execute::<serde_json::Value>()
            .await
            .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

        let items = results
            .hits
            .into_iter()
            .map(|h| AutocompleteItem {
                spelling:       h.result["spelling"].as_str().unwrap_or("").to_string(),
                display:        h.result["spelling"].as_str().unwrap_or("").to_string(),
                category:       "word".to_string(),
                frequency_rank: h.result["frequency_rank"].as_i64().map(|v| v as i32),
                score:          1.0, // TODO: 実際のスコアを計算
            })
            .collect();

        Ok(items)
    }

    async fn find_similar(
        &self,
        _item_id: Uuid,
        _limit: usize,
    ) -> Result<Vec<VocabularySearchItem>> {
        // TODO: 類似検索の実装（ベクトル検索など）
        Ok(Vec::new())
    }

    async fn index_document(&self, document: &VocabularySearchItem) -> Result<()> {
        let index = self.get_or_create_index().await?;

        // has_definition と has_examples を計算
        let doc_with_flags = json!({
            "item_id": document.item_id,
            "entry_id": document.entry_id,
            "spelling": document.spelling,
            "disambiguation": document.disambiguation,
            "part_of_speech": document.part_of_speech,
            "definition": document.definition,
            "ipa_pronunciation": document.ipa_pronunciation,
            "cefr_level": document.cefr_level,
            "frequency_rank": document.frequency_rank,
            "example_count": document.example_count,
            "has_definition": document.definition.is_some(),
            "has_examples": document.example_count > 0,
            "is_published": true, // TODO: 実際の値を使用
            "created_at": document.created_at,
            "updated_at": document.updated_at,
        });

        index
            .add_documents(&[doc_with_flags], Some("item_id"))
            .await
            .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

        Ok(())
    }

    async fn batch_index(&self, documents: &[VocabularySearchItem]) -> Result<()> {
        if documents.is_empty() {
            return Ok(());
        }

        let index = self.get_or_create_index().await?;

        let docs_with_flags: Vec<_> = documents
            .iter()
            .map(|doc| {
                json!({
                    "item_id": doc.item_id,
                    "entry_id": doc.entry_id,
                    "spelling": doc.spelling,
                    "disambiguation": doc.disambiguation,
                    "part_of_speech": doc.part_of_speech,
                    "definition": doc.definition,
                    "ipa_pronunciation": doc.ipa_pronunciation,
                    "cefr_level": doc.cefr_level,
                    "frequency_rank": doc.frequency_rank,
                    "example_count": doc.example_count,
                    "has_definition": doc.definition.is_some(),
                    "has_examples": doc.example_count > 0,
                    "is_published": true, // TODO: 実際の値を使用
                    "created_at": doc.created_at,
                    "updated_at": doc.updated_at,
                })
            })
            .collect();

        index
            .add_documents(&docs_with_flags, Some("item_id"))
            .await
            .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

        Ok(())
    }

    async fn delete_document(&self, item_id: Uuid) -> Result<()> {
        let index = self.get_or_create_index().await?;

        index
            .delete_document(&item_id.to_string())
            .await
            .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

        Ok(())
    }

    async fn clear_index(&self) -> Result<()> {
        let index = self.get_or_create_index().await?;

        index
            .delete_all_documents()
            .await
            .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

        Ok(())
    }

    async fn get_statistics(&self) -> Result<IndexStatistics> {
        let index = self.get_or_create_index().await?;

        let stats = index
            .get_stats()
            .await
            .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

        Ok(IndexStatistics {
            total_documents:   stats.number_of_documents,
            indexed_documents: stats.number_of_documents,
            deleted_documents: 0, // Meilisearch doesn't track this
            primary_key:       "item_id".to_string(),
            index_size:        0, // TODO: Get actual size
            last_updated:      Utc::now(),
            is_indexing:       stats.is_indexing,
        })
    }

    async fn get_settings(&self) -> Result<IndexSettings> {
        let index = self.get_or_create_index().await?;

        let settings = index
            .get_settings()
            .await
            .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

        Ok(IndexSettings {
            searchable_attributes: settings
                .searchable_attributes
                .unwrap_or_else(|| vec!["*".to_string()]),
            filterable_attributes: settings.filterable_attributes.unwrap_or_default(),
            sortable_attributes:   settings.sortable_attributes.unwrap_or_default(),
            displayed_attributes:  settings
                .displayed_attributes
                .unwrap_or_else(|| vec!["*".to_string()]),
            ranking_rules:         settings.ranking_rules.unwrap_or_default(),
            stop_words:            settings.stop_words.unwrap_or_default(),
            synonyms:              Vec::new(), // TODO: Convert from Meilisearch format
            distinct_attribute:    settings.distinct_attribute.flatten(),
            proximity_precision:   ProximityPrecision::ByWord, // TODO: Map from settings
            typo_tolerance:        Default::default(),         // TODO: Map from settings
        })
    }

    async fn update_settings(&self, settings: &IndexSettings) -> Result<()> {
        let index = self.get_or_create_index().await?;

        let ms_settings = Settings::new()
            .with_searchable_attributes(settings.searchable_attributes.clone())
            .with_filterable_attributes(settings.filterable_attributes.clone())
            .with_sortable_attributes(settings.sortable_attributes.clone())
            .with_displayed_attributes(settings.displayed_attributes.clone())
            .with_ranking_rules(settings.ranking_rules.clone())
            .with_stop_words(settings.stop_words.clone());

        index
            .set_settings(&ms_settings)
            .await
            .map_err(|e| SearchError::SearchEngine(e.to_string()))?;

        Ok(())
    }

    async fn health_check(&self) -> Result<()> {
        self.client
            .health()
            .await
            .map_err(|e| SearchError::SearchEngine(e.to_string()))?;
        Ok(())
    }
}
