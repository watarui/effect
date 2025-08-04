//! イベントハンドラー実装
//!
//! ドメインイベントを処理し、Read Model を更新する

use std::sync::Arc;

use async_trait::async_trait;
use shared_error::{DomainError, DomainResult};
use uuid::Uuid;

use crate::{
    domain::{
        projections::{ProjectionStateBuilder, VocabularyItemViewBuilder},
        read_models::{DefinitionData, ExampleData},
    },
    ports::{
        inbound::EventHandler,
        outbound::{ProjectionStateRepository, ReadModelRepository},
    },
};

/// Vocabulary イベントハンドラー
pub struct VocabularyEventHandler {
    _read_model_repo: Arc<dyn ReadModelRepository>,
    projection_repo:  Arc<dyn ProjectionStateRepository>,
    projection_name:  String,
}

impl VocabularyEventHandler {
    /// 新しいイベントハンドラーを作成
    pub fn new(
        read_model_repo: Arc<dyn ReadModelRepository>,
        projection_repo: Arc<dyn ProjectionStateRepository>,
    ) -> Self {
        Self {
            _read_model_repo: read_model_repo,
            projection_repo,
            projection_name: "vocabulary_items_view".to_string(),
        }
    }

    /// プロジェクション状態を更新（成功時）
    async fn update_projection_success(
        &self,
        event_id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        position: i64,
    ) -> DomainResult<()> {
        let state = self
            .projection_repo
            .get_state(&self.projection_name)
            .await?
            .unwrap_or_else(|| ProjectionStateBuilder::create(&self.projection_name));

        let updated_state =
            ProjectionStateBuilder::update_success(state, event_id, timestamp, position);

        self.projection_repo.save_state(&updated_state).await?;
        Ok(())
    }

    /// プロジェクション状態を更新（エラー時）
    async fn _update_projection_error(&self, error: &str) -> DomainResult<()> {
        let state = self
            .projection_repo
            .get_state(&self.projection_name)
            .await?
            .unwrap_or_else(|| ProjectionStateBuilder::create(&self.projection_name));

        let updated_state = ProjectionStateBuilder::update_error(state, error);
        self.projection_repo.save_state(&updated_state).await?;
        Ok(())
    }

    /// エントリ作成イベントを処理
    async fn _handle_entry_created(
        &self,
        entry_id: Uuid,
        spelling: String,
        _occurred_at: chrono::DateTime<chrono::Utc>,
    ) -> DomainResult<()> {
        // Vocabulary Context ではエントリだけでは Read Model を作成しない
        // ItemAddedToEntry イベントが来たときに初めて作成する
        tracing::info!("Entry created: {} ({})", spelling, entry_id);
        Ok(())
    }

    /// 項目追加イベントを処理
    #[allow(clippy::too_many_arguments)]
    async fn _handle_item_added_to_entry(
        &self,
        item_id: Uuid,
        entry_id: Uuid,
        spelling: String,
        disambiguation: String,
        created_by_type: String,
        created_by_id: Option<Uuid>,
        occurred_at: chrono::DateTime<chrono::Utc>,
    ) -> DomainResult<()> {
        // 新しい項目ビューを作成
        let view = VocabularyItemViewBuilder::from_item_added(
            item_id,
            entry_id,
            spelling,
            disambiguation,
            created_by_type,
            created_by_id,
            occurred_at,
        )?;

        self._read_model_repo.save_item_view(&view).await?;
        Ok(())
    }

    /// 定義追加イベントを処理
    #[allow(clippy::too_many_arguments)]
    async fn _handle_definition_added(
        &self,
        item_id: Uuid,
        definition_id: Uuid,
        part_of_speech: String,
        meaning: String,
        meaning_translation: Option<String>,
        domain: Option<String>,
        register: Option<String>,
    ) -> DomainResult<()> {
        // 既存のビューを取得
        let view = self
            ._read_model_repo
            .get_item_view(item_id)
            .await?
            .ok_or_else(|| DomainError::not_found("VocabularyItemView", item_id))?;

        // 定義を追加
        let definition = DefinitionData {
            id: definition_id,
            part_of_speech,
            meaning,
            meaning_translation,
            domain,
            register,
            examples: Vec::new(),
        };

        let updated_view = VocabularyItemViewBuilder::add_definition(view, definition);
        self._read_model_repo
            .update_item_view(&updated_view)
            .await?;
        Ok(())
    }

    /// 例文追加イベントを処理
    async fn _handle_example_added(
        &self,
        item_id: Uuid,
        definition_id: Uuid,
        example_id: Uuid,
        example_text: String,
        example_translation: Option<String>,
    ) -> DomainResult<()> {
        // 既存のビューを取得
        let mut view = self
            ._read_model_repo
            .get_item_view(item_id)
            .await?
            .ok_or_else(|| DomainError::not_found("VocabularyItemView", item_id))?;

        // 対象の定義を見つけて例文を追加
        let definitions = &mut view.definitions.0;
        if let Some(def) = definitions.iter_mut().find(|d| d.id == definition_id) {
            def.examples.push(ExampleData {
                id: example_id,
                example_text,
                example_translation,
            });

            // 例文数を再計算
            view.example_count = definitions.iter().map(|d| d.examples.len() as i32).sum();

            view.version += 1;
            view.last_modified_at = chrono::Utc::now();

            self._read_model_repo.update_item_view(&view).await?;
        }

        Ok(())
    }

    /// ステータス変更イベントを処理
    async fn _handle_item_status_changed(
        &self,
        item_id: Uuid,
        status: String,
        modified_by: Uuid,
    ) -> DomainResult<()> {
        let view = self
            ._read_model_repo
            .get_item_view(item_id)
            .await?
            .ok_or_else(|| DomainError::not_found("VocabularyItemView", item_id))?;

        let updated_view = VocabularyItemViewBuilder::update_status(view, status, modified_by);
        self._read_model_repo
            .update_item_view(&updated_view)
            .await?;
        Ok(())
    }

    /// 項目削除イベントを処理
    async fn _handle_item_deleted(&self, item_id: Uuid) -> DomainResult<()> {
        self._read_model_repo.delete_item_view(item_id).await?;
        Ok(())
    }
}

#[async_trait]
impl EventHandler for VocabularyEventHandler {
    async fn handle_event(&self, event_data: Vec<u8>) -> DomainResult<()> {
        // Proto メッセージのデシリアライズと処理を実装
        // 現時点では、実際のイベント処理ロジックを後で実装する

        tracing::info!("Handling event: {} bytes", event_data.len());

        // TODO: 実際のイベント処理の実装
        // 1. VocabularyEvent をデシリアライズ
        // 2. イベントタイプに応じて処理を振り分け
        // 3. プロジェクション状態を更新

        // 仮の成功処理
        let event_id = Uuid::new_v4();
        let timestamp = chrono::Utc::now();
        let position = 0;

        self.update_projection_success(event_id, timestamp, position)
            .await?;

        Ok(())
    }
}
