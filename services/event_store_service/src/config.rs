//! Event Store Service の設定

use serde::{Deserialize, Serialize};

/// サービス設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// gRPC サーバーのポート
    pub port: u16,

    /// データベース URL
    pub database_url: String,

    /// スナップショット設定
    pub snapshot: SnapshotConfig,
}

/// スナップショット設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// スナップショットを作成するイベント数の閾値
    pub threshold: u32,

    /// スナップショットの保持期間（日）
    pub retention_days: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port:         50051,
            database_url: "postgres://effect:effect_password@localhost:5432/event_store_db"
                .to_string(),
            snapshot:     SnapshotConfig {
                threshold:      100,
                retention_days: 30,
            },
        }
    }
}

/// 設定を読み込む
pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    // 環境変数から読み込み
    let config = Config {
        port:         std::env::var("PORT")
            .unwrap_or_else(|_| "50051".to_string())
            .parse()?,
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| Config::default().database_url),
        snapshot:     SnapshotConfig {
            threshold:      std::env::var("SNAPSHOT_THRESHOLD")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            retention_days: std::env::var("SNAPSHOT_RETENTION_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
        },
    };

    Ok(config)
}
