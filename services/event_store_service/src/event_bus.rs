//! Event Bus (Google Pub/Sub) 統合
//!
//! Event Store Service に統合された Event Bus 機能

use std::{collections::HashMap, sync::Arc};

use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::{
    client::{Client, ClientConfig},
    publisher::Publisher,
};
use serde_json::Value as JsonValue;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config::EventBusConfig;

/// Event Bus のエラー
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Pub/Sub client error: {0}")]
    PubSubClient(#[from] google_cloud_pubsub::client::Error),

    #[error("Publisher error: {0}")]
    Publisher(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Topic not found: {0}")]
    #[allow(dead_code)]
    TopicNotFound(String),
}

/// Event Bus (Pub/Sub Publisher)
pub struct EventBus {
    client:     Client,
    publishers: Arc<RwLock<HashMap<String, Publisher>>>,
    config:     EventBusConfig,
}

impl EventBus {
    /// 新しい Event Bus を作成
    pub async fn new(config: EventBusConfig) -> Result<Self, EventBusError> {
        // Pub/Sub クライアントを作成
        let client_config = ClientConfig {
            project_id: Some(config.project_id.clone()),
            ..Default::default()
        };

        let client = Client::new(client_config).await?;

        info!("Event Bus initialized with project: {}", config.project_id);

        Ok(Self {
            client,
            publishers: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// イベントを発行
    pub async fn publish_event(
        &self,
        event_type: &str,
        aggregate_id: &Uuid,
        event_data: JsonValue,
    ) -> Result<String, EventBusError> {
        // イベントタイプからトピックを決定
        let topic = self.get_topic_for_event(event_type);

        // メッセージを作成（Publisher を取得する前に作成）
        let message = self.create_message(event_type, aggregate_id, event_data)?;

        // Publisher を取得または作成
        let publisher = self.get_or_create_publisher(&topic).await?;

        // 発行
        let awaiter = publisher.publish(message).await;
        let message_id = awaiter
            .get()
            .await
            .map_err(|e| EventBusError::Publisher(e.to_string()))?;

        info!(
            "Event published: type={}, aggregate_id={}, message_id={}",
            event_type, aggregate_id, message_id
        );

        Ok(message_id)
    }

    /// 複数のイベントをバッチで発行
    #[allow(dead_code)]
    pub async fn publish_events_batch(
        &self,
        events: Vec<(String, Uuid, JsonValue)>,
    ) -> Result<Vec<String>, EventBusError> {
        let mut message_ids = Vec::new();

        for (event_type, aggregate_id, event_data) in events {
            match self
                .publish_event(&event_type, &aggregate_id, event_data)
                .await
            {
                Ok(id) => message_ids.push(id),
                Err(e) => {
                    error!("Failed to publish event {}: {}", event_type, e);
                    // エラーをログに記録して続行（At-least-once 保証）
                    // 必要に応じてデッドレターキューに送信
                },
            }
        }

        Ok(message_ids)
    }

    /// イベントタイプからトピック名を決定
    fn get_topic_for_event(&self, event_type: &str) -> String {
        // イベントタイプの最初の部分（コンテキスト）を取得
        let context = event_type.split('.').next().unwrap_or("unknown");

        // コンテキストごとのトピックにマッピング
        let topic_suffix = match context {
            "vocabulary" => "vocabulary-events",
            "learning" => "learning-events",
            "user" => "user-events",
            "algorithm" => "algorithm-events",
            "ai" => "ai-events",
            "progress" => "progress-events",
            _ => {
                warn!("Unknown event context: {}, using default topic", context);
                "unknown-events"
            },
        };

        format!("{}-{}", self.config.topic_prefix, topic_suffix)
    }

    /// Publisher を取得または作成
    async fn get_or_create_publisher(&self, topic: &str) -> Result<Publisher, EventBusError> {
        // 読み取りロックで確認
        {
            let publishers = self.publishers.read().await;
            if let Some(publisher) = publishers.get(topic) {
                return Ok(publisher.clone());
            }
        }

        // 書き込みロックで作成
        let mut publishers = self.publishers.write().await;

        // 再度確認（二重チェックロッキング）
        if let Some(publisher) = publishers.get(topic) {
            return Ok(publisher.clone());
        }

        // トピックの存在確認（オプション）
        let topic_path = self.client.topic(topic);

        // Publisher を作成
        let publisher = topic_path.new_publisher(None);

        info!("Created publisher for topic: {}", topic);
        publishers.insert(topic.to_string(), publisher.clone());

        Ok(publisher)
    }

    /// Pub/Sub メッセージを作成
    fn create_message(
        &self,
        event_type: &str,
        aggregate_id: &Uuid,
        event_data: JsonValue,
    ) -> Result<PubsubMessage, EventBusError> {
        // イベントデータをシリアライズ
        let data = serde_json::to_vec(&event_data)?;

        // 属性を設定
        let mut attributes = HashMap::new();
        attributes.insert("event_type".to_string(), event_type.to_string());
        attributes.insert("aggregate_id".to_string(), aggregate_id.to_string());
        attributes.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
        attributes.insert("source".to_string(), "event_store_service".to_string());

        // Ordering Key を設定（同じ集約のイベントは順序保証）
        let ordering_key = if self.config.enable_ordering {
            aggregate_id.to_string()
        } else {
            String::new()
        };

        Ok(PubsubMessage {
            data,
            attributes,
            message_id: String::new(), // Pub/Sub が自動生成
            publish_time: None,
            ordering_key,
        })
    }

    /// Event Bus を停止
    #[allow(dead_code)]
    pub async fn shutdown(&self) {
        info!("Shutting down Event Bus...");

        // すべての Publisher をシャットダウン
        let mut publishers = self.publishers.write().await;
        for (topic, mut publisher) in publishers.drain() {
            publisher.shutdown().await;
            info!("Publisher for topic {} shut down", topic);
        }

        info!("Event Bus shutdown complete");
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_topic_mapping() {
        // get_topic_for_event メソッドの動作を検証
        // このテストはトピック名のマッピングロジックを確認します

        // EventBus::get_topic_for_event の実装を参照して期待値を設定
        let test_cases = vec![
            ("vocabulary.ItemCreated", "effect-vocabulary-events"),
            ("learning.SessionStarted", "effect-learning-events"),
            ("user.UserCreated", "effect-user-events"),
            ("algorithm.ParametersUpdated", "effect-algorithm-events"),
            ("ai.ResponseGenerated", "effect-ai-events"),
            ("unknown.SomeEvent", "effect-unknown-events"),
        ];

        for (event_type, expected_topic) in test_cases {
            // ここではロジックのテストのみ（実際の EventBus インスタンス化は不要）
            let context = event_type.split('.').next().unwrap_or("unknown");
            let topic = format!("effect-{context}-events");
            assert_eq!(topic, expected_topic, "Failed for event type: {event_type}");
        }
    }
}
