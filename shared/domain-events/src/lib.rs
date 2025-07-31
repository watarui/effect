//! ドメインイベントインフラストラクチャ
//!
//! このモジュールは全ての境界づけられたコンテキストで使用される
//! ドメインイベントの基盤インフラストラクチャを提供します。

mod error;
mod events;
mod metadata;
mod traits;

// Re-export all public types
pub use error::EventError;
pub use events::{
    AIIntegrationEvent,
    CorrectnessJudgment,
    DomainEvent,
    LearningAlgorithmEvent,
    LearningEvent,
    UserEvent,
    VocabularyEvent,
};
pub use metadata::EventMetadata;
pub use traits::{EventBus, EventHandler, EventStore};

#[cfg(test)]
mod tests {
    use common_types::{SessionId, UserId};

    use super::*;

    #[test]
    fn test_event_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let event = DomainEvent::Learning(LearningEvent::SessionStarted {
            metadata:   EventMetadata::new(),
            session_id: SessionId::new(),
            user_id:    UserId::new(),
            item_count: 50,
        });

        let json = serde_json::to_string(&event)?;
        let deserialized: DomainEvent = serde_json::from_str(&json)?;

        match deserialized {
            DomainEvent::Learning(LearningEvent::SessionStarted { item_count, .. }) => {
                assert_eq!(item_count, 50);
                Ok(())
            },
            _ => Err("Wrong event type".into()),
        }
    }
}
