use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server:      ServerConfig,
    pub database:    DatabaseConfig,
    pub event_store: EventStoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url:             String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStoreConfig {
    pub url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            server:      ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "50052".to_string())
                    .parse()
                    .map_err(|e| Error::Config(format!("Invalid port: {}", e)))?,
            },
            database:    DatabaseConfig {
                url:             std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://effect:effect_password@localhost:5434/vocabulary_command_db"
                        .to_string()
                }),
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .map_err(|e| Error::Config(format!("Invalid max_connections: {}", e)))?,
            },
            event_store: EventStoreConfig {
                url: std::env::var("EVENT_STORE_URL").unwrap_or_else(|_| {
                    "postgres://effect:effect_password@localhost:5432/event_store_db".to_string()
                }),
            },
        })
    }
}
