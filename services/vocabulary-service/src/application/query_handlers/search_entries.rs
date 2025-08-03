//! `SearchEntries` クエリハンドラー

use async_trait::async_trait;
use tracing::info;

use crate::{
    application::{queries::SearchEntries, query_handlers::get_item::QueryHandler},
    domain::entities::vocabulary_item::VocabularyItem,
    ports::outbound::repository::Repository,
};

/// 検索結果の項目
#[derive(Debug, Clone)]
pub struct SearchResultItem {
    /// 項目 ID
    pub item_id:            common_types::ItemId,
    /// 単語
    pub word:               String,
    /// 品詞
    pub part_of_speech:     String,
    /// 最初の定義
    pub primary_definition: String,
}

impl From<&VocabularyItem> for SearchResultItem {
    fn from(item: &VocabularyItem) -> Self {
        Self {
            item_id:            *item.id(),
            word:               item.word().to_string(),
            // TODO: VocabularyItem に適切な getter を追加後、修正
            part_of_speech:     "unknown".to_string(),
            primary_definition: if item.definitions().is_empty() {
                "No definition".to_string()
            } else {
                "Definition available".to_string()
            },
        }
    }
}

/// 検索結果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 項目のリスト
    pub items:       Vec<SearchResultItem>,
    /// 総件数
    pub total_count: u32,
}

/// `SearchEntries` クエリハンドラーのエラー
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// リポジトリエラー
    #[error("Repository error: {0}")]
    Repository(String),
    /// 無効なパラメータ
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// `SearchEntries` クエリハンドラー
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
impl<R> QueryHandler for Handler<R>
where
    R: Repository + Send + Sync,
{
    type Query = SearchEntries;
    type Result = Result<SearchResult, Error>;

    async fn handle(&self, query: Self::Query) -> Self::Result {
        // パラメータのバリデーション
        let limit = query.limit.unwrap_or(10).min(50);
        let offset = query.offset.unwrap_or(0);

        if query.query.is_empty() {
            return Err(Error::InvalidParameter("Query cannot be empty".to_string()));
        }

        info!(
            "Searching vocabulary entries: query={}, limit={}, offset={}",
            query.query, limit, offset
        );

        // TODO: 現在の実装では単一項目の検索のみ対応
        // 将来的には複数エントリーの検索とページネーションを実装
        let items = match self.repository.find_by_word(&query.query).await {
            Ok(Some(item)) => vec![SearchResultItem::from(&item)],
            Ok(None) => vec![],
            Err(e) => return Err(Error::Repository(e.to_string())),
        };

        let total_count = u32::try_from(items.len()).unwrap_or(u32::MAX);

        info!("Found {} entries for query: {}", total_count, query.query);

        Ok(SearchResult { items, total_count })
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
    async fn test_search_entries_success() {
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

        // find_by_word は既存の項目を返す
        mock_repo
            .expect_find_by_word()
            .with(eq("test"))
            .times(1)
            .returning(move |_| Ok(Some(item.clone())));

        let handler = Handler::new(mock_repo);
        let query = SearchEntries {
            query:  "test".to_string(),
            limit:  None,
            offset: None,
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let search_result = result.unwrap();
        assert_eq!(search_result.items.len(), 1);
        assert_eq!(search_result.total_count, 1);
    }

    #[tokio::test]
    async fn test_search_entries_no_results() {
        let mut mock_repo = MockRepository::new();

        // find_by_word は None を返す（項目が存在しない）
        mock_repo
            .expect_find_by_word()
            .with(eq("nonexistent"))
            .times(1)
            .returning(|_| Ok(None));

        let handler = Handler::new(mock_repo);
        let query = SearchEntries {
            query:  "nonexistent".to_string(),
            limit:  None,
            offset: None,
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let search_result = result.unwrap();
        assert_eq!(search_result.items.len(), 0);
        assert_eq!(search_result.total_count, 0);
    }

    #[tokio::test]
    async fn test_search_entries_empty_query() {
        let mock_repo = MockRepository::new();

        let handler = Handler::new(mock_repo);
        let query = SearchEntries {
            query:  String::new(),
            limit:  None,
            offset: None,
        };

        let result = handler.handle(query).await;
        assert!(matches!(result, Err(super::Error::InvalidParameter(_))));
    }
}
