//! Repository port definitions
//!
//! リポジトリの抽象インターフェースを定義

use async_trait::async_trait;
use common_types::ItemId;

use crate::domain::entities::vocabulary_item::VocabularyItem;

/// 語彙項目リポジトリのエラー型
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// リポジトリ操作が失敗した場合
    #[error("Repository operation failed: {0}")]
    OperationFailed(String),
    /// エンティティが見つからない場合
    #[error("Entity not found: {0}")]
    NotFound(String),
    /// 楽観的ロックの失敗
    #[error("Optimistic lock failure")]
    OptimisticLockFailure,
}

/// 語彙項目リポジトリのポート
#[async_trait]
pub trait Repository: Send + Sync {
    /// エラー型
    type Error: std::error::Error + Send + Sync + 'static;

    /// 語彙項目を保存
    async fn save(&self, item: &VocabularyItem) -> Result<(), Self::Error>;

    /// ID で語彙項目を検索
    async fn find_by_id(&self, id: &ItemId) -> Result<Option<VocabularyItem>, Self::Error>;

    /// 単語で語彙項目を検索
    async fn find_by_word(&self, word: &str) -> Result<Option<VocabularyItem>, Self::Error>;

    /// 語彙項目をソフトデリート
    async fn soft_delete(&self, id: &ItemId) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_error_should_be_displayable() {
        let error = Error::NotFound("test".to_string());
        assert_eq!(error.to_string(), "Entity not found: test");

        let error = Error::OperationFailed("connection failed".to_string());
        assert_eq!(
            error.to_string(),
            "Repository operation failed: connection failed"
        );

        let error = Error::OptimisticLockFailure;
        assert_eq!(error.to_string(), "Optimistic lock failure");
    }

    #[test]
    fn repository_error_should_implement_error_trait() {
        let error = Error::NotFound("test".to_string());
        // Error トレイトが実装されていることを確認
        let _: &dyn std::error::Error = &error;
    }
}
