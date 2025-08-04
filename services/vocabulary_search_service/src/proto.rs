//! Proto types temporary implementation
//!
//! This is a temporary implementation until proto compilation is set up

use serde::{Deserialize, Serialize};
// Re-export tonic Status
pub use tonic::Status;

// Request and Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItemsRequest {
    pub query:      String,
    pub pagination: Option<Pagination>,
    pub filters:    Option<SearchFilters>,
    pub options:    Option<SearchOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItemsResponse {
    pub items:       Vec<SearchResultItem>,
    pub total_hits:  u64,
    pub max_score:   f32,
    pub took_ms:     u32,
    pub suggestions: Vec<SpellingSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSuggestionsRequest {
    pub prefix: String,
    pub limit:  u32,
    pub r#type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSuggestionsResponse {
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRelatedItemsRequest {
    pub item_id:       String,
    pub relation_type: i32,
    pub limit:         u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRelatedItemsResponse {
    pub items: Vec<RelatedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWithFacetsRequest {
    pub query:      String,
    pub pagination: Option<Pagination>,
    pub filters:    Option<SearchFilters>,
    pub options:    Option<SearchOptions>,
    pub facets:     Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWithFacetsResponse {
    pub items:       Vec<SearchResultItem>,
    pub total_hits:  u64,
    pub max_score:   f32,
    pub took_ms:     u32,
    pub suggestions: Vec<SpellingSuggestion>,
    pub facets:      std::collections::HashMap<String, FacetDistribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetDistribution {
    pub values: std::collections::HashMap<String, u64>,
}

// Supporting types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub offset: u32,
    pub limit:  u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub part_of_speech:    Vec<String>,
    pub cefr_levels:       Vec<String>,
    pub ai_generated_only: bool,
    pub domains:           Vec<String>,
    pub tags:              Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub mode:          i32,
    pub highlight_tag: String,
    pub sort_by:       Option<SortOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOptions {
    pub field:      i32,
    pub descending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub item_id:        String,
    pub entry_id:       String,
    pub spelling:       String,
    pub disambiguation: Option<String>,
    pub score:          f32,
    pub highlights:     std::collections::HashMap<String, Vec<String>>,
    pub explanation:    Option<MatchExplanation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellingSuggestion {
    pub text:  String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub text:         String,
    pub display_text: String,
    pub score:        f32,
    pub r#type:       i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedItem {
    pub item_id:        String,
    pub spelling:       String,
    pub disambiguation: Option<String>,
    pub relation_score: f32,
    pub relation_type:  i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchExplanation {
    pub field_matches: Vec<FieldMatch>,
    pub total_score:   f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMatch {
    pub field: String,
    pub score: f32,
}

// Enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum SearchMode {
    Exact    = 0,
    Fuzzy    = 1,
    Phrase   = 2,
    Wildcard = 3,
    Semantic = 4,
}

impl TryFrom<i32> for SearchMode {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SearchMode::Exact),
            1 => Ok(SearchMode::Fuzzy),
            2 => Ok(SearchMode::Phrase),
            3 => Ok(SearchMode::Wildcard),
            4 => Ok(SearchMode::Semantic),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum SuggestionType {
    Spelling   = 0,
    Definition = 1,
    Example    = 2,
}

impl TryFrom<i32> for SuggestionType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SuggestionType::Spelling),
            1 => Ok(SuggestionType::Definition),
            2 => Ok(SuggestionType::Example),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum RelationType {
    Synonyms     = 0,
    Antonyms     = 1,
    SimilarUsage = 2,
    SameDomain   = 3,
    SameLevel    = 4,
}

impl TryFrom<i32> for RelationType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RelationType::Synonyms),
            1 => Ok(RelationType::Antonyms),
            2 => Ok(RelationType::SimilarUsage),
            3 => Ok(RelationType::SameDomain),
            4 => Ok(RelationType::SameLevel),
            _ => Err(()),
        }
    }
}
