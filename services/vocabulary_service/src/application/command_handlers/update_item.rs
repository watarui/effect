//! `UpdateItem` コマンドハンドラー

use async_trait::async_trait;
use shared_repository::Entity;
use tracing::{error, info};

use crate::{
    application::command_handlers::create_item::CommandHandler,
    domain::{
        commands::UpdateItem,
        entities::vocabulary_item::{ItemError, VocabularyItem},
    },
    ports::outbound::repository::Repository,
};

/// `UpdateItem` コマンドハンドラーのエラー
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// リポジトリエラー
    #[error("Repository error: {0}")]
    Repository(String),
    /// ドメインエラー
    #[error("Domain error: {0}")]
    Domain(#[from] ItemError),
    /// 項目が見つからない
    #[error("Item not found: {0}")]
    NotFound(common_types::ItemId),
    /// 楽観的ロックエラー
    #[error("Version mismatch: expected {expected}, actual {actual}")]
    VersionMismatch {
        /// 期待されたバージョン
        expected: u64,
        /// 実際のバージョン
        actual:   u64,
    },
}

/// `UpdateItem` コマンドハンドラー
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
    type Command = UpdateItem;
    type Result = Result<VocabularyItem, Error>;

    async fn handle(&self, command: Self::Command) -> Self::Result {
        info!("Updating vocabulary item: id={}", command.item_id);

        // 既存の項目を取得
        let mut item = match self.repository.find_by_id(&command.item_id).await {
            Ok(Some(item)) => item,
            Ok(None) => return Err(Error::NotFound(command.item_id)),
            Err(e) => return Err(Error::Repository(e.to_string())),
        };

        // バージョンチェック（楽観的ロック）
        if item.version() != command.expected_version {
            error!(
                "Version mismatch for item {}: expected {}, actual {}",
                command.item_id,
                command.expected_version,
                item.version()
            );
            return Err(Error::VersionMismatch {
                expected: command.expected_version,
                actual:   item.version(),
            });
        }

        // 各フィールドを更新
        // TODO: VocabularyItem に適切な更新メソッドを追加後、実装を完成させる

        if let Some(cefr_level) = command.cefr_level {
            item.update_cefr_level(Some(cefr_level));
        }

        if let Some(definitions) = command.definitions {
            // 定義を追加（現在のAPIでは置き換えはできない）
            for definition in definitions {
                if let Err(e) = item.add_definition(&definition) {
                    error!("Failed to add definition: {}", e);
                }
            }
        }

        if let Some(pronunciation) = command.pronunciation {
            item.set_pronunciation(Some(&pronunciation));
        }

        if let Some(synonyms) = command.synonyms {
            for synonym in synonyms {
                item.add_synonym(&synonym);
            }
        }

        if let Some(antonyms) = command.antonyms {
            for antonym in antonyms {
                item.add_antonym(&antonym);
            }
        }

        if let Some(examples) = command.examples {
            // 例文を追加
            for example in examples {
                if let Err(e) = item.add_example(&example, None) {
                    error!("Failed to add example: {}", e);
                }
            }
        }

        if let Some(collocations) = command.collocations {
            // コロケーションを追加
            for collocation in collocations {
                let parts: Vec<&str> = collocation.split(',').collect();
                if parts.len() >= 2 {
                    item.add_collocation(parts[0].trim(), parts[1..].to_vec());
                }
            }
        }

        // リポジトリに保存
        self.repository
            .save(&item)
            .await
            .map_err(|e| Error::Repository(e.to_string()))?;

        info!(
            "Updated vocabulary item: id={}, version={}",
            item.id(),
            item.version()
        );

        Ok(item)
    }
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
    async fn test_update_item_success() {
        let mut mock_repo = MockRepository::new();

        let item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            Some(domain_events::CefrLevel::B1),
            Register::Neutral,
            Domain::General,
            vec!["Original definition"],
        )
        .unwrap();

        let item_id = *item.id();
        let version = item.version();

        // find_by_id は既存の項目を返す
        mock_repo
            .expect_find_by_id()
            .with(eq(item_id))
            .times(1)
            .returning(move |_| Ok(Some(item.clone())));

        // save は成功する
        mock_repo.expect_save().times(1).returning(|_| Ok(()));

        let handler = Handler::new(mock_repo);
        let command = UpdateItem {
            item_id,
            part_of_speech: Some(PartOfSpeech::Verb(VerbType::Transitive)),
            cefr_level: Some(domain_events::CefrLevel::B2),
            register: None,
            domain: None,
            definitions: Some(vec!["Updated definition".to_string()]),
            pronunciation: None,
            synonyms: None,
            antonyms: None,
            examples: None,
            collocations: None,
            updated_by: common_types::UserId::new(),
            expected_version: version,
        };

        let result = handler.handle(command).await;
        assert!(result.is_ok());

        let _updated_item = result.unwrap();
        // TODO: 更新されたフィールドの確認
    }

    #[tokio::test]
    async fn test_update_item_not_found() {
        let mut mock_repo = MockRepository::new();

        let item_id = common_types::ItemId::new();

        // find_by_id は None を返す（項目が存在しない）
        mock_repo
            .expect_find_by_id()
            .with(eq(item_id))
            .times(1)
            .returning(|_| Ok(None));

        let handler = Handler::new(mock_repo);
        let command = UpdateItem {
            item_id,
            part_of_speech: None,
            cefr_level: None,
            register: None,
            domain: None,
            definitions: None,
            pronunciation: None,
            synonyms: None,
            antonyms: None,
            examples: None,
            collocations: None,
            updated_by: common_types::UserId::new(),
            expected_version: 0,
        };

        let result = handler.handle(command).await;
        assert!(matches!(result, Err(super::Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_update_item_version_mismatch() {
        let mut mock_repo = MockRepository::new();

        let item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            None,
            Register::Neutral,
            Domain::General,
            vec!["Original definition"],
        )
        .unwrap();

        let item_id = *item.id();
        let actual_version = item.version();

        // find_by_id は既存の項目を返す
        mock_repo
            .expect_find_by_id()
            .with(eq(item_id))
            .times(1)
            .returning(move |_| Ok(Some(item.clone())));

        let handler = Handler::new(mock_repo);
        let command = UpdateItem {
            item_id,
            part_of_speech: None,
            cefr_level: None,
            register: None,
            domain: None,
            definitions: None,
            pronunciation: None,
            synonyms: None,
            antonyms: None,
            examples: None,
            collocations: None,
            updated_by: common_types::UserId::new(),
            expected_version: actual_version + 1, // 間違ったバージョン
        };

        let result = handler.handle(command).await;
        assert!(matches!(
            result,
            Err(super::Error::VersionMismatch {
                expected: _,
                actual:   _,
            })
        ));
    }
}
