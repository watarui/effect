//! サービス設定

use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::{Validate, ValidationError, ValidationErrors};

/// 設定エラー
#[derive(Error, Debug)]
pub enum ConfigError {
    /// 設定の読み込みエラー
    #[error("Configuration loading failed: {0}")]
    Load(#[from] config::ConfigError),
    /// バリデーションエラー
    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationErrors),
    /// 環境変数エラー
    #[error("Environment variable error: {0}")]
    Env(String),
}

/// ユーザーサービスの設定
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct Config {
    /// サーバー設定
    #[validate(nested)]
    pub server:   Server,
    /// データベース設定
    #[validate(nested)]
    pub database: Database,
    /// 認証設定
    pub auth:     Auth,
    /// イベントバス設定
    pub event:    Event,
}

/// ポート番号のバリデーション
fn validate_port(port: u16) -> Result<(), ValidationError> {
    if port < 1024 {
        return Err(ValidationError::new("invalid_port_range"));
    }
    Ok(())
}

/// データベース URL のバリデーション
fn validate_database_url(url: &str) -> Result<(), ValidationError> {
    if !url.starts_with("postgres://") && !url.starts_with("postgresql://") {
        return Err(ValidationError::new("invalid_database_url"));
    }
    Ok(())
}

/// サーバー設定
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Server {
    /// ホスト（デフォルト: 0.0.0.0）
    #[validate(length(min = 1, message = "Host cannot be empty"))]
    pub host: String,
    /// ポート（デフォルト: 50051）
    #[validate(custom(function = "validate_port"))]
    pub port: u16,
}

/// データベース設定
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Database {
    /// データベース URL
    #[validate(custom(function = "validate_database_url"))]
    pub url:             String,
    /// 最大接続数
    #[validate(range(
        min = 1,
        max = 100,
        message = "Max connections must be between 1 and 100"
    ))]
    pub max_connections: u32,
}

/// 認証設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Auth {
    /// Mock 認証（開発環境用）
    #[serde(rename = "mock")]
    Mock {
        /// 事前定義されたトークン（オプション）
        predefined_tokens: Option<Vec<MockToken>>,
    },
    /// Firebase 認証
    #[serde(rename = "firebase")]
    Firebase {
        /// プロジェクト ID
        project_id:               String,
        /// サービスアカウントキーのパス
        service_account_key_path: Option<String>,
    },
}

/// Mock トークン設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockToken {
    /// トークン
    pub token:   String,
    /// ユーザー ID
    pub user_id: String,
}

/// イベントバス設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// インメモリイベントバス（開発環境用）
    #[serde(rename = "memory")]
    Memory,
    /// Google Pub/Sub
    #[serde(rename = "pubsub")]
    PubSub {
        /// プロジェクト ID
        project_id: String,
        /// トピック名
        topic_name: String,
    },
}

impl Default for Server {
    fn default() -> Self {
        Self {
            host: String::from("0.0.0.0"),
            port: 50051,
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self {
            url:             String::from("postgres://localhost/effect_user"),
            max_connections: 10,
        }
    }
}

impl Default for Auth {
    fn default() -> Self {
        Self::Mock {
            predefined_tokens: None,
        }
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::Memory
    }
}

impl Config {
    /// 設定を読み込む（環境変数と設定ファイルから）
    ///
    /// # Errors
    ///
    /// * 設定の読み込みまたはバリデーションに失敗した場合
    pub fn load() -> Result<Self, ConfigError> {
        // 設定を構築
        let config = config::Config::builder()
            // デフォルト値を設定
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 50051)?
            .set_default("database.url", "postgres://localhost/effect_user")?
            .set_default("database.max_connections", 10)?
            .set_default("auth.type", "mock")?
            .set_default("event.type", "memory")?
            // 設定ファイルから読み込み（オプション）
            .add_source(
                config::File::with_name("config/user-service")
                    .required(false)
                    .format(config::FileFormat::Yaml),
            )
            .add_source(
                config::File::with_name("config/user-service.json")
                    .required(false)
                    .format(config::FileFormat::Json),
            )
            // 環境変数から読み込み（USER_SERVICE_ プレフィックス）
            .add_source(
                config::Environment::with_prefix("USER_SERVICE")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        // デシリアライズしてバリデーション
        let config: Self = config.try_deserialize()?;
        config.validate()?;

        Ok(config)
    }

    /// 環境変数から設定を読み込む（レガシーメソッド、`Config::load()`
    /// の使用を推奨）
    ///
    /// # Errors
    ///
    /// * 環境変数の読み込みに失敗した場合
    #[deprecated(since = "0.1.0", note = "Use Config::load() instead")]
    pub fn from_env() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            // デフォルト値
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 50051)?
            .set_default("database.max_connections", 10)?
            .set_default("auth.type", "mock")?
            .set_default("event.type", "memory")?
            // 環境変数から読み込み（USER_SERVICE_ プレフィックス）
            .add_source(
                config::Environment::with_prefix("USER_SERVICE")
                    .separator("__")
                    .try_parsing(true),
            )
            // 設定ファイルから読み込み（オプション）
            .add_source(
                config::File::with_name("config/user-service")
                    .required(false),
            )
            .build()?
            .try_deserialize()
    }

    /// 設定の妥当性を検証
    ///
    /// # Errors
    ///
    /// * バリデーションに失敗した場合
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        Validate::validate(self)
    }

    /// 安全な設定でのテスト用インスタンス作成
    #[must_use]
    pub fn for_test() -> Self {
        Self {
            server:   Server {
                host: String::from("127.0.0.1"),
                port: 50051,
            },
            database: Database {
                url:             String::from("postgres://localhost/effect_user_test"),
                max_connections: 5,
            },
            auth:     Auth::Mock {
                predefined_tokens: Some(vec![MockToken {
                    token:   String::from("test-token"),
                    user_id: String::from("test-user"),
                }]),
            },
            event:    Event::Memory,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_should_use_mock_auth() {
        // When
        let config = Config::default();

        // Then
        assert!(matches!(config.auth, Auth::Mock { .. }));
    }

    #[test]
    fn config_default_should_use_memory_event_bus() {
        // When
        let config = Config::default();

        // Then
        assert!(matches!(config.event, Event::Memory));
    }

    #[test]
    fn server_config_default_values() {
        // When
        let config = Server::default();

        // Then
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 50051);
    }

    #[test]
    fn config_validation_should_pass_for_default() {
        // Given
        let config = Config::default();

        // When
        let result = config.validate();

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn server_validation_should_fail_for_invalid_port() {
        // Given
        let server = Server {
            host: String::from("localhost"),
            port: 80, // ポート番号が範囲外（1024未満）
        };

        // When
        let result = server.validate();

        // Then
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("port"));
    }

    #[test]
    fn server_validation_should_fail_for_empty_host() {
        // Given
        let server = Server {
            host: String::new(), // 空のホスト名
            port: 50051,
        };

        // When
        let result = server.validate();

        // Then
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("host"));
    }

    #[test]
    fn database_validation_should_fail_for_invalid_url() {
        // Given
        let database = Database {
            url:             String::from("mysql://localhost/test"), // PostgreSQL以外のURL
            max_connections: 10,
        };

        // When
        let result = database.validate();

        // Then
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("url"));
    }

    #[test]
    fn database_validation_should_fail_for_invalid_max_connections() {
        // Given
        let database = Database {
            url:             String::from("postgres://localhost/test"),
            max_connections: 0, // 最小値未満
        };

        // When
        let result = database.validate();

        // Then
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("max_connections"));
    }

    #[test]
    fn database_validation_should_fail_for_too_many_connections() {
        // Given
        let database = Database {
            url:             String::from("postgres://localhost/test"),
            max_connections: 101, // 最大値超過
        };

        // When
        let result = database.validate();

        // Then
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("max_connections"));
    }

    #[test]
    fn config_for_test_should_be_valid() {
        // When
        let config = Config::for_test();

        // Then
        assert!(config.validate().is_ok());
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 50051);
        assert!(config.database.url.contains("test"));
        assert!(matches!(config.auth, Auth::Mock { .. }));
        assert!(matches!(config.event, Event::Memory));
    }

    #[test]
    fn config_load_should_use_defaults() {
        // When
        let result = Config::load();

        // Then
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 50051);
        assert_eq!(config.database.max_connections, 10);
    }

    #[test]
    fn config_builder_can_create_valid_config() {
        // Given
        let config_builder = config::Config::builder()
            .set_default("server.host", "localhost")
            .unwrap()
            .set_default("server.port", 8080)
            .unwrap()
            .set_default("database.url", "postgres://localhost/test")
            .unwrap()
            .set_default("database.max_connections", 5)
            .unwrap()
            .set_default("auth.type", "mock")
            .unwrap()
            .set_default("event.type", "memory")
            .unwrap()
            .build()
            .unwrap();

        // When
        let result: Result<Config, _> = config_builder.try_deserialize();

        // Then
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 5);
    }

    #[test]
    fn validate_port_should_reject_system_ports() {
        // Given
        let port = 80u16;

        // When
        let result = validate_port(port);

        // Then
        assert!(result.is_err());
    }

    #[test]
    fn validate_port_should_accept_valid_ports() {
        // Given
        let port = 8080u16;

        // When
        let result = validate_port(port);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn validate_database_url_should_accept_postgres_schemes() {
        // Given
        let urls = vec![
            "postgres://localhost/test",
            "postgresql://user:pass@localhost:5432/db",
        ];

        for url in urls {
            // When
            let result = validate_database_url(url);

            // Then
            assert!(result.is_ok(), "Failed for URL: {url}");
        }
    }

    #[test]
    fn validate_database_url_should_reject_non_postgres_schemes() {
        // Given
        let urls = vec![
            "mysql://localhost/test",
            "sqlite:///path/to/db.sqlite",
            "mongodb://localhost/test",
        ];

        for url in urls {
            // When
            let result = validate_database_url(url);

            // Then
            assert!(result.is_err(), "Should have failed for URL: {url}");
        }
    }
}
