//! Google Pub/Sub による [`EventBus`] 実装
//!
//! このモジュールは [`EventBus`] トレイトの Google Pub/Sub
//! ベースの実装を提供します。 ドメインイベントの発行と購読機能を実现します。

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::{client::Client, publisher::Publisher};
use shared_kernel::{EventBus, EventError};
use tokio::sync::RwLock;
use tracing::{error, info};

/// Google Pub/Sub ベースのイベントバス実装
pub struct PubSubEventBus {
    client:     Client,
    project_id: String,
    publishers: Arc<RwLock<HashMap<String, Publisher>>>,
}

impl PubSubEventBus {
    /// 新しい [`PubSubEventBus`] インスタンスを作成
    ///
    /// # Arguments
    ///
    /// * `project_id` - Google Cloud プロジェクト ID
    ///
    /// # Errors
    ///
    /// Pub/Sub クライアントの作成に失敗した場合はエラーを返す
    pub async fn new(project_id: String) -> Result<Self, EventError> {
        let client = Client::new(google_cloud_pubsub::client::ClientConfig {
            project_id: Some(project_id.clone()),
            ..Default::default()
        })
        .await
        .map_err(|e| EventError::Publish(format!("Failed to create Pub/Sub client: {e}")))?;

        Ok(Self {
            client,
            project_id,
            publishers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 指定されたトピック用のパブリッシャーを取得または作成
    async fn get_or_create_publisher(&self, topic_name: &str) -> Result<Publisher, EventError> {
        let mut publishers = self.publishers.write().await;

        if let Some(publisher) = publishers.get(topic_name) {
            return Ok(publisher.clone());
        }

        let topic = self
            .client
            .topic(&format!("{}-{}", self.project_id, topic_name));

        // トピックが存在しない場合は作成
        if !topic
            .exists(None)
            .await
            .map_err(|e| EventError::Publish(format!("Failed to check topic existence: {e}")))?
        {
            topic
                .create(None, None)
                .await
                .map_err(|e| EventError::Publish(format!("Failed to create topic: {e}")))?;
            info!("Created topic: {}", topic_name);
        }

        let publisher = topic.new_publisher(None);
        publishers.insert(topic_name.to_string(), publisher.clone());

        Ok(publisher)
    }

    /// トピック名からイベントタイプを取得
    fn get_topic_name(topic: &str) -> String {
        format!("effect-{topic}")
    }

    /// サブスクリプションの存在確認と作成
    async fn ensure_subscription_exists(
        &self,
        subscription_name: &str,
        topic_name: &str,
    ) -> Result<(), EventError> {
        let full_topic_name = format!("{}-{}", self.project_id, topic_name);
        let topic = self.client.topic(&full_topic_name);

        // トピックが存在しない場合は作成
        if !topic
            .exists(None)
            .await
            .map_err(|e| EventError::Handler(format!("Failed to check topic existence: {e}")))?
        {
            topic
                .create(None, None)
                .await
                .map_err(|e| EventError::Handler(format!("Failed to create topic: {e}")))?;
            info!("Created topic: {}", topic_name);
        }

        // サブスクリプションを作成
        let subscription = self.client.subscription(subscription_name);
        if !subscription.exists(None).await.map_err(|e| {
            EventError::Handler(format!("Failed to check subscription existence: {e}"))
        })? {
            subscription
                .create(
                    topic.fully_qualified_name(),
                    google_cloud_pubsub::subscription::SubscriptionConfig::default(),
                    None,
                )
                .await
                .map_err(|e| EventError::Handler(format!("Failed to create subscription: {e}")))?;
            info!("Created subscription: {}", subscription_name);
        }
        drop(topic);
        drop(subscription);

        Ok(())
    }
}

#[async_trait]
impl EventBus for PubSubEventBus {
    /// イベントを適切なトピックに発行
    async fn publish(&self, topic: &str, event: &[u8]) -> Result<(), EventError> {
        let topic_name = Self::get_topic_name(topic);

        // タイムスタンプを取得
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Pub/Sub メッセージを作成
        let message = PubsubMessage {
            data: event.to_vec(),
            attributes: HashMap::from([
                ("topic".to_string(), topic.to_string()),
                ("timestamp".to_string(), timestamp),
            ]),
            ..Default::default()
        };

        // メッセージを発行
        let awaiter = self
            .get_or_create_publisher(&topic_name)
            .await?
            .publish(message)
            .await;
        awaiter
            .get()
            .await
            .map_err(|e| EventError::Publish(format!("Failed to publish message: {e}")))?;

        info!("Published event to topic {}", topic_name);
        Ok(())
    }

    /// 指定されたハンドラーでイベントを購読
    async fn subscribe<F>(&self, topic: &str, handler: F) -> Result<(), EventError>
    where
        F: Fn(&[u8]) -> Result<(), EventError> + Send + Sync + 'static,
    {
        let subscription_name = format!("effect-{}-{}", topic, uuid::Uuid::new_v4());
        let topic_name = Self::get_topic_name(topic);

        // サブスクリプションの存在確認と作成
        self.ensure_subscription_exists(&subscription_name, &topic_name)
            .await?;

        // spawn に必要な情報をクローン
        let client = self.client.clone();
        let handler = Arc::new(handler);
        let subscription_name_clone = subscription_name.clone();

        // メッセージの受信を開始
        tokio::spawn(async move {
            // タスク内で subscription を新規作成
            let subscription = client.subscription(&subscription_name_clone);

            loop {
                let stream = match subscription.pull(100, None).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        error!("Error pulling messages: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    },
                };

                for msg in stream {
                    // イベントを処理
                    if let Err(e) = handler(&msg.message.data) {
                        error!("Error handling event: {}", e);
                        // リトライ可能にするためメッセージを否定応答
                        let _ = msg.nack().await;
                    } else {
                        // メッセージを確認応答
                        let _ = msg.ack().await;
                    }
                }
            }
        });

        info!("Started subscription: {}", subscription_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // テストはモックまたはテスト用 Pub/Sub インスタンスが必要
}
