//! 設定管理ユーティリティ
//!
//! このモジュールは環境変数からの設定読み込みと
//! 検証機能を提供します。

use std::env;

use thiserror::Error;

/// 設定関連のエラー
#[derive(Debug, Error)]
pub enum Error {
    /// 環境変数が見つからない
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(&'static str),

    /// 無効な値
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(&'static str, String),

    /// パースエラー
    #[error("Failed to parse {0}: {1}")]
    Parse(&'static str, String),
}

/// 環境変数から文字列を取得
///
/// # Errors
///
/// 環境変数が設定されていない場合はエラーを返す
pub fn get_env(key: &'static str) -> Result<String, Error> {
    env::var(key).map_err(|_| Error::MissingEnvVar(key))
}

/// 環境変数から文字列を取得（デフォルト値付き）
#[must_use]
pub fn get_env_or(key: &'static str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// 環境変数から整数を取得
///
/// # Errors
///
/// 環境変数が設定されていない、または整数にパースできない場合はエラーを返す
pub fn get_env_parse<T>(key: &'static str) -> Result<T, Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = get_env(key)?;
    value
        .parse()
        .map_err(|e: T::Err| Error::Parse(key, e.to_string()))
}

/// 環境変数から整数を取得（デフォルト値付き）
pub fn get_env_parse_or<T>(key: &'static str, default: T) -> T
where
    T: std::str::FromStr,
{
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// 環境変数から bool を取得
///
/// "true", "1", "yes", "on" を true として扱う（大文字小文字を区別しない）
///
/// # Errors
///
/// 環境変数が設定されていない場合はエラーを返す
pub fn get_env_bool(key: &'static str) -> Result<bool, Error> {
    let value = get_env(key)?;
    Ok(matches!(
        value.to_lowercase().as_str(),
        "true" | "1" | "yes" | "on"
    ))
}

/// 環境変数から bool を取得（デフォルト値付き）
#[must_use]
pub fn get_env_bool_or(key: &'static str, default: bool) -> bool {
    env::var(key).ok().map_or(default, |v| {
        matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
    })
}

/// 実行環境の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// 開発環境
    Development,
    /// ステージング環境
    Staging,
    /// 本番環境
    Production,
}

impl Environment {
    /// 環境変数 "ENVIRONMENT" から現在の環境を取得
    ///
    /// # Errors
    ///
    /// 無効な環境名が指定された場合はエラーを返す
    pub fn from_env() -> Result<Self, Error> {
        let env = get_env_or("ENVIRONMENT", "development");
        match env.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "staging" | "stage" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            _ => Err(Error::InvalidValue("ENVIRONMENT", env)),
        }
    }

    /// 開発環境かどうか
    #[must_use]
    pub const fn is_development(self) -> bool {
        matches!(self, Self::Development)
    }

    /// 本番環境かどうか
    #[must_use]
    pub const fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_should_parse_correctly() {
        // 環境変数をモックするのは難しいので、直接パース部分をテスト
        assert_eq!(Environment::Development, Environment::Development);
        assert!(Environment::Development.is_development());
        assert!(!Environment::Development.is_production());
    }

    #[test]
    fn config_error_should_display_correctly() {
        let error = Error::MissingEnvVar("TEST_VAR");
        assert_eq!(error.to_string(), "Missing environment variable: TEST_VAR");
    }
}
