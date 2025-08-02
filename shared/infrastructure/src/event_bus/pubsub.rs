//! Google Pub/Sub による [`EventBus`] 実装
//!
//! このモジュールは [`EventBus`] トレイトの Google Pub/Sub
//! ベースの実装を提供します。 ドメインイベントの発行と購読機能を実现します。

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use domain_events::{DomainEvent, EventBus, EventError, EventHandler};
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::{client::Client, publisher::Publisher};
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

    /// ドメインイベント用のトピック名を取得
    fn get_topic_name(event: &DomainEvent) -> String {
        format!("effect-{}", event.event_type().to_lowercase())
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
    /// ドメインイベントを適切なトピックに発行
    async fn publish(&self, event: DomainEvent) -> Result<(), EventError> {
        let topic_name = Self::get_topic_name(&event);

        // イベントを JSON にシリアライズ
        let event_data = serde_json::to_vec(&event)
            .map_err(|e| EventError::Deserialization(format!("Failed to serialize event: {e}")))?;

        // タイムスタンプを取得（メタデータがない場合は現在時刻を使用）
        let timestamp = event
            .metadata()
            .and_then(|meta| meta.occurred_at.as_ref())
            .and_then(|ts| {
                use chrono::{DateTime, Utc};
                let nanos = u32::try_from(ts.nanos).ok()?;
                DateTime::<Utc>::from_timestamp(ts.seconds, nanos)
            })
            .map_or_else(|| chrono::Utc::now().to_rfc3339(), |dt| dt.to_rfc3339());

        // Pub/Sub メッセージを作成
        let message = PubsubMessage {
            data: event_data,
            attributes: HashMap::from([
                ("event_type".to_string(), event.event_type().to_string()),
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

        info!(
            "Published event {} to topic {}",
            event.event_type(),
            topic_name
        );
        Ok(())
    }

    /// 指定されたハンドラーでイベントを購読
    async fn subscribe(&self, handler: Box<dyn EventHandler>) -> Result<(), EventError> {
        // 現時点では、すべてのコンテキストをリッスンするシンプルな購読を実装
        // 実際の実装では、コンテキストやイベントタイプでフィルタリングする機能を
        // 追加することを検討

        let subscription_name = format!("effect-all-events-{}", uuid::Uuid::new_v4());
        let topic_name = "effect-all"; // すべてのイベントを受信する特別なトピック

        // サブスクリプションの存在確認と作成
        self.ensure_subscription_exists(&subscription_name, topic_name)
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
                    // イベントをデシリアライズ
                    match serde_json::from_slice::<DomainEvent>(&msg.message.data) {
                        Ok(event) => {
                            // イベントを処理
                            if let Err(e) = handler.handle(event).await {
                                error!("Error handling event: {}", e);
                            }

                            // メッセージを確認応答
                            let _ = msg.ack().await;
                        },
                        Err(e) => {
                            error!("Error deserializing event: {}", e);
                            // リトライ可能にするためメッセージを否定応答
                            let _ = msg.nack().await;
                        },
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
