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
pub use user::{CefrLevel, LearningGoal, UserEvent};
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
                | UserEvent::AccountDeleted { metadata, .. }
                | UserEvent::ProfileUpdated { metadata, .. } => metadata,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use common_types::{SessionId, UserId};

    use super::*;

    #[test]
    fn domain_event_should_have_correct_event_type() {
        let learning_event = DomainEvent::Learning(LearningEvent::SessionStarted {
            metadata:   EventMetadata::new(),
            session_id: SessionId::new(),
            user_id:    UserId::new(),
            item_count: 50,
        });

        assert_eq!(learning_event.event_type(), "Learning");
    }

    #[test]
    fn domain_event_should_serialize_with_type_tag() {
        let event = DomainEvent::Learning(LearningEvent::SessionStarted {
            metadata:   EventMetadata::new(),
            session_id: SessionId::new(),
            user_id:    UserId::new(),
            item_count: 50,
        });

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"Learning\""));
    }

    #[test]
    fn domain_event_should_deserialize_correctly() {
        let original = DomainEvent::User(UserEvent::AccountCreated {
            metadata: EventMetadata::new(),
            user_id:  UserId::new(),
            email:    "test@example.com".to_string(),
        });

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: DomainEvent = serde_json::from_str(&json).unwrap();

        match deserialized {
            DomainEvent::User(UserEvent::AccountCreated { email, .. }) => {
                assert_eq!(email, "test@example.com");
            },
            _ => unreachable!("Expected User AccountCreated event"),
        }
    }

    #[test]
    fn profile_updated_event_should_work_in_domain_event() {
        use chrono::Utc;

        use super::user::{CefrLevel, LearningGoal};

        // Given
        let event = DomainEvent::User(UserEvent::ProfileUpdated {
            metadata:              EventMetadata::new(),
            user_id:               UserId::new(),
            display_name:          Some("Test User".to_string()),
            current_level:         Some(CefrLevel::B2),
            learning_goal:         Some(LearningGoal::GeneralLevel(CefrLevel::C1)),
            questions_per_session: Some(25),
            occurred_at:           Utc::now(),
        });

        // When
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: DomainEvent = serde_json::from_str(&json).unwrap();

        // Then
        assert_eq!(event.event_type(), "User");
        match deserialized {
            DomainEvent::User(UserEvent::ProfileUpdated {
                display_name,
                current_level,
                learning_goal,
                questions_per_session,
                ..
            }) => {
                assert_eq!(display_name, Some("Test User".to_string()));
                assert_eq!(current_level, Some(CefrLevel::B2));
                assert_eq!(
                    learning_goal,
                    Some(LearningGoal::GeneralLevel(CefrLevel::C1))
                );
                assert_eq!(questions_per_session, Some(25));
            },
            _ => unreachable!("Expected User ProfileUpdated event"),
        }
    }

    #[test]
    fn metadata_should_be_accessible() {
        let metadata = EventMetadata::new();
        let event = DomainEvent::Vocabulary(VocabularyEvent::ItemCreated {
            metadata:   metadata.clone(),
            item_id:    common_types::ItemId::new(),
            spelling:   "test".to_string(),
            created_by: UserId::new(),
        });

        assert_eq!(event.metadata().event_id, metadata.event_id);
    }
}
