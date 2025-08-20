//! 検索ドメインモデル

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 検索結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub items:           Vec<T>,
    pub total_results:   usize,
    pub total_pages:     usize,
    pub current_page:    usize,
    pub processing_time: u64, // milliseconds
    pub query:           String,
}

/// 語彙検索アイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularySearchItem {
    pub item_id:           Uuid,
    pub entry_id:          Uuid,
    pub spelling:          String,
    pub disambiguation:    Option<String>,
    pub part_of_speech:    Option<String>,
    pub definition:        Option<String>,
    pub ipa_pronunciation: Option<String>,
    pub cefr_level:        Option<String>,
    pub frequency_rank:    Option<i32>,
    pub example_count:     i32,
    pub score:             f32, // 検索スコア
    pub created_at:        DateTime<Utc>,
    pub updated_at:        DateTime<Utc>,
}

/// オートコンプリート候補
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteItem {
    pub spelling:       String,
    pub display:        String,
    pub category:       String, // "word", "phrase", "collocation"
    pub frequency_rank: Option<i32>,
    pub score:          f32,
}

/// 検索フィルター
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilter {
    pub part_of_speech: Option<Vec<String>>,
    pub cefr_level:     Option<Vec<String>>,
    pub min_frequency:  Option<i32>,
    pub max_frequency:  Option<i32>,
    pub has_definition: Option<bool>,
    pub has_examples:   Option<bool>,
    pub is_published:   Option<bool>,
}

/// ソートオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,     // 関連性（デフォルト）
    Spelling,      // アルファベット順
    FrequencyRank, // 頻度順
    CefrLevel,     // CEFR レベル順
    ExampleCount,  // 例文数順
    CreatedAt,     // 作成日順
    UpdatedAt,     // 更新日順
}

impl Default for SortBy {
    fn default() -> Self {
        Self::Relevance
    }
}

/// ソート順序
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Descending
    }
}

/// 検索クエリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query:      String,
    pub filter:     Option<SearchFilter>,
    pub sort_by:    Option<SortBy>,
    pub sort_order: Option<SortOrder>,
    pub page:       Option<u32>,
    pub per_page:   Option<u32>,
    pub facets:     Option<Vec<String>>,
}

/// ファセット検索結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetValue {
    pub value: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacets {
    pub part_of_speech: Vec<FacetValue>,
    pub cefr_level:     Vec<FacetValue>,
    pub has_definition: Vec<FacetValue>,
    pub has_examples:   Vec<FacetValue>,
}

/// インデックス統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatistics {
    pub total_documents:   usize,
    pub indexed_documents: usize,
    pub deleted_documents: usize,
    pub primary_key:       String,
    pub index_size:        u64, // bytes
    pub last_updated:      DateTime<Utc>,
    pub is_indexing:       bool,
}

/// 検索ハイライト設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightOptions {
    pub enabled:    bool,
    pub tag_open:   String,
    pub tag_close:  String,
    pub attributes: Vec<String>,
}

impl Default for HighlightOptions {
    fn default() -> Self {
        Self {
            enabled:    true,
            tag_open:   "<mark>".to_string(),
            tag_close:  "</mark>".to_string(),
            attributes: vec!["spelling".to_string(), "definition".to_string()],
        }
    }
}

/// 同義語
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synonym {
    pub term:     String,
    pub synonyms: Vec<String>,
}

/// インデックス設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSettings {
    pub searchable_attributes: Vec<String>,
    pub filterable_attributes: Vec<String>,
    pub sortable_attributes:   Vec<String>,
    pub displayed_attributes:  Vec<String>,
    pub ranking_rules:         Vec<String>,
    pub stop_words:            Vec<String>,
    pub synonyms:              Vec<Synonym>,
    pub distinct_attribute:    Option<String>,
    pub proximity_precision:   ProximityPrecision,
    pub typo_tolerance:        TypoTolerance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProximityPrecision {
    ByWord,
    ByAttribute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypoTolerance {
    pub enabled:                     bool,
    pub min_word_size_for_one_typo:  u32,
    pub min_word_size_for_two_typos: u32,
    pub disable_on_words:            Vec<String>,
    pub disable_on_attributes:       Vec<String>,
}

impl Default for TypoTolerance {
    fn default() -> Self {
        Self {
            enabled:                     true,
            min_word_size_for_one_typo:  5,
            min_word_size_for_two_typos: 9,
            disable_on_words:            Vec::new(),
            disable_on_attributes:       Vec::new(),
        }
    }
}
