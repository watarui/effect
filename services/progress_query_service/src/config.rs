use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server:   ServerConfig,
    pub database: DatabaseConfig,
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

impl Config {
    pub fn from_env() -> crate::error::Result<Self> {
        Ok(Config {
            server:   ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8091".to_string())
                    .parse()?,
            },
            database: DatabaseConfig {
                url:             std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://effect:effect_password@localhost:5436/progress_read_model"
                        .to_string()
                }),
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
        })
    }
}
