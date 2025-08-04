//! ドメインサービス
//!
//! 複数の集約にまたがるビジネスロジック

use shared_error::{DomainError, DomainResult};

use super::aggregates::VocabularyEntry;

/// 語彙ドメインサービス
pub struct VocabularyDomainService;

impl VocabularyDomainService {
    /// 重複チェック
    pub fn check_duplicate(existing_entries: &[VocabularyEntry], word: &str) -> DomainResult<()> {
        if existing_entries.iter().any(|e| e.word() == word) {
            return Err(DomainError::DuplicateEntry(format!(
                "Entry for '{word}' already exists"
            )));
        }
        Ok(())
    }

    /// 公開可能かチェック
    pub fn can_publish(entry: &VocabularyEntry) -> DomainResult<()> {
        // ビジネスルール: 少なくとも1つの項目が必要
        if !entry.has_items() {
            return Err(DomainError::InvalidState(
                "Cannot publish entry without any items".to_string(),
            ));
        }
        Ok(())
    }
}
