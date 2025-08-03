//! イベントハンドラー実装
//!
//! ドメインイベントを処理してプロジェクションを更新

use shared_error::DomainResult;
use shared_vocabulary_context::events::proto::{
    EntryCreated,
    FieldUpdated,
    ItemCreated,
    ItemPublished,
};
use tracing::{info, instrument};

/// Vocabulary イベントハンドラー
pub struct VocabularyEventHandler {
    // TODO: プロジェクションストアへの参照を追加
    // projection_store: Arc<dyn ProjectionStore>,
}

impl VocabularyEventHandler {
    /// 新しいイベントハンドラーを作成
    pub fn new() -> Self {
        Self {
            // TODO: 依存関係を注入
        }
    }

    /// EntryCreated イベントを処理
    #[instrument(skip(self))]
    pub async fn handle_entry_created(&self, event: &EntryCreated) -> DomainResult<()> {
        info!("Processing EntryCreated event");

        // TODO: 実装
        // 1. イベントからプロジェクションモデルを作成
        // 2. PostgreSQL に保存

        Ok(())
    }

    /// ItemCreated イベントを処理
    #[instrument(skip(self))]
    pub async fn handle_item_created(&self, event: &ItemCreated) -> DomainResult<()> {
        info!("Processing ItemCreated event for item: {}", event.item_id);

        // TODO: 実装
        // 1. イベントからプロジェクションモデルを作成
        // 2. PostgreSQL に保存

        Ok(())
    }

    /// FieldUpdated イベントを処理
    #[instrument(skip(self))]
    pub async fn handle_field_updated(&self, event: &FieldUpdated) -> DomainResult<()> {
        info!("Processing FieldUpdated event for item: {}", event.item_id);

        // TODO: 実装
        // 1. 既存のプロジェクションを取得
        // 2. 更新内容を適用
        // 3. PostgreSQL に保存

        Ok(())
    }

    /// ItemPublished イベントを処理
    #[instrument(skip(self))]
    pub async fn handle_item_published(&self, event: &ItemPublished) -> DomainResult<()> {
        info!("Processing ItemPublished event for item: {}", event.item_id);

        // TODO: 実装
        // 1. プロジェクションのステータスを更新
        // 2. 公開日時を記録

        Ok(())
    }
}

impl Default for VocabularyEventHandler {
    fn default() -> Self {
        Self::new()
    }
}
