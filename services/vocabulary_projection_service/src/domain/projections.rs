//! プロジェクション
//!
//! イベントから Read Model を構築するロジック

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use shared_error::{DomainError, DomainResult};
use uuid::Uuid;

use super::read_models::{CollocationData, DefinitionData, ProjectionState, VocabularyItemView};

/// プロジェクションの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionStatus {
    Initialized,
    Running,
    Paused,
    Failed,
    Rebuilding,
}

impl From<String> for ProjectionStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "initialized" => Self::Initialized,
            "running" => Self::Running,
            "paused" => Self::Paused,
            "failed" => Self::Failed,
            "rebuilding" => Self::Rebuilding,
            _ => Self::Failed,
        }
    }
}

impl From<ProjectionStatus> for String {
    fn from(status: ProjectionStatus) -> Self {
        match status {
            ProjectionStatus::Initialized => "initialized".to_string(),
            ProjectionStatus::Running => "running".to_string(),
            ProjectionStatus::Paused => "paused".to_string(),
            ProjectionStatus::Failed => "failed".to_string(),
            ProjectionStatus::Rebuilding => "rebuilding".to_string(),
        }
    }
}

/// プロジェクション状態ビルダー
pub struct ProjectionStateBuilder;

impl ProjectionStateBuilder {
    /// 新しいプロジェクション状態を作成
    pub fn create(projection_name: &str) -> ProjectionState {
        ProjectionState {
            projection_name:          projection_name.to_string(),
            last_processed_event_id:  None,
            last_processed_timestamp: None,
            event_store_position:     None,
            status:                   ProjectionStatus::Initialized.into(),
            error_count:              0,
            last_error:               None,
            updated_at:               Utc::now(),
        }
    }

    /// イベント処理成功時の更新
    pub fn update_success(
        mut state: ProjectionState,
        event_id: Uuid,
        timestamp: DateTime<Utc>,
        position: i64,
    ) -> ProjectionState {
        state.last_processed_event_id = Some(event_id);
        state.last_processed_timestamp = Some(timestamp);
        state.event_store_position = Some(position);
        state.status = ProjectionStatus::Running.into();
        state.error_count = 0; // エラーカウントをリセット
        state.last_error = None;
        state.updated_at = Utc::now();
        state
    }

    /// エラー発生時の更新
    pub fn update_error(mut state: ProjectionState, error: &str) -> ProjectionState {
        state.error_count += 1;
        state.last_error = Some(error.to_string());
        if state.error_count > 10 {
            state.status = ProjectionStatus::Failed.into();
        }
        state.updated_at = Utc::now();
        state
    }
}

/// 語彙項目ビューのビルダー
pub struct VocabularyItemViewBuilder;

impl VocabularyItemViewBuilder {
    /// EntryCreated イベントから初期状態を作成
    pub fn from_entry_created(
        _entry_id: Uuid,
        _spelling: String,
        _created_at: DateTime<Utc>,
    ) -> DomainResult<VocabularyItemView> {
        Err(DomainError::InvalidState(
            "Cannot create item view from entry created event alone".to_string(),
        ))
    }

    /// ItemAddedToEntry イベントから作成
    pub fn from_item_added(
        item_id: Uuid,
        entry_id: Uuid,
        spelling: String,
        disambiguation: String,
        created_by_type: String,
        created_by_id: Option<Uuid>,
        created_at: DateTime<Utc>,
    ) -> DomainResult<VocabularyItemView> {
        Ok(VocabularyItemView {
            item_id,
            entry_id,
            spelling,
            disambiguation,
            pronunciation: None,
            phonetic_respelling: None,
            audio_url: None,
            register: None,
            cefr_level: None,
            definitions: sqlx::types::Json(Vec::new()),
            synonyms: None,
            antonyms: None,
            collocations: None,
            definition_count: 0,
            example_count: 0,
            quality_score: None,
            status: "draft".to_string(),
            created_by_type,
            created_by_id,
            created_at,
            last_modified_at: created_at,
            last_modified_by: created_by_id.unwrap_or(Uuid::nil()),
            version: 1,
        })
    }

    /// 定義を追加
    pub fn add_definition(
        mut view: VocabularyItemView,
        definition: DefinitionData,
    ) -> VocabularyItemView {
        view.definitions.0.push(definition);
        view.definition_count = view.definitions.0.len() as i32;

        // 例文数を再計算
        view.example_count = view
            .definitions
            .0
            .iter()
            .map(|d| d.examples.len() as i32)
            .sum();

        view.version += 1;
        view.last_modified_at = Utc::now();
        view
    }

    /// 同義語を設定
    pub fn set_synonyms(
        mut view: VocabularyItemView,
        synonyms: HashMap<String, Vec<String>>,
    ) -> VocabularyItemView {
        view.synonyms = Some(sqlx::types::Json(synonyms));
        view.version += 1;
        view.last_modified_at = Utc::now();
        view
    }

    /// 対義語を設定
    pub fn set_antonyms(
        mut view: VocabularyItemView,
        antonyms: HashMap<String, Vec<String>>,
    ) -> VocabularyItemView {
        view.antonyms = Some(sqlx::types::Json(antonyms));
        view.version += 1;
        view.last_modified_at = Utc::now();
        view
    }

    /// コロケーションを追加
    pub fn add_collocations(
        mut view: VocabularyItemView,
        collocations: Vec<CollocationData>,
    ) -> VocabularyItemView {
        let mut existing = view.collocations.map(|c| c.0).unwrap_or_default();
        existing.extend(collocations);

        view.collocations = Some(sqlx::types::Json(existing));
        view.version += 1;
        view.last_modified_at = Utc::now();
        view
    }

    /// ステータスを更新
    pub fn update_status(
        mut view: VocabularyItemView,
        status: String,
        modified_by: Uuid,
    ) -> VocabularyItemView {
        view.status = status;
        view.last_modified_by = modified_by;
        view.version += 1;
        view.last_modified_at = Utc::now();
        view
    }
}
