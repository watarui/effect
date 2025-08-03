//! `CreateItem` コマンドハンドラー

use async_trait::async_trait;
use tracing::{error, info};

use crate::{
    domain::{commands::CreateItem, entities::vocabulary_item::VocabularyItem},
    ports::outbound::repository::Repository,
};

/// `CreateItem` コマンドハンドラーのエラー
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// リポジトリエラー
    #[error("Repository error: {0}")]
    Repository(String),
    /// ドメインエラー
    #[error("Domain error: {0}")]
    Domain(#[from] crate::domain::entities::vocabulary_item::ItemError),
    /// 項目が既に存在する
    #[error("Item already exists: {0}")]
    AlreadyExists(String),
}

/// `CreateItem` コマンドハンドラー
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
    type Command = CreateItem;
    type Result = Result<VocabularyItem, Error>;

    async fn handle(&self, command: Self::Command) -> Self::Result {
        info!(
            "Creating vocabulary item: word={}, part_of_speech={:?}",
            command.word, command.part_of_speech
        );

        // 既存の項目をチェック
        match self.repository.find_by_word(&command.word).await {
            Ok(Some(existing)) => {
                error!(
                    "Item already exists: word={}, id={}",
                    command.word,
                    existing.id()
                );
                return Err(Error::AlreadyExists(command.word));
            },
            Ok(None) => {
                // 項目が存在しない場合は続行
            },
            Err(e) => {
                return Err(Error::Repository(e.to_string()));
            },
        }

        // 新しい語彙項目を作成
        let definitions: Vec<&str> = command.definitions.iter().map(String::as_str).collect();
        let item = VocabularyItem::new(
            &command.word,
            command.part_of_speech,
            command.cefr_level,
            command.register,
            command.domain,
            definitions,
        )?;

        // リポジトリに保存
        self.repository
            .save(&item)
            .await
            .map_err(|e| Error::Repository(e.to_string()))?;

        info!(
            "Created vocabulary item: id={}, word={}",
            item.id(),
            item.word()
        );

        Ok(item)
    }
}

/// コマンドハンドラーのトレイト
#[async_trait]
pub trait CommandHandler {
    /// コマンドの型
    type Command;
    /// 結果の型
    type Result;

    /// コマンドを処理する
    async fn handle(&self, command: Self::Command) -> Self::Result;
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;

    use super::*;
    use crate::{
        domain::value_objects::{domain::Domain, part_of_speech::*, register::Register},
        ports::outbound::repository::MockRepository,
    };

    #[tokio::test]
    async fn test_create_item_success() {
        let mut mock_repo = MockRepository::new();

        // find_by_word は None を返す（項目が存在しない）
        mock_repo
            .expect_find_by_word()
            .with(eq("test"))
            .times(1)
            .returning(|_| Ok(None));

        // save は成功する
        mock_repo.expect_save().times(1).returning(|_| Ok(()));

        let handler = Handler::new(mock_repo);
        let command = CreateItem {
            word:           "test".to_string(),
            part_of_speech: PartOfSpeech::Noun(NounType::Countable),
            cefr_level:     Some(domain_events::CefrLevel::B1),
            register:       Register::Neutral,
            domain:         Domain::General,
            definitions:    vec!["A test item".to_string()],
            created_by:     common_types::UserId::new(),
        };

        let result = handler.handle(command).await;
        assert!(result.is_ok());

        let item = result.unwrap();
        assert_eq!(item.word(), "test");
    }

    #[tokio::test]
    async fn test_create_item_already_exists() {
        let mut mock_repo = MockRepository::new();

        let existing_item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
            vec!["Existing item"],
        )
        .unwrap();

        // find_by_word は既存の項目を返す
        mock_repo
            .expect_find_by_word()
            .with(eq("test"))
            .times(1)
            .returning(move |_| Ok(Some(existing_item.clone())));

        let handler = Handler::new(mock_repo);
        let command = CreateItem {
            word:           "test".to_string(),
            part_of_speech: PartOfSpeech::Noun(NounType::Countable),
            cefr_level:     None,
            register:       Register::Neutral,
            domain:         Domain::General,
            definitions:    vec!["A test item".to_string()],
            created_by:     common_types::UserId::new(),
        };

        let result = handler.handle(command).await;
        assert!(result.is_err());
    }
}
