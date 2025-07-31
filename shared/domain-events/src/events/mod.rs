//! ドメインイベント定義
//!
//! このモジュールは境界づけられたコンテキストごとに整理された全てのドメインイベントを含みます。

mod ai;
mod algorithm;
mod learning;
mod user;
mod vocabulary;

pub use ai::AIIntegrationEvent;
pub use algorithm::LearningAlgorithmEvent;
pub use learning::{CorrectnessJudgment, LearningEvent};
use serde::{Deserialize, Serialize};
pub use user::UserEvent;
pub use vocabulary::VocabularyEvent;

use crate::EventMetadata;

/// システム内の全てのドメインイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DomainEvent {
    /// 学習コンテキストのイベント
    Learning(LearningEvent),
    /// 学習アルゴリズムコンテキストのイベント
    Algorithm(LearningAlgorithmEvent),
    /// 語彙コンテキストのイベント
    Vocabulary(VocabularyEvent),
    /// AI 統合コンテキストのイベント
    AI(AIIntegrationEvent),
    /// ユーザーコンテキストのイベント
    User(UserEvent),
}

impl DomainEvent {
    /// イベントタイプを文字列として取得
    #[must_use]
    pub const fn event_type(&self) -> &'static str {
        match self {
            Self::Learning(_) => "Learning",
            Self::Algorithm(_) => "Algorithm",
            Self::Vocabulary(_) => "Vocabulary",
            Self::AI(_) => "AI",
            Self::User(_) => "User",
        }
    }

    /// イベントメタデータを取得
    #[must_use]
    pub const fn metadata(&self) -> &EventMetadata {
        match self {
            Self::Learning(e) => match e {
                LearningEvent::SessionStarted { metadata, .. }
                | LearningEvent::CorrectnessJudged { metadata, .. }
                | LearningEvent::SessionCompleted { metadata, .. } => metadata,
            },
            Self::Algorithm(e) => match e {
                LearningAlgorithmEvent::ReviewScheduleUpdated { metadata, .. }
                | LearningAlgorithmEvent::StatisticsUpdated { metadata, .. } => metadata,
            },
            Self::Vocabulary(e) => match e {
                VocabularyEvent::EntryCreated { metadata, .. }
                | VocabularyEvent::ItemCreated { metadata, .. }
                | VocabularyEvent::AIGenerationRequested { metadata, .. }
                | VocabularyEvent::AIInfoGenerated { metadata, .. } => metadata,
            },
            Self::AI(e) => match e {
                AIIntegrationEvent::TaskCreated { metadata, .. }
                | AIIntegrationEvent::TaskStarted { metadata, .. }
                | AIIntegrationEvent::TaskCompleted { metadata, .. }
                | AIIntegrationEvent::TaskFailed { metadata, .. } => metadata,
            },
            Self::User(e) => match e {
                UserEvent::AccountCreated { metadata, .. }
                | UserEvent::AccountDeleted { metadata, .. } => metadata,
            },
        }
    }
}
