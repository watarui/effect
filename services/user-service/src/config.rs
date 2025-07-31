//! サービス設定

use serde::{Deserialize, Serialize};

/// ユーザーサービスの設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// サーバー設定
    pub server:   Server,
    /// データベース設定
    pub database: Database,
    /// 認証設定
    pub auth:     Auth,
    /// イベントバス設定
    pub event:    Event,
}

/// サーバー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// ホスト（デフォルト: 0.0.0.0）
    pub host: String,
    /// ポート（デフォルト: 50051）
    pub port: u16,
}

/// データベース設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    /// データベース URL
    pub url:             String,
    /// 最大接続数
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
            host: "0.0.0.0".to_string(),
            port: 50051,
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self {
            url:             "postgres://localhost/effect_user".to_string(),
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
    /// 環境変数から設定を読み込む
    ///
    /// # Errors
    ///
    /// * 環境変数の読み込みに失敗した場合
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
}
