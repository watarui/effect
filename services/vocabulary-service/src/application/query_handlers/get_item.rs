//! `GetItem` クエリハンドラー

use async_trait::async_trait;
use tracing::info;

use crate::{
    application::queries::GetItem,
    domain::entities::vocabulary_item::VocabularyItem,
    ports::outbound::repository::Repository,
};

/// `GetItem` クエリハンドラーのエラー
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// リポジトリエラー
    #[error("Repository error: {0}")]
    Repository(String),
    /// 項目が見つからない
    #[error("Item not found: {0}")]
    NotFound(common_types::ItemId),
}

/// `GetItem` クエリハンドラー
pub struct Handler<R> {
    repository: R,
}

impl<R> Handler<R> {
    /// 新しいハンドラーを作成
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

/// クエリハンドラーのトレイト
#[async_trait]
pub trait QueryHandler {
    /// クエリの型
    type Query;
    /// 結果の型
    type Result;

    /// クエリを処理する
    async fn handle(&self, query: Self::Query) -> Self::Result;
}

#[async_trait]
impl<R> QueryHandler for Handler<R>
where
    R: Repository + Send + Sync,
{
    type Query = GetItem;
    type Result = Result<VocabularyItem, Error>;

    async fn handle(&self, query: Self::Query) -> Self::Result {
        info!("Getting vocabulary item: id={}", query.item_id);

        // リポジトリから項目を取得
        let item = match self.repository.find_by_id(&query.item_id).await {
            Ok(Some(item)) => item,
            Ok(None) => return Err(Error::NotFound(query.item_id)),
            Err(e) => return Err(Error::Repository(e.to_string())),
        };

        info!(
            "Found vocabulary item: id={}, word={}",
            item.id(),
            item.word()
        );

        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;

    use super::*;
    use crate::{
        domain::{
            entities::vocabulary_item::VocabularyItem,
            value_objects::{domain::Domain, part_of_speech::*, register::Register},
        },
        ports::outbound::repository::MockRepository,
    };

    #[tokio::test]
    async fn test_get_item_success() {
        let mut mock_repo = MockRepository::new();

        let item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
            vec!["A test item"],
        )
        .unwrap();

        let item_id = *item.id();

        // find_by_id は既存の項目を返す
        mock_repo
            .expect_find_by_id()
            .with(eq(item_id))
            .times(1)
            .returning(move |_| Ok(Some(item.clone())));

        let handler = Handler::new(mock_repo);
        let query = GetItem { item_id };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let fetched_item = result.unwrap();
        assert_eq!(fetched_item.id(), &item_id);
        assert_eq!(fetched_item.word(), "test");
    }

    #[tokio::test]
    async fn test_get_item_not_found() {
        let mut mock_repo = MockRepository::new();

        let item_id = common_types::ItemId::new();

        // find_by_id は None を返す（項目が存在しない）
        mock_repo
            .expect_find_by_id()
            .with(eq(item_id))
            .times(1)
            .returning(|_| Ok(None));

        let handler = Handler::new(mock_repo);
        let query = GetItem { item_id };

        let result = handler.handle(query).await;
        assert!(matches!(result, Err(super::Error::NotFound(_))));
    }
}
