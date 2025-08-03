//! Google Pub/Sub サブスクライバー実装

use google_cloud_pubsub::client::{Client, ClientConfig};
use shared_error::{DomainError, DomainResult};
use tracing::{info, instrument};

/// Pub/Sub サブスクライバー
pub struct PubSubSubscriber {
    client:            Client,
    subscription_name: String,
}

impl PubSubSubscriber {
    /// 新しいサブスクライバーを作成
    pub async fn new(_project_id: String, subscription_name: String) -> DomainResult<Self> {
        // TODO: プロジェクトIDを環境変数から取得
        let client = Client::new(ClientConfig::default())
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to create Pub/Sub client: {e}")))?;

        Ok(Self {
            client,
            subscription_name,
        })
    }

    /// サブスクリプションを開始
    #[instrument(skip(self))]
    pub async fn start_subscription(&self) -> DomainResult<()> {
        info!("Starting Pub/Sub subscription: {}", self.subscription_name);

        let _subscription = self.client.subscription(&self.subscription_name);

        // TODO: 実装
        // 1. メッセージの受信ループを開始
        // 2. 受信したメッセージをデシリアライズ
        // 3. 適切なイベントハンドラーにディスパッチ
        // 4. 処理成功時にメッセージを ACK

        Ok(())
    }

    /// メッセージを処理
    #[allow(dead_code)]
    async fn process_message(&self, _message: Vec<u8>) -> DomainResult<()> {
        // TODO: 実装
        // 1. メッセージをデシリアライズ
        // 2. イベントタイプを判定
        // 3. 適切なハンドラーを呼び出し

        Ok(())
    }
}
