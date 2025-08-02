//! `PostgreSQL` vocabulary repository implementation
//!
//! 新しい設計に基づく語彙リポジトリの実装

use async_trait::async_trait;
use common_types::ItemId;
use sqlx::PgPool;

use crate::{
    domain::entities::vocabulary_item::VocabularyItem,
    ports::outbound::repository::{
        Error as RepositoryError,
        Repository as VocabularyItemRepository,
    },
};

/// `PostgreSQL` vocabulary repository
pub struct Repository {
    pool: PgPool,
}

impl Repository {
    /// 新しいインスタンスを作成
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VocabularyItemRepository for Repository {
    type Error = RepositoryError;

    async fn save(&self, _item: &VocabularyItem) -> Result<(), Self::Error> {
        // トランザクション内で複数テーブルに保存
        let _tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        // TODO: 実際の保存ロジックを実装
        // 1. vocabulary_entries テーブルに保存
        // 2. vocabulary_items テーブルに保存
        // 3. 関連テーブル（definitions, examples, etc.）に保存

        _tx.commit()
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, _id: &ItemId) -> Result<Option<VocabularyItem>, Self::Error> {
        // TODO: 複数テーブルから結合して取得
        // 現在は空の実装
        Ok(None)
    }

    async fn find_by_word(&self, _word: &str) -> Result<Option<VocabularyItem>, Self::Error> {
        // TODO: 実装
        Ok(None)
    }

    async fn soft_delete(&self, _id: &ItemId) -> Result<(), Self::Error> {
        // TODO: 実装
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use domain_events::CefrLevel;

    use super::*;
    use crate::domain::value_objects::{
        domain::Domain,
        part_of_speech::{NounType, PartOfSpeech},
        register::Register,
    };

    #[tokio::test]
    async fn repository_should_implement_trait() {
        // 接続が失敗する場合はテストをスキップ
        let Ok(pool) = PgPool::connect("postgresql://test").await else {
            eprintln!("Database connection failed, skipping test");
            return;
        };

        let repo = Repository::new(pool);

        // トレイトが実装されていることを確認
        let item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            Some(CefrLevel::B1),
            Register::Neutral,
            Domain::General,
            vec!["test definition"],
        )
        .unwrap();

        // メソッドが呼び出せることを確認
        let _result = repo.save(&item).await;
        let _result = repo.find_by_id(item.id()).await;
    }
}
