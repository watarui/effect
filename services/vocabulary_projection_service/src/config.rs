use serde::{Deserialize, Serialize};

/// Vocabulary Projection Service の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database:    DatabaseConfig,
    pub event_store: EventStoreConfig,
    pub projection:  ProjectionConfig,
}

/// データベース設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url:             String,
    pub max_connections: u32,
}

/// Event Store 接続設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStoreConfig {
    pub url:                 String,
    pub batch_size:          usize,
    pub polling_interval_ms: u64,
}

/// プロジェクション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionConfig {
    pub name:                String,
    pub checkpoint_interval: usize,
    pub error_retry_limit:   u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            database:    DatabaseConfig {
                url:             std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://effect:effect_password@localhost:5434/vocabulary_db".to_string()
                }),
                max_connections: std::env::var("DB_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
            },
            event_store: EventStoreConfig {
                url:                 std::env::var("EVENT_STORE_URL")
                    .unwrap_or_else(|_| "http://localhost:50051".to_string()),
                batch_size:          std::env::var("EVENT_BATCH_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100),
                polling_interval_ms: std::env::var("POLLING_INTERVAL_MS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1000),
            },
            projection:  ProjectionConfig {
                name:                "vocabulary_projection".to_string(),
                checkpoint_interval: 100,
                error_retry_limit:   3,
            },
        })
    }
}
