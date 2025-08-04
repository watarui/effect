//! Google Pub/Sub サブスクライバー実装
//!
//! イベントストリームをサブスクライブする

use std::sync::Arc;

use async_trait::async_trait;
use google_cloud_pubsub::client::{Client, ClientConfig};
use shared_error::{DomainError, DomainResult};
use tracing::{info, instrument};

use crate::ports::{inbound::EventHandler, outbound::EventSubscriber};

/// Pub/Sub サブスクライバー
pub struct PubSubSubscriber {
    client:            Client,
    subscription_name: String,
    event_handler:     Arc<dyn EventHandler>,
}

impl PubSubSubscriber {
    /// 新しいサブスクライバーを作成
    pub async fn new(
        subscription_name: String,
        event_handler: Arc<dyn EventHandler>,
    ) -> DomainResult<Self> {
        let client = Client::new(ClientConfig::default())
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to create Pub/Sub client: {e}")))?;

        Ok(Self {
            client,
            subscription_name,
            event_handler,
        })
    }
}

#[async_trait]
impl EventSubscriber for PubSubSubscriber {
    #[instrument(skip(self))]
    async fn subscribe(&self) -> DomainResult<()> {
        info!("Starting Pub/Sub subscription: {}", self.subscription_name);

        let _subscription = self.client.subscription(&self.subscription_name);
        let _cancel = tokio_util::sync::CancellationToken::new();

        // receive メソッドを使用してメッセージを処理
        let _handler = self.event_handler.clone();

        // TODO: 実際のサブスクリプション処理を実装
        // 現時点では google-cloud-pubsub の receive
        // メソッドのライフタイムの問題により、 完全な実装ができません。
        //
        // subscription
        //     .receive(
        //         move |mut message, _cancel| {
        //             let handler = handler.clone();
        //             async move {
        //                 let data = message.message.data.clone();
        //                 match handler.handle_event(data).await {
        //                     Ok(()) => {
        //                         // 成功時は ACK を送信
        //                         let _ = message.ack().await;
        //                     },
        //                     Err(e) => {
        //                         error!("Failed to handle event: {}", e);
        //                         // エラー時は ACK しない（自動的に再配信される）
        //                     },
        //                 }
        //             }
        //         },
        //         cancel.clone(),
        //         None,
        //     )
        //     .await
        //     .map_err(|e| DomainError::Internal(format!("Failed to receive messages:
        // {}", e)))?;

        // 一時的にループで待機
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        Ok(())
    }

    async fn unsubscribe(&self) -> DomainResult<()> {
        info!("Stopping Pub/Sub subscription: {}", self.subscription_name);
        // Graceful shutdown はループの外側で管理
        Ok(())
    }
}
