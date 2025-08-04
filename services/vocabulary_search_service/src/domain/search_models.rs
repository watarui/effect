//! 検索用のドメインモデル
//!
//! Meilisearch に格納される検索ドキュメントと関連する型を定義

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 語彙検索ドキュメント
///
/// Meilisearch に格納される検索用のドキュメント構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularySearchDocument {
    /// ドキュメント ID（Meilisearch のプライマリキー）
    pub id: String,

    // 基本フィールド
    pub item_id:        String,
    pub entry_id:       String,
    pub spelling:       String,
    pub disambiguation: String,

    // 検索対象フィールド
    pub search_text: String, // 全フィールドを結合したテキスト
    pub definitions: Vec<String>,
    pub examples:    Vec<String>,
    pub synonyms:    Vec<String>,
    pub antonyms:    Vec<String>,

    // フィルタ用フィールド
    pub part_of_speech:  String,
    pub cefr_level:      Option<String>,
    pub domain:          String,
    pub tags:            Vec<String>,
    pub is_ai_generated: bool,

    // 並べ替え用フィールド
    pub popularity_score: f32,
    pub quality_score:    f32,
    pub created_at:       DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,

    // サジェスト用
    pub spelling_grams: String, // n-gram for partial matching
}

impl VocabularySearchDocument {
    /// 新しい検索ドキュメントを作成
    pub fn new(
        item_id: String,
        entry_id: String,
        spelling: String,
        disambiguation: String,
    ) -> Self {
        let id = format!("{entry_id}__{item_id}");

        Self {
            id,
            item_id,
            entry_id,
            spelling: spelling.clone(),
            disambiguation,
            search_text: spelling, // 初期値として spelling を設定
            definitions: Vec::new(),
            examples: Vec::new(),
            synonyms: Vec::new(),
            antonyms: Vec::new(),
            part_of_speech: String::new(),
            cefr_level: None,
            domain: String::new(),
            tags: Vec::new(),
            is_ai_generated: false,
            popularity_score: 0.0,
            quality_score: 0.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            spelling_grams: String::new(),
        }
    }

    /// 検索用テキストを構築
    pub fn build_search_text(&mut self) {
        let mut parts = vec![self.spelling.clone(), self.disambiguation.clone()];

        parts.extend(self.definitions.clone());
        parts.extend(self.examples.clone());
        parts.extend(self.synonyms.clone());

        self.search_text = parts.join(" ");
    }
}

/// 検索ファセット
///
/// 検索結果のファセット集計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacets {
    pub part_of_speech: HashMap<String, u64>,
    pub cefr_level:     HashMap<String, u64>,
    pub domain:         HashMap<String, u64>,
    pub tags:           HashMap<String, u64>,
    pub year_created:   HashMap<u32, u64>,
}

/// スペリングサジェスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellingSuggestion {
    pub text:      String,
    pub score:     f32,
    pub frequency: u64,
}

/// 分析済みクエリ
///
/// クエリアナライザーによって分析されたクエリ情報
#[derive(Debug, Clone)]
pub struct AnalyzedQuery {
    pub original_query:   String,
    pub normalized_query: String,
    pub tokens:           Vec<String>,
    pub synonyms:         Vec<String>,
    pub language:         Language,
}

/// 言語
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Japanese,
    Mixed,
    Unknown,
}

/// 検索結果
#[derive(Debug, Clone)]
pub struct SearchResult<T> {
    pub hits:       Vec<SearchHit<T>>,
    pub total_hits: u32,
    pub max_score:  f32,
    pub facets:     Option<SearchFacets>,
}

/// 検索ヒット
#[derive(Debug, Clone)]
pub struct SearchHit<T> {
    pub document:   T,
    pub score:      f32,
    pub highlights: HashMap<String, String>,
}

/// Meilisearch クエリ
#[derive(Debug, Clone)]
pub struct MeilisearchQuery {
    pub query_string: String,
    pub filter:       Option<String>,
    pub highlight:    Option<HighlightConfig>,
    pub sort:         Option<Vec<String>>,
}

/// ハイライト設定
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    pub attributes: Vec<String>,
    pub pre_tag:    String,
    pub post_tag:   String,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            attributes: vec![],
            pre_tag:    "<mark>".to_string(),
            post_tag:   "</mark>".to_string(),
        }
    }
}
