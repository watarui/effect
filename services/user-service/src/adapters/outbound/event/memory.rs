//! インメモリイベントパブリッシャー（開発環境用）

use std::sync::Arc;

use async_trait::async_trait;
use domain_events::DomainEvent;
use thiserror::Error;
use tokio::sync::Mutex;
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
/// イベントをログに出力し、テスト用に発行されたイベントを保存する。
#[derive(Debug, Clone)]
pub struct InMemoryPublisher {
    /// 発行されたイベントを保存するストレージ（テスト用）
    events: Arc<Mutex<Vec<DomainEvent>>>,
}

impl Default for InMemoryPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryPublisher {
    /// 新しいインメモリパブリッシャーを作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 発行されたイベントの一覧を取得（テスト用）
    pub async fn get_published_events(&self) -> Vec<DomainEvent> {
        let events = self.events.lock().await;
        events.clone()
    }

    /// 発行されたイベントをクリア（テスト用）
    pub async fn clear_events(&self) {
        let mut events = self.events.lock().await;
        events.clear();
    }
}

#[async_trait]
impl EventPublisher for InMemoryPublisher {
    type Error = Error;

    async fn publish(&self, event: &DomainEvent) -> Result<(), Self::Error> {
        // イベントタイプとメタデータをログ出力
        match event.metadata() {
            Some(metadata) => {
                info!(
                    event_type = event.event_type(),
                    event_id = %metadata.event_id,
                    occurred_at = ?metadata.occurred_at,
                    version = metadata.version,
                    "Publishing domain event"
                );
            },
            None => {
                info!(
                    event_type = event.event_type(),
                    "Publishing domain event (no metadata)"
                );
            },
        }

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

        // テスト用にイベントを保存
        {
            let mut events = self.events.lock().await;
            events.push(event.clone());
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
        let user_id = UserId::new();
        let event = DomainEvent::User(UserEvent {
            event: Some(domain_events::user_event::Event::UserSignedUp(
                domain_events::UserSignedUp {
                    metadata:     Some(EventMetadata::new(user_id.to_string())),
                    user_id:      user_id.to_string(),
                    email:        String::from("test@example.com"),
                    display_name: String::from("Test User"),
                    photo_url:    None,
                    initial_role: domain_events::UserRole::User as i32,
                    created_at:   Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
                },
            )),
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
        let user_id = UserId::new();
        let event = DomainEvent::Learning(domain_events::LearningEvent {
            event: Some(domain_events::learning_event::Event::SessionStarted(
                domain_events::SessionStarted {
                    metadata:   Some(EventMetadata::new(user_id.to_string())),
                    session_id: common_types::SessionId::new().to_string(),
                    user_id:    user_id.to_string(),
                    item_count: 0,
                },
            )),
        });

        // When
        let result = publisher.publish(&event).await;

        // Then
        assert!(result.is_ok());
    }
}
