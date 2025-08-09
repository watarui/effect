use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub pubsub:   PubSubConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubConfig {
    pub project_id:   String,
    pub subscription: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://effect:effect_password@localhost:5434/vocabulary_query_db"
                        .to_string()
                }),
            },
            pubsub:   PubSubConfig {
                project_id:   std::env::var("GCP_PROJECT_ID")
                    .unwrap_or_else(|_| "effect-project".to_string()),
                subscription: std::env::var("PUBSUB_SUBSCRIPTION")
                    .unwrap_or_else(|_| "vocabulary-projection-sub".to_string()),
            },
        })
    }
}
