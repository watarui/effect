//! Event Store Service の設定

use serde::{Deserialize, Serialize};

/// サービス設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// gRPC サーバーのポート
    pub port: u16,

    /// データベース URL
    pub database_url: String,

    /// スナップショット設定
    pub snapshot: SnapshotConfig,

    /// Event Bus (Pub/Sub) 設定
    pub event_bus: EventBusConfig,

    /// Domain Events Service 設定
    pub domain_events: DomainEventsConfig,
}

/// Event Bus 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusConfig {
    /// Google Cloud プロジェクト ID
    pub project_id: String,

    /// トピックのプレフィックス
    pub topic_prefix: String,

    /// 順序保証を有効にするか
    pub enable_ordering: bool,

    /// バッチ発行の最大サイズ
    pub max_batch_size: usize,
}

/// Domain Events Service 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEventsConfig {
    /// Domain Events Service の URL
    pub url: String,

    /// 検証を有効にするか
    pub enable_validation: bool,
}

/// スナップショット設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// スナップショットを作成するイベント数の閾値
    pub threshold: u32,

    /// スナップショットの保持期間（日）
    pub retention_days: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port:          50051,
            database_url:  "postgres://effect:effect_password@localhost:5432/event_store_db"
                .to_string(),
            snapshot:      SnapshotConfig {
                threshold:      100,
                retention_days: 30,
            },
            event_bus:     EventBusConfig {
                project_id:      "effect-project".to_string(),
                topic_prefix:    "effect".to_string(),
                enable_ordering: true,
                max_batch_size:  100,
            },
            domain_events: DomainEventsConfig {
                url:               "http://localhost:50053".to_string(),
                enable_validation: true,
            },
        }
    }
}

/// 設定を読み込む
pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    // 環境変数から読み込み
    let config = Config {
        port:          std::env::var("PORT")
            .unwrap_or_else(|_| "50051".to_string())
            .parse()?,
        database_url:  std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| Config::default().database_url),
        snapshot:      SnapshotConfig {
            threshold:      std::env::var("SNAPSHOT_THRESHOLD")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            retention_days: std::env::var("SNAPSHOT_RETENTION_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
        },
        event_bus:     EventBusConfig {
            project_id:      std::env::var("GCP_PROJECT_ID")
                .unwrap_or_else(|_| Config::default().event_bus.project_id),
            topic_prefix:    std::env::var("PUBSUB_TOPIC_PREFIX")
                .unwrap_or_else(|_| Config::default().event_bus.topic_prefix),
            enable_ordering: std::env::var("PUBSUB_ENABLE_ORDERING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            max_batch_size:  std::env::var("PUBSUB_MAX_BATCH_SIZE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
        },
        domain_events: DomainEventsConfig {
            url:               std::env::var("DOMAIN_EVENTS_URL")
                .unwrap_or_else(|_| Config::default().domain_events.url),
            enable_validation: std::env::var("ENABLE_EVENT_VALIDATION")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        },
    };

    Ok(config)
}
