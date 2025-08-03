//! アプリケーションサービス

use std::sync::Arc;

use crate::{
    application::{
        command_handlers::{
            create_item::{CommandHandler, Error as CreateItemError, Handler as CreateItemHandler},
            delete_item::{Error as DeleteItemError, Handler as DeleteItemHandler},
            update_item::{Error as UpdateItemError, Handler as UpdateItemHandler},
        },
        query_handlers::{
            get_item::{Error as GetItemError, Handler as GetItemHandler, QueryHandler},
            search_entries::{
                Error as SearchEntriesError,
                Handler as SearchEntriesHandler,
                SearchResult,
            },
        },
    },
    domain::{
        commands::{CreateItem, DeleteItem, UpdateItem},
        entities::vocabulary_item::VocabularyItem,
    },
    ports::outbound::repository::Repository,
};

/// 語彙サービスのアプリケーション層
pub struct VocabularyService<R> {
    create_item:    Arc<CreateItemHandler<R>>,
    update_item:    Arc<UpdateItemHandler<R>>,
    delete_item:    Arc<DeleteItemHandler<R>>,
    get_item:       Arc<GetItemHandler<R>>,
    search_entries: Arc<SearchEntriesHandler<R>>,
}

impl<R> VocabularyService<R>
where
    R: Repository + Send + Sync + Clone + 'static,
{
    /// 新しいサービスを作成
    pub fn new(repository: R) -> Self {
        Self {
            create_item:    Arc::new(CreateItemHandler::new(repository.clone())),
            update_item:    Arc::new(UpdateItemHandler::new(repository.clone())),
            delete_item:    Arc::new(DeleteItemHandler::new(repository.clone())),
            get_item:       Arc::new(GetItemHandler::new(repository.clone())),
            search_entries: Arc::new(SearchEntriesHandler::new(repository)),
        }
    }

    /// 語彙項目を作成
    ///
    /// # Errors
    ///
    /// - `CreateItemError::AlreadyExists` - 同じ単語の項目が既に存在する場合
    /// - `CreateItemError::Repository` - リポジトリ操作が失敗した場合
    /// - `CreateItemError::Domain` - ドメインエラーが発生した場合
    pub async fn create_item(
        &self,
        command: CreateItem,
    ) -> Result<VocabularyItem, CreateItemError> {
        self.create_item.handle(command).await
    }

    /// 語彙項目を更新
    ///
    /// # Errors
    ///
    /// - `UpdateItemError::NotFound` - 指定された項目が存在しない場合
    /// - `UpdateItemError::VersionMismatch` - 楽観的ロックが失敗した場合
    /// - `UpdateItemError::Repository` - リポジトリ操作が失敗した場合
    /// - `UpdateItemError::Domain` - ドメインエラーが発生した場合
    pub async fn update_item(
        &self,
        command: UpdateItem,
    ) -> Result<VocabularyItem, UpdateItemError> {
        self.update_item.handle(command).await
    }

    /// 語彙項目を削除
    ///
    /// # Errors
    ///
    /// - `DeleteItemError::NotFound` - 指定された項目が存在しない場合
    /// - `DeleteItemError::Repository` - リポジトリ操作が失敗した場合
    pub async fn delete_item(&self, command: DeleteItem) -> Result<(), DeleteItemError> {
        self.delete_item.handle(command).await
    }

    /// ID で語彙項目を取得
    ///
    /// # Errors
    ///
    /// - `GetItemError::NotFound` - 指定された項目が存在しない場合
    /// - `GetItemError::Repository` - リポジトリ操作が失敗した場合
    pub async fn get_item(&self, id: common_types::ItemId) -> Result<VocabularyItem, GetItemError> {
        self.get_item
            .handle(crate::application::queries::GetItem { item_id: id })
            .await
    }

    /// 語彙エントリーを検索
    ///
    /// # Errors
    ///
    /// - `SearchEntriesError::InvalidParameter` -
    ///   無効なパラメータが指定された場合
    /// - `SearchEntriesError::Repository` - リポジトリ操作が失敗した場合
    pub async fn search_entries(
        &self,
        query: String,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<SearchResult, SearchEntriesError> {
        self.search_entries
            .handle(crate::application::queries::SearchEntries {
                query,
                limit,
                offset,
            })
            .await
    }
}

impl<R> Clone for VocabularyService<R> {
    fn clone(&self) -> Self {
        Self {
            create_item:    Arc::clone(&self.create_item),
            update_item:    Arc::clone(&self.update_item),
            delete_item:    Arc::clone(&self.delete_item),
            get_item:       Arc::clone(&self.get_item),
            search_entries: Arc::clone(&self.search_entries),
        }
    }
}
