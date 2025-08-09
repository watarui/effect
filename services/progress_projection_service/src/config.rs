use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub event_store: DatabaseConfig,
    pub read_model:  DatabaseConfig,
    pub pubsub:      PubSubConfig,
    pub processor:   ProcessorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url:             String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubConfig {
    pub project_id:   String,
    pub subscription: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    pub batch_size:       usize,
    pub poll_interval_ms: u64,
}

impl Config {
    pub fn from_env() -> crate::error::Result<Self> {
        Ok(Config {
            event_store: DatabaseConfig {
                url:             std::env::var("EVENT_STORE_URL").unwrap_or_else(|_| {
                    "postgres://effect:effect_password@localhost:5436/progress_event_store"
                        .to_string()
                }),
                max_connections: 5,
            },
            read_model:  DatabaseConfig {
                url:             std::env::var("READ_MODEL_URL").unwrap_or_else(|_| {
                    "postgres://effect:effect_password@localhost:5436/progress_read_model"
                        .to_string()
                }),
                max_connections: 10,
            },
            pubsub:      PubSubConfig {
                project_id:   std::env::var("GCP_PROJECT_ID")
                    .unwrap_or_else(|_| "effect-project".to_string()),
                subscription: std::env::var("PUBSUB_SUBSCRIPTION")
                    .unwrap_or_else(|_| "progress-projection".to_string()),
            },
            processor:   ProcessorConfig {
                batch_size:       std::env::var("BATCH_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                poll_interval_ms: std::env::var("POLL_INTERVAL_MS")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()?,
            },
        })
    }
}
