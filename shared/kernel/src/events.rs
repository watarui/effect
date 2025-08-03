//! イベント関連の共通型定義
//!
//! すべての Bounded Context で共有されるイベントの基本構造を定義します。
//! ビジネスロジックは含まず、メタデータと基本的なトレイトのみを提供します。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ids::UserId;

/// イベントメタデータ
///
/// すべてのドメインイベントが持つべき共通のメタデータ情報
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventMetadata {
    /// イベントID（一意）
    pub event_id: String,

    /// 集約ID
    pub aggregate_id: String,

    /// イベント発生時刻
    pub occurred_at: DateTime<Utc>,

    /// イベントバージョン（楽観的ロック用）
    pub version: u64,

    /// イベントを引き起こしたユーザーID
    pub caused_by_user_id: Option<UserId>,

    /// 相関ID（複数のイベントを関連付ける）
    pub correlation_id: Option<String>,

    /// 因果関係ID（このイベントを引き起こしたイベントのID）
    pub causation_id: Option<String>,

    /// トレースコンテキスト（分散トレーシング用）
    pub trace_context: Option<TraceContext>,

    /// コマンドID（このイベントを生成したコマンドのID）
    pub command_id: Option<String>,

    /// ソースコンテキスト（どの Bounded Context から発生したか）
    pub source_context: Option<String>,

    /// スキーマバージョン（イベントの構造バージョン）
    pub schema_version: Option<u32>,
}

impl EventMetadata {
    /// 新しいイベントメタデータを作成
    pub fn new(aggregate_id: impl Into<String>) -> Self {
        Self {
            event_id:          Uuid::new_v4().to_string(),
            aggregate_id:      aggregate_id.into(),
            occurred_at:       Utc::now(),
            version:           1,
            caused_by_user_id: None,
            correlation_id:    None,
            causation_id:      None,
            trace_context:     None,
            command_id:        None,
            source_context:    None,
            schema_version:    Some(1),
        }
    }

    /// ユーザーIDを設定
    pub fn with_user(mut self, user_id: UserId) -> Self {
        self.caused_by_user_id = Some(user_id);
        self
    }

    /// 相関IDを設定
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// 因果関係IDを設定
    pub fn with_causation_id(mut self, causation_id: impl Into<String>) -> Self {
        self.causation_id = Some(causation_id.into());
        self
    }

    /// ソースコンテキストを設定
    pub fn with_source_context(mut self, source_context: impl Into<String>) -> Self {
        self.source_context = Some(source_context.into());
        self
    }
}

/// 分散トレーシング用のコンテキスト
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id:       String,
    pub span_id:        String,
    pub parent_span_id: Option<String>,
}

/// ドメインイベントの基本トレイト
///
/// すべてのドメインイベントが実装すべきインターフェース
pub trait DomainEvent: Send + Sync {
    /// イベントタイプを取得
    fn event_type(&self) -> &str;

    /// メタデータを取得
    fn metadata(&self) -> &EventMetadata;

    /// 集約IDを取得
    fn aggregate_id(&self) -> &str {
        &self.metadata().aggregate_id
    }
}

/// 統合イベントの基本トレイト
///
/// Bounded Context 間で共有される統合イベント用のインターフェース
pub trait IntegrationEvent: Send + Sync {
    /// イベントタイプを取得
    fn event_type(&self) -> &str;

    /// ソースコンテキストを取得
    fn source_context(&self) -> &str;

    /// イベントIDを取得
    fn event_id(&self) -> &str;

    /// 発生時刻を取得
    fn occurred_at(&self) -> DateTime<Utc>;
}

/// イベントエラー型
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Event store error: {0}")]
    Store(String),

    #[error("Event bus error: {0}")]
    Bus(String),

    #[error("Invalid event: {0}")]
    InvalidEvent(String),
}

/// イベントハンドラーのトレイト
///
/// 特定のイベントタイプを処理するハンドラー
#[async_trait::async_trait]
pub trait EventHandler<E>: Send + Sync
where
    E: Send + Sync,
{
    /// イベントを処理
    async fn handle(&self, event: E) -> Result<(), EventError>;
}

/// イベントバスのトレイト
///
/// イベントの発行と購読を管理
#[async_trait::async_trait]
pub trait EventBus: Send + Sync {
    /// イベントを発行
    async fn publish(&self, topic: &str, event: &[u8]) -> Result<(), EventError>;

    /// イベントを購読
    async fn subscribe<F>(&self, topic: &str, handler: F) -> Result<(), EventError>
    where
        F: Fn(&[u8]) -> Result<(), EventError> + Send + Sync + 'static;
}

/// イベントストアのトレイト
///
/// イベントの永続化と取得を管理
#[async_trait::async_trait]
pub trait EventStore: Send + Sync {
    /// イベントを保存
    async fn append(&self, stream_id: &str, events: &[&[u8]]) -> Result<(), EventError>;

    /// イベントを取得
    async fn read(&self, stream_id: &str, from_version: u64) -> Result<Vec<Vec<u8>>, EventError>;

    /// スナップショットを保存
    async fn save_snapshot(
        &self,
        stream_id: &str,
        version: u64,
        data: &[u8],
    ) -> Result<(), EventError>;

    /// スナップショットを取得
    async fn load_snapshot(&self, stream_id: &str) -> Result<Option<(u64, Vec<u8>)>, EventError>;
}

/// イベントのシリアライゼーションヘルパー
pub mod serde_helpers {
    use prost_types::Timestamp;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    /// prost Timestamp と DateTime<Utc> の変換
    pub mod timestamp {
        use super::*;

        pub fn serialize<S>(date: &Option<Timestamp>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match date {
                Some(ts) => {
                    let dt = DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                        .ok_or_else(|| serde::ser::Error::custom("Invalid timestamp"))?;
                    dt.serialize(serializer)
                },
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Timestamp>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let opt: Option<DateTime<Utc>> = Option::deserialize(deserializer)?;
            Ok(opt.map(|dt| Timestamp {
                seconds: dt.timestamp(),
                nanos:   dt.timestamp_subsec_nanos() as i32,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_metadata_creation() {
        let metadata = EventMetadata::new("test-aggregate-123");

        assert_eq!(metadata.aggregate_id, "test-aggregate-123");
        assert_eq!(metadata.version, 1);
        assert!(metadata.caused_by_user_id.is_none());
        assert!(metadata.correlation_id.is_none());
        assert_eq!(metadata.schema_version, Some(1));
    }

    #[test]
    fn test_event_metadata_builder() {
        let user_id = UserId::new();
        let metadata = EventMetadata::new("test-aggregate")
            .with_user(user_id)
            .with_correlation_id("correlation-123")
            .with_causation_id("cause-456")
            .with_source_context("test-context");

        assert_eq!(metadata.caused_by_user_id, Some(user_id));
        assert_eq!(metadata.correlation_id, Some("correlation-123".to_string()));
        assert_eq!(metadata.causation_id, Some("cause-456".to_string()));
        assert_eq!(metadata.source_context, Some("test-context".to_string()));
    }
}
