//! インメモリイベントパブリッシャー（開発環境用）

use async_trait::async_trait;
use domain_events::DomainEvent;
use thiserror::Error;
use tracing::{debug, info};

use crate::ports::outbound::EventPublisher;

/// イベントパブリッシャーエラー
#[derive(Error, Debug)]
pub enum Error {
    /// 内部エラー
    #[error("Event publisher error: {0}")]
    Internal(String),
}

/// インメモリイベントパブリッシャー
///
/// 開発環境用のイベントパブリッシャー実装。
/// イベントをログに出力するだけで、実際の永続化や配信は行わない。
#[derive(Debug, Clone, Default)]
pub struct InMemoryPublisher;

impl InMemoryPublisher {
    /// 新しいインメモリパブリッシャーを作成
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventPublisher for InMemoryPublisher {
    type Error = Error;

    async fn publish(&self, event: &DomainEvent) -> Result<(), Self::Error> {
        // イベントタイプとメタデータをログ出力
        let metadata = event.metadata();

        info!(
            event_type = event.event_type(),
            event_id = %metadata.event_id,
            occurred_at = %metadata.occurred_at,
            version = %metadata.version,
            "Publishing domain event"
        );

        // イベントの詳細をデバッグログに出力
        match event {
            DomainEvent::User(user_event) => {
                debug!("User event details: {:?}", user_event);
            },
            DomainEvent::Learning(learning_event) => {
                debug!("Learning event details: {:?}", learning_event);
            },
            DomainEvent::Algorithm(algorithm_event) => {
                debug!("Algorithm event details: {:?}", algorithm_event);
            },
            DomainEvent::Vocabulary(vocabulary_event) => {
                debug!("Vocabulary event details: {:?}", vocabulary_event);
            },
            DomainEvent::AI(ai_event) => {
                debug!("AI event details: {:?}", ai_event);
            },
        }

        // 開発環境では常に成功を返す
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common_types::UserId;
    use domain_events::{EventMetadata, UserEvent};

    use super::*;

    #[tokio::test]
    async fn publish_user_event_should_succeed() {
        // Given
        let publisher = InMemoryPublisher::new();
        let event = DomainEvent::User(UserEvent::AccountCreated {
            metadata: EventMetadata::new(),
            user_id:  UserId::new(),
            email:    "test@example.com".to_string(),
        });

        // When
        let result = publisher.publish(&event).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn publish_learning_event_should_succeed() {
        // Given
        let publisher = InMemoryPublisher::new();
        let event = DomainEvent::Learning(domain_events::LearningEvent::SessionStarted {
            metadata:   EventMetadata::new(),
            session_id: common_types::SessionId::new(),
            user_id:    UserId::new(),
            item_count: 0,
        });

        // When
        let result = publisher.publish(&event).await;

        // Then
        assert!(result.is_ok());
    }
}
