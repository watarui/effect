//! データベース接続管理
//!
//! このモジュールは `PostgreSQL` データベースへの接続プールと
//! 関連するユーティリティを提供します。

use std::time::Duration;

use sqlx::{
    ConnectOptions,
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use thiserror::Error;
use tracing::log;

/// データベース関連のエラー
#[derive(Debug, Error)]
pub enum Error {
    /// 接続エラー
    #[error("Database connection error: {0}")]
    Connection(String),

    /// 設定エラー
    #[error("Database configuration error: {0}")]
    Configuration(String),

    /// SQL 実行エラー
    #[error("SQL execution error: {0}")]
    Execution(#[from] sqlx::Error),
}

/// データベース接続設定
#[derive(Debug, Clone)]
pub struct Config {
    /// データベース URL
    pub url:                  String,
    /// 最大接続数
    pub max_connections:      u32,
    /// 最小接続数
    pub min_connections:      u32,
    /// 接続タイムアウト（秒）
    pub connect_timeout_secs: u64,
    /// アイドルタイムアウト（秒）
    pub idle_timeout_secs:    u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url:                  String::new(),
            max_connections:      10,
            min_connections:      1,
            connect_timeout_secs: 30,
            idle_timeout_secs:    10 * 60, // 10分
        }
    }
}

/// `PostgreSQL` 接続プールの作成
///
/// # Errors
///
/// データベースへの接続に失敗した場合はエラーを返す
pub async fn create_pool(config: &Config) -> Result<PgPool, Error> {
    // 接続オプションの設定
    let connect_options = config
        .url
        .parse::<PgConnectOptions>()
        .map_err(|e| Error::Configuration(format!("Invalid database URL: {e}")))?
        .log_statements(log::LevelFilter::Debug)
        .log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(1));

    // プールオプションの設定
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .connect_with(connect_options)
        .await
        .map_err(|e| Error::Connection(format!("Failed to create connection pool: {e}")))?;

    Ok(pool)
}

/// データベースのヘルスチェック
///
/// # Errors
///
/// データベースへの接続またはクエリ実行に失敗した場合はエラーを返す
pub async fn health_check(pool: &PgPool) -> Result<(), Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(Error::Execution)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_config_should_have_sensible_defaults() {
        let config = Config::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
        assert_eq!(config.connect_timeout_secs, 30);
        assert_eq!(config.idle_timeout_secs, 600);
    }

    #[test]
    fn database_error_should_display_correctly() {
        let error = Error::Connection("test error".to_string());
        assert_eq!(error.to_string(), "Database connection error: test error");
    }

    // 実際のデータベース接続テストは integration tests で実施
}
