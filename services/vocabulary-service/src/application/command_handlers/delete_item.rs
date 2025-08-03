//! `DeleteItem` コマンドハンドラー

use async_trait::async_trait;
use tracing::{error, info};

use crate::{
    application::command_handlers::create_item::CommandHandler,
    domain::commands::DeleteItem,
    ports::outbound::repository::Repository,
};

/// `DeleteItem` コマンドハンドラーのエラー
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// リポジトリエラー
    #[error("Repository error: {0}")]
    Repository(String),
    /// 項目が見つからない
    #[error("Item not found: {0}")]
    NotFound(common_types::ItemId),
}

/// `DeleteItem` コマンドハンドラー
pub struct Handler<R> {
    repository: R,
}

impl<R> Handler<R> {
    /// 新しいハンドラーを作成
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> CommandHandler for Handler<R>
where
    R: Repository + Send + Sync,
{
    type Command = DeleteItem;
    type Result = Result<(), Error>;

    async fn handle(&self, command: Self::Command) -> Self::Result {
        info!("Deleting vocabulary item: id={}", command.item_id);

        // 項目の存在確認
        let item = match self.repository.find_by_id(&command.item_id).await {
            Ok(Some(item)) => item,
            Ok(None) => return Err(Error::NotFound(command.item_id)),
            Err(e) => return Err(Error::Repository(e.to_string())),
        };

        // 論理削除を実行
        self.repository
            .soft_delete(&command.item_id)
            .await
            .map_err(|e| Error::Repository(e.to_string()))?;

        info!(
            "Deleted vocabulary item: id={}, word={}",
            command.item_id,
            item.word()
        );

        Ok(())
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
    async fn test_delete_item_success() {
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

        // soft_delete は成功する
        mock_repo
            .expect_soft_delete()
            .with(eq(item_id))
            .times(1)
            .returning(|_| Ok(()));

        let handler = Handler::new(mock_repo);
        let command = DeleteItem {
            item_id,
            deleted_by: common_types::UserId::new(),
        };

        let result = handler.handle(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_item_not_found() {
        let mut mock_repo = MockRepository::new();

        let item_id = common_types::ItemId::new();

        // find_by_id は None を返す（項目が存在しない）
        mock_repo
            .expect_find_by_id()
            .with(eq(item_id))
            .times(1)
            .returning(|_| Ok(None));

        let handler = Handler::new(mock_repo);
        let command = DeleteItem {
            item_id,
            deleted_by: common_types::UserId::new(),
        };

        let result = handler.handle(command).await;
        assert!(matches!(result, Err(super::Error::NotFound(_))));
    }
}
