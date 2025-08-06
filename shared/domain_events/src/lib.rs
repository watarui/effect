//! ドメインイベントインフラストラクチャ
//!
//! このモジュールは全ての境界づけられたコンテキストで使用される
//! ドメインイベントの基盤インフラストラクチャを提供します。

mod error;
mod traits;

// serde ヘルパーモジュール
pub mod serde_helpers;

// Proto 生成コードを含める
#[allow(warnings)]
#[allow(missing_docs)]
mod proto {
    pub mod effect {
        pub mod events {
            pub mod learning {
                include!(concat!(env!("OUT_DIR"), "/effect.events.learning.rs"));
            }
            pub mod vocabulary {
                include!(concat!(env!("OUT_DIR"), "/effect.events.vocabulary.rs"));
            }
            pub mod algorithm {
                include!(concat!(env!("OUT_DIR"), "/effect.events.algorithm.rs"));
            }
            pub mod ai {
                include!(concat!(env!("OUT_DIR"), "/effect.events.ai.rs"));
            }
            pub mod user {
                include!(concat!(env!("OUT_DIR"), "/effect.events.user.rs"));
            }
        }
        pub mod common {
            include!(concat!(env!("OUT_DIR"), "/effect.common.rs"));
        }
        pub mod services {
            pub mod user {
                include!(concat!(env!("OUT_DIR"), "/effect.services.user.rs"));
            }
        }
    }
}

// Proto 型を再エクスポート
pub use proto::effect::{
    common::{CefrLevel, CorrectnessJudgment, EventMetadata, UserRole},
    events::{ai::*, algorithm::*, learning::*, user::*, vocabulary::*},
    services::user::{LearningGoal, learning_goal},
};

// EventMetadata のヘルパー関数
impl EventMetadata {
    /// 新しいイベントメタデータを作成
    #[must_use]
    pub fn new(aggregate_id: String) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            aggregate_id,
            occurred_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            version: 1,
            caused_by_user_id: None,
            correlation_id: None,
            causation_id: None,
            trace_context: None,
            command_id: None,
            source: None,
            schema_version: Some(1),
        }
    }
}

// トレイトを再エクスポート
pub use error::EventError;
pub use traits::{EventBus, EventHandler};

// DomainEvent を定義（proto の各イベントをラップ）

/// システム内の全てのドメインイベント
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    /// 学習コンテキストのイベント
    Learning(LearningEvent),
    /// 学習アルゴリズムコンテキストのイベント
    Algorithm(LearningAlgorithmEvent),
    /// 語彙コンテキストのイベント
    Vocabulary(VocabularyEvent),
    /// AI 統合コンテキストのイベント
    AI(AiIntegrationEvent),
    /// ユーザーコンテキストのイベント
    User(UserEvent),
}

// イベントタイプ定数
const EVENT_TYPE_LEARNING: &str = "Learning";
const EVENT_TYPE_ALGORITHM: &str = "Algorithm";
const EVENT_TYPE_VOCABULARY: &str = "Vocabulary";
const EVENT_TYPE_AI: &str = "AI";
const EVENT_TYPE_USER: &str = "User";

// 便利なヘルパーメソッド
impl DomainEvent {
    /// イベントタイプを文字列として取得
    #[must_use]
    pub const fn event_type(&self) -> &'static str {
        match self {
            Self::Learning(_) => EVENT_TYPE_LEARNING,
            Self::Algorithm(_) => EVENT_TYPE_ALGORITHM,
            Self::Vocabulary(_) => EVENT_TYPE_VOCABULARY,
            Self::AI(_) => EVENT_TYPE_AI,
            Self::User(_) => EVENT_TYPE_USER,
        }
    }

    /// イベントメタデータを取得
    ///
    /// # Returns
    ///
    /// イベントメタデータが存在する場合は Some、存在しない場合は None
    #[must_use]
    pub const fn metadata(&self) -> Option<&EventMetadata> {
        match self {
            Self::Learning(e) => match &e.event {
                Some(learning_event::Event::SessionStarted(e)) => e.metadata.as_ref(),
                Some(learning_event::Event::ItemsSelected(e)) => e.metadata.as_ref(),
                Some(learning_event::Event::ItemPresented(e)) => e.metadata.as_ref(),
                Some(learning_event::Event::AnswerRevealed(e)) => e.metadata.as_ref(),
                Some(learning_event::Event::CorrectnessJudged(e)) => e.metadata.as_ref(),
                Some(learning_event::Event::CorrectAnswerProvided(e)) => e.metadata.as_ref(),
                Some(learning_event::Event::SessionCompleted(e)) => e.metadata.as_ref(),
                Some(learning_event::Event::SessionAbandoned(e)) => e.metadata.as_ref(),
                None => None,
            },
            Self::Algorithm(e) => match &e.event {
                Some(learning_algorithm_event::Event::ReviewScheduleUpdated(e)) => {
                    e.metadata.as_ref()
                },
                Some(learning_algorithm_event::Event::StatisticsUpdated(e)) => e.metadata.as_ref(),
                Some(learning_algorithm_event::Event::DifficultyAdjusted(e)) => e.metadata.as_ref(),
                Some(learning_algorithm_event::Event::PerformanceAnalyzed(e)) => {
                    e.metadata.as_ref()
                },
                Some(learning_algorithm_event::Event::StrategyAdjusted(e)) => e.metadata.as_ref(),
                Some(learning_algorithm_event::Event::ItemReviewed(e)) => e.metadata.as_ref(),
                None => None,
            },
            Self::Vocabulary(e) => match &e.event {
                Some(vocabulary_event::Event::EntryCreated(e)) => e.metadata.as_ref(),
                Some(vocabulary_event::Event::ItemCreated(e)) => e.metadata.as_ref(),
                Some(vocabulary_event::Event::FieldUpdated(e)) => e.metadata.as_ref(),
                Some(vocabulary_event::Event::AiGenerationRequested(e)) => e.metadata.as_ref(),
                Some(vocabulary_event::Event::AiGenerationCompleted(e)) => e.metadata.as_ref(),
                Some(vocabulary_event::Event::AiGenerationFailed(e)) => e.metadata.as_ref(),
                Some(vocabulary_event::Event::ItemPublished(e)) => e.metadata.as_ref(),
                Some(vocabulary_event::Event::UpdateConflicted(e)) => e.metadata.as_ref(),
                None => None,
            },
            Self::AI(e) => match &e.event {
                Some(ai_integration_event::Event::TaskCreated(e)) => e.metadata.as_ref(),
                Some(ai_integration_event::Event::TaskStarted(e)) => e.metadata.as_ref(),
                Some(ai_integration_event::Event::TaskCompleted(e)) => e.metadata.as_ref(),
                Some(ai_integration_event::Event::TaskFailed(e)) => e.metadata.as_ref(),
                Some(ai_integration_event::Event::TaskRetried(e)) => e.metadata.as_ref(),
                Some(ai_integration_event::Event::TaskCancelled(e)) => e.metadata.as_ref(),
                Some(ai_integration_event::Event::GenerationCancelled(e)) => e.metadata.as_ref(),
                Some(ai_integration_event::Event::ChatSessionStarted(e)) => e.metadata.as_ref(),
                Some(ai_integration_event::Event::ChatMessageSent(e)) => e.metadata.as_ref(),
                None => None,
            },
            Self::User(e) => match &e.event {
                Some(user_event::Event::UserSignedUp(e)) => e.metadata.as_ref(),
                Some(user_event::Event::ProfileUpdated(e)) => e.metadata.as_ref(),
                Some(user_event::Event::LearningGoalSet(e)) => e.metadata.as_ref(),
                Some(user_event::Event::UserRoleChanged(e)) => e.metadata.as_ref(),
                Some(user_event::Event::UserDeleted(e)) => e.metadata.as_ref(),
                Some(user_event::Event::UserSignedIn(e)) => e.metadata.as_ref(),
                Some(user_event::Event::UserSignedOut(e)) => e.metadata.as_ref(),
                Some(user_event::Event::SessionRefreshed(e)) => e.metadata.as_ref(),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let metadata = EventMetadata {
            event_id:          uuid::Uuid::new_v4().to_string(),
            aggregate_id:      uuid::Uuid::new_v4().to_string(),
            occurred_at:       Some(prost_types::Timestamp {
                seconds: 1_234_567_890,
                nanos:   0,
            }),
            version:           1,
            caused_by_user_id: None,
            correlation_id:    None,
            causation_id:      None,
            trace_context:     None,
            command_id:        None,
            source:            None,
            schema_version:    Some(1),
        };

        let event = DomainEvent::Learning(LearningEvent {
            event: Some(learning_event::Event::SessionStarted(SessionStarted {
                metadata:   Some(metadata),
                session_id: uuid::Uuid::new_v4().to_string(),
                user_id:    uuid::Uuid::new_v4().to_string(),
                item_count: 50,
                strategy:   0, // SelectionStrategy::SELECTION_STRATEGY_UNSPECIFIED
            })),
        });

        // JSON シリアライゼーションのテスト
        let json = serde_json::to_string(&event)?;
        println!("Serialized JSON: {json}");

        // JSONに必要なフィールドが含まれているか確認
        assert!(json.contains("\"type\":\"Learning\""));
        assert!(json.contains("\"sessionStarted\""));
        assert!(json.contains("\"itemCount\":50"));

        // デシリアライゼーションのテスト
        let deserialized: DomainEvent = serde_json::from_str(&json)?;

        // 元のイベントと同じ内容か確認
        match deserialized {
            DomainEvent::Learning(learning_event) => {
                if let Some(learning_event::Event::SessionStarted(session)) = &learning_event.event
                {
                    assert_eq!(session.item_count, 50);
                    Ok(())
                } else {
                    Err("Wrong event type".into())
                }
            },
            _ => Err("Wrong event type".into()),
        }
    }

    #[test]
    fn test_event_metadata_timestamp_serialization() -> Result<(), Box<dyn std::error::Error>> {
        use chrono::{DateTime, Utc};

        let now = DateTime::<Utc>::from_timestamp(1_609_459_200, 0).unwrap(); // 2021-01-01T00:00:00Z
        let metadata = EventMetadata {
            event_id:          "test-event-id".to_string(),
            aggregate_id:      "test-aggregate-id".to_string(),
            occurred_at:       Some(prost_types::Timestamp {
                seconds: now.timestamp(),
                nanos:   0,
            }),
            version:           1,
            caused_by_user_id: Some("user-123".to_string()),
            correlation_id:    Some("correlation-123".to_string()),
            causation_id:      None,
            trace_context:     None,
            command_id:        None,
            source:            None,
            schema_version:    Some(1),
        };

        // メタデータのシリアライゼーションテスト
        let event = DomainEvent::User(UserEvent {
            event: Some(user_event::Event::UserSignedUp(UserSignedUp {
                metadata:     Some(metadata),
                user_id:      "user-123".to_string(),
                email:        "test@example.com".to_string(),
                display_name: "Test User".to_string(),
                photo_url:    None,
                initial_role: UserRole::User as i32,
                created_at:   Some(prost_types::Timestamp {
                    seconds: now.timestamp(),
                    nanos:   0,
                }),
            })),
        });

        let json = serde_json::to_string(&event)?;
        println!("Serialized Event with Timestamp: {json}");

        // タイムスタンプが正しくシリアライズされているか確認
        assert!(json.contains("2021-01-01T00:00:00"));
        assert!(json.contains("\"eventId\":\"test-event-id\""));
        assert!(json.contains("\"aggregateId\":\"test-aggregate-id\""));

        // デシリアライゼーションが正しく動作するか確認
        let deserialized: DomainEvent = serde_json::from_str(&json)?;

        match deserialized {
            DomainEvent::User(user_event) => {
                if let Some(user_event::Event::UserSignedUp(signed_up)) = &user_event.event {
                    let meta = signed_up.metadata.as_ref().unwrap();
                    assert_eq!(meta.event_id, "test-event-id");
                    assert_eq!(meta.aggregate_id, "test-aggregate-id");
                    assert_eq!(meta.occurred_at.as_ref().unwrap().seconds, 1_609_459_200);
                    Ok(())
                } else {
                    Err("Wrong event type".into())
                }
            },
            _ => Err("Wrong event type".into()),
        }
    }
}
