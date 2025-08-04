//! アプリケーション層
//!
//! コマンドハンドラーの実装

use std::sync::Arc;

use async_trait::async_trait;
use shared_error::DomainResult;
use shared_vocabulary_context::commands::*;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{
    domain::{aggregates::VocabularyEntry, services::VocabularyDomainService},
    ports::{
        inbound::CommandService,
        outbound::{EventBus, EventStore, VocabularyRepository},
    },
};

/// コマンドハンドラー
pub struct CommandHandler {
    #[allow(dead_code)]
    event_store: Arc<dyn EventStore>,
    #[allow(dead_code)]
    event_bus:   Arc<dyn EventBus>,
    repository:  Arc<dyn VocabularyRepository>,
}

impl CommandHandler {
    /// 新しいコマンドハンドラーを作成
    pub fn new(
        event_store: Arc<dyn EventStore>,
        event_bus: Arc<dyn EventBus>,
        repository: Arc<dyn VocabularyRepository>,
    ) -> Self {
        Self {
            event_store,
            event_bus,
            repository,
        }
    }

    /// 語彙項目作成コマンドを処理
    async fn handle_create_vocabulary_item(
        &self,
        command: CreateVocabularyItem,
    ) -> DomainResult<Uuid> {
        info!("Creating vocabulary item: {}", command.word);

        // 1. 既存エントリの確認
        let mut entry = match self.repository.find_by_word(&command.word).await? {
            Some(existing) => {
                // 既存エントリがある場合、重複チェック
                VocabularyDomainService::check_duplicate(
                    std::slice::from_ref(&existing),
                    &command.word,
                )?;
                existing
            },
            None => {
                // 新規エントリを作成
                VocabularyEntry::create(command.word.clone())?
            },
        };

        // 2. 項目を追加
        let item_id = entry.add_item(
            command.definitions,
            command.part_of_speech.into(),
            command.register.into(),
            command.domain.into(),
            *command.user_id.as_uuid(),
        )?;

        // 3. リポジトリに保存（Event Store への保存とイベント発行を含む）
        self.repository.save(&mut entry).await?;

        info!("Created vocabulary item with ID: {}", item_id);
        Ok(item_id)
    }

    /// 語彙項目更新コマンドを処理
    #[instrument(skip(self))]
    pub async fn handle_update_vocabulary_item(
        &self,
        command: UpdateVocabularyItem,
    ) -> DomainResult<()> {
        info!("Updating vocabulary item: {}", command.item_id);

        // TODO: 実装
        // 1. Event Store から現在の状態を読み込み
        // 2. ドメインモデルにコマンドを適用
        // 3. ビジネスルールを検証
        // 4. イベントを生成
        // 5. Event Store に保存
        // 6. Pub/Sub に発行

        Ok(())
    }

    /// 語彙項目削除コマンドを処理
    #[instrument(skip(self))]
    pub async fn handle_delete_vocabulary_item(
        &self,
        command: DeleteVocabularyItem,
    ) -> DomainResult<()> {
        info!("Deleting vocabulary item: {}", command.item_id);

        // TODO: 実装

        Ok(())
    }

    /// 例文追加コマンドを処理
    #[instrument(skip(self))]
    pub async fn handle_add_example(&self, command: AddExample) -> DomainResult<()> {
        info!("Adding example to vocabulary item: {}", command.item_id);

        // TODO: 実装

        Ok(())
    }

    /// AI エンリッチメント要求コマンドを処理
    #[instrument(skip(self))]
    pub async fn handle_request_ai_enrichment(
        &self,
        command: RequestAiEnrichment,
    ) -> DomainResult<()> {
        info!(
            "Requesting AI enrichment for vocabulary item: {}",
            command.item_id
        );

        // TODO: 実装
        // AI Context への統合イベントを発行

        Ok(())
    }
}

// CommandService trait の実装
#[async_trait]
impl CommandService for CommandHandler {
    #[instrument(skip(self))]
    async fn create_vocabulary_item(&self, command: CreateVocabularyItem) -> DomainResult<Uuid> {
        self.handle_create_vocabulary_item(command).await
    }

    #[instrument(skip(self))]
    async fn update_vocabulary_item(&self, command: UpdateVocabularyItem) -> DomainResult<()> {
        self.handle_update_vocabulary_item(command).await
    }

    #[instrument(skip(self))]
    async fn delete_vocabulary_item(&self, command: DeleteVocabularyItem) -> DomainResult<()> {
        self.handle_delete_vocabulary_item(command).await
    }

    #[instrument(skip(self))]
    async fn add_example(&self, command: AddExample) -> DomainResult<()> {
        self.handle_add_example(command).await
    }

    #[instrument(skip(self))]
    async fn request_ai_enrichment(&self, command: RequestAiEnrichment) -> DomainResult<()> {
        self.handle_request_ai_enrichment(command).await
    }
}
