//! クエリ分析器の実装

#![allow(clippy::should_implement_trait)]

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::{
    domain::{
        error::SearchError,
        search_models::{AnalyzedQuery, Language},
    },
    ports::outbound::QueryAnalyzer as QueryAnalyzerTrait,
};

/// トークナイザー trait
#[async_trait]
pub trait Tokenizer: Send + Sync {
    /// テキストをトークンに分割
    async fn tokenize(&self, text: &str) -> Result<Vec<String>, SearchError>;
}

/// シンプルなトークナイザー実装
pub struct SimpleTokenizer;

#[async_trait]
impl Tokenizer for SimpleTokenizer {
    async fn tokenize(&self, text: &str) -> Result<Vec<String>, SearchError> {
        // シンプルな空白区切りのトークン化
        let tokens = text.split_whitespace().map(|s| s.to_string()).collect();

        Ok(tokens)
    }
}

/// 同義語辞書
pub struct SynonymDictionary {
    synonyms: HashMap<String, Vec<String>>,
}

impl SynonymDictionary {
    /// 新しい同義語辞書を作成
    pub fn new() -> Self {
        let mut synonyms = HashMap::new();

        // 基本的な同義語を設定
        synonyms.insert(
            "learn".to_string(),
            vec![
                "study".to_string(),
                "acquire".to_string(),
                "master".to_string(),
            ],
        );
        synonyms.insert(
            "big".to_string(),
            vec![
                "large".to_string(),
                "huge".to_string(),
                "enormous".to_string(),
            ],
        );
        synonyms.insert(
            "small".to_string(),
            vec![
                "little".to_string(),
                "tiny".to_string(),
                "petite".to_string(),
            ],
        );
        synonyms.insert(
            "fast".to_string(),
            vec![
                "quick".to_string(),
                "rapid".to_string(),
                "swift".to_string(),
            ],
        );
        synonyms.insert(
            "slow".to_string(),
            vec!["sluggish".to_string(), "leisurely".to_string()],
        );

        Self { synonyms }
    }

    /// 単語の同義語を取得
    pub fn get_synonyms(&self, word: &str) -> Vec<String> {
        self.synonyms
            .get(&word.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for SynonymDictionary {
    fn default() -> Self {
        Self::new()
    }
}

/// クエリアナライザー実装
pub struct QueryAnalyzer {
    tokenizer:    Arc<dyn Tokenizer>,
    synonym_dict: Arc<SynonymDictionary>,
}

impl QueryAnalyzer {
    /// 新しいクエリアナライザーを作成
    pub fn new(tokenizer: Arc<dyn Tokenizer>, synonym_dict: Arc<SynonymDictionary>) -> Self {
        Self {
            tokenizer,
            synonym_dict,
        }
    }

    /// デフォルトのアナライザーを作成
    pub fn default() -> Self {
        Self::new(
            Arc::new(SimpleTokenizer),
            Arc::new(SynonymDictionary::new()),
        )
    }

    /// 同義語を展開
    async fn expand_synonyms(&self, tokens: &[String]) -> Vec<String> {
        let mut all_synonyms = Vec::new();

        for token in tokens {
            let synonyms = self.synonym_dict.get_synonyms(token);
            all_synonyms.extend(synonyms);
        }

        all_synonyms
    }

    /// 言語を検出
    fn detect_language(&self, tokens: &[String]) -> Language {
        let mut has_japanese = false;
        let mut has_english = false;

        for token in tokens {
            for ch in token.chars() {
                if Self::is_japanese_char(ch) {
                    has_japanese = true;
                } else if ch.is_ascii_alphabetic() {
                    has_english = true;
                }
            }
        }

        match (has_japanese, has_english) {
            (true, true) => Language::Mixed,
            (true, false) => Language::Japanese,
            (false, true) => Language::English,
            _ => Language::Unknown,
        }
    }

    /// 日本語文字かどうかを判定
    fn is_japanese_char(ch: char) -> bool {
        matches!(ch as u32,
            0x3040..=0x309F | // ひらがな
            0x30A0..=0x30FF | // カタカナ
            0x4E00..=0x9FAF   // 漢字
        )
    }
}

#[async_trait]
impl QueryAnalyzerTrait for QueryAnalyzer {
    async fn analyze(&self, query: &str) -> Result<AnalyzedQuery, SearchError> {
        // 1. トークン化
        let tokens = self.tokenizer.tokenize(query).await?;

        // 2. 正規化
        let normalized = tokens
            .iter()
            .map(|t| t.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");

        // 3. 同義語展開
        let synonyms = self.expand_synonyms(&tokens).await;

        // 4. 言語検出
        let language = self.detect_language(&tokens);

        Ok(AnalyzedQuery {
            original_query: query.to_string(),
            normalized_query: normalized,
            tokens,
            synonyms,
            language,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_tokenizer() {
        let tokenizer = SimpleTokenizer;
        let tokens = tokenizer.tokenize("hello world test").await.unwrap();

        assert_eq!(tokens, vec!["hello", "world", "test"]);
    }

    #[tokio::test]
    async fn test_synonym_dictionary() {
        let dict = SynonymDictionary::new();

        let synonyms = dict.get_synonyms("learn");
        assert!(synonyms.contains(&"study".to_string()));
        assert!(synonyms.contains(&"acquire".to_string()));

        let empty = dict.get_synonyms("nonexistent");
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn test_query_analyzer() {
        let analyzer = QueryAnalyzer::default();

        let result = analyzer.analyze("learn English").await.unwrap();

        assert_eq!(result.original_query, "learn English");
        assert_eq!(result.normalized_query, "learn english");
        assert_eq!(result.tokens, vec!["learn", "English"]);
        assert!(!result.synonyms.is_empty());
        assert_eq!(result.language, Language::English);
    }

    #[test]
    fn test_language_detection() {
        let analyzer = QueryAnalyzer::default();

        // 英語のみ
        let english_tokens = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(analyzer.detect_language(&english_tokens), Language::English);

        // 日本語のみ
        let japanese_tokens = vec!["こんにちは".to_string(), "世界".to_string()];
        assert_eq!(
            analyzer.detect_language(&japanese_tokens),
            Language::Japanese
        );

        // 混合
        let mixed_tokens = vec!["hello".to_string(), "世界".to_string()];
        assert_eq!(analyzer.detect_language(&mixed_tokens), Language::Mixed);
    }
}
