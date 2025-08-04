//! リポジトリ実装

use async_trait::async_trait;
use shared_error::DomainResult;
use uuid::Uuid;

use crate::{domain::aggregates::VocabularyEntry, ports::outbound::VocabularyRepository};

/// インメモリリポジトリ（仮実装）
pub struct InMemoryVocabularyRepository;

impl Default for InMemoryVocabularyRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryVocabularyRepository {
    /// 新しいリポジトリを作成
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl VocabularyRepository for InMemoryVocabularyRepository {
    async fn find_by_word(&self, _word: &str) -> DomainResult<Option<VocabularyEntry>> {
        // TODO: 実装
        Ok(None)
    }

    async fn save(&self, _entry: &VocabularyEntry) -> DomainResult<()> {
        // TODO: 実装
        Ok(())
    }

    async fn delete(&self, _id: Uuid) -> DomainResult<()> {
        // TODO: 実装
        Ok(())
    }
}
