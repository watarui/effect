//! サービス設定

use std::env;

use serde::Deserialize;
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

/// Learning Service の設定
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ServiceConfig {
    /// サービスのポート番号
    pub port: u16,

    /// データベースURL
    pub database_url: String,

    /// Redis URL
    pub redis_url: String,

    /// 環境（local/production）
    pub environment: Environment,
}

/// 実行環境
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// ローカル開発環境
    Local,
    /// 本番環境（Cloud Run）
    Production,
}

impl ServiceConfig {
    /// 環境変数から設定を読み込む
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            port:         env::var("PORT")
                .unwrap_or_else(|_| "50051".to_string())
                .parse()?,
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?,
            redis_url:    env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            environment:  env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "local".to_string())
                .parse()
                .map_err(ConfigError::UnknownEnvironment)?,
        })
    }

    /// データベーススキーマ名を取得
    #[allow(dead_code)]
    pub const fn database_schema(&self) -> &str {
        match self.environment {
            Environment::Local => "public",
            Environment::Production => "learning_schema",
        }
    }
}

impl std::str::FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            _ => Err(format!("Unknown environment: {s}")),
        }
    }
}
