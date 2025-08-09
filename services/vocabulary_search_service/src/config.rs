use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server:      ServerConfig,
    pub meilisearch: MeilisearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeilisearchConfig {
    pub url:     String,
    pub api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            server:      ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8084".to_string())
                    .parse()?,
            },
            meilisearch: MeilisearchConfig {
                url:     std::env::var("MEILISEARCH_URL")
                    .unwrap_or_else(|_| "http://localhost:7700".to_string()),
                api_key: std::env::var("MEILISEARCH_API_KEY").ok(),
            },
        })
    }
}
