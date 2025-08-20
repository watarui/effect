use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database:  DatabasesConfig,
    pub processor: ProcessorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasesConfig {
    pub event_store_url: String,
    pub read_model_url:  String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    pub batch_size:       usize,
    pub poll_interval_ms: u64,
}

impl Config {
    pub fn from_env() -> crate::error::Result<Self> {
        Ok(Config {
            database:  DatabasesConfig {
                event_store_url: std::env::var("EVENT_STORE_URL").unwrap_or_else(|_| {
                    "postgres://effect:effect_password@localhost:5436/progress_event_store"
                        .to_string()
                }),
                read_model_url:  std::env::var("READ_MODEL_URL").unwrap_or_else(|_| {
                    "postgres://effect:effect_password@localhost:5436/progress_read_model"
                        .to_string()
                }),
            },
            processor: ProcessorConfig {
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
