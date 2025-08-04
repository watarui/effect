//! Pub/Sub 実装

use async_trait::async_trait;
use shared_error::DomainResult;

use crate::{domain::events::VocabularyDomainEvent, ports::outbound::EventBus};

/// Google Pub/Sub 実装
pub struct PubSubEventBus {
    // TODO: Pub/Sub クライアントを追加
    // client: google_cloud_pubsub::client::Client,
}

impl Default for PubSubEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl PubSubEventBus {
    /// 新しい EventBus を作成
    pub fn new() -> Self {
        Self {
            // TODO: client を注入
        }
    }
}

#[async_trait]
impl EventBus for PubSubEventBus {
    async fn publish(&self, events: Vec<VocabularyDomainEvent>) -> DomainResult<()> {
        // TODO: 実装
        // 1. イベントを Proto メッセージに変換
        // 2. Pub/Sub に発行
        for event in events {
            tracing::info!("Publishing event: {:?}", event);
        }
        Ok(())
    }
}
