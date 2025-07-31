//! AI Service の設定モジュール

use std::env;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 設定エラー
#[derive(Debug, Error)]
pub enum ConfigError {
    /// 環境変数が見つからない
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(&'static str),

    /// ポート番号が不正
    #[error("Invalid port number: {0}")]
    InvalidPort(#[from] std::num::ParseIntError),

    /// 不明な環境
    #[error("Unknown environment: {0}")]
    UnknownEnvironment(String),
}

/// サービスの動作環境
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// 開発環境
    Development,
    /// ステージング環境
    Staging,
    /// 本番環境
    Production,
}

/// AI Service の設定
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServiceConfig {
    /// サービスのポート番号
    pub port:         u16,
    /// データベース接続URL
    pub database_url: String,
    /// `Redis接続URL`
    pub redis_url:    String,
    /// 動作環境
    pub environment:  Environment,
}

impl ServiceConfig {
    /// 環境変数から設定を読み込む
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .parse::<Environment>()
            .map_err(ConfigError::UnknownEnvironment)?;

        let config = match environment {
            Environment::Development => Self {
                port: env::var("PORT")
                    .unwrap_or_else(|_| "50056".to_string())
                    .parse()?,
                database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://postgres:password@localhost/ai_dev".to_string()
                }),
                redis_url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379/5".to_string()),
                environment,
            },
            Environment::Production => Self {
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
                database_url: env::var("DATABASE_URL")
                    .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?,
                redis_url: env::var("REDIS_URL")
                    .map_err(|_| ConfigError::MissingEnvVar("REDIS_URL"))?,
                environment,
            },
            Environment::Staging => Self {
                port: env::var("PORT")
                    .unwrap_or_else(|_| "50056".to_string())
                    .parse()?,
                database_url: env::var("DATABASE_URL")
                    .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?,
                redis_url: env::var("REDIS_URL")
                    .map_err(|_| ConfigError::MissingEnvVar("REDIS_URL"))?,
                environment,
            },
        };

        Ok(config)
    }
}

impl std::str::FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "staging" | "stage" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            _ => Err(format!("Unknown environment: {s}")),
        }
    }
}
