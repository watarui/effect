//! Domain Events Service の設定

use serde::Deserialize;

/// サービス設定
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// サーバーポート
    #[serde(default = "default_port")]
    #[allow(dead_code)]
    pub port: u16,

    /// データベース URL
    pub database_url: String,

    /// gRPC サーバー設定
    #[serde(default)]
    pub grpc: GrpcConfig,

    /// スキーマレジストリ設定
    #[serde(default)]
    pub registry: RegistryConfig,
}

/// gRPC サーバー設定
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GrpcConfig {
    /// gRPC ポート
    #[serde(default = "default_grpc_port")]
    pub port: u16,

    /// 最大メッセージサイズ (bytes)
    #[serde(default = "default_max_message_size")]
    #[allow(dead_code)]
    pub max_message_size: usize,
}

/// スキーマレジストリ設定
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RegistryConfig {
    /// スキーマキャッシュの有効期限（秒）
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,

    /// スキーマの最大バージョン数
    #[serde(default = "default_max_versions")]
    pub max_versions: usize,
}

impl Config {
    /// 環境変数から設定を読み込む
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut builder = config::Config::builder();

        // デフォルト値を設定
        builder = builder
            .set_default("port", default_port())?
            .set_default("grpc.port", default_grpc_port())?
            .set_default(
                "grpc.max_message_size",
                i64::try_from(default_max_message_size()).unwrap_or(i64::MAX),
            )?
            .set_default("registry.cache_ttl_seconds", default_cache_ttl())?
            .set_default(
                "registry.max_versions",
                i64::try_from(default_max_versions()).unwrap_or(i64::MAX),
            )?;

        // 環境変数から設定を読み込む
        builder = builder.add_source(
            config::Environment::with_prefix("DOMAIN_EVENTS")
                .separator("_")
                .try_parsing(true),
        );

        // DATABASE_URL は共通の環境変数
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            builder = builder.set_override("database_url", database_url)?;
        }

        builder.build()?.try_deserialize()
    }
}

const fn default_port() -> u16 {
    8090
}

const fn default_grpc_port() -> u16 {
    50053
}

const fn default_max_message_size() -> usize {
    4 * 1024 * 1024 // 4MB
}

const fn default_cache_ttl() -> u64 {
    300 // 5 minutes
}

const fn default_max_versions() -> usize {
    10
}

/// 設定を読み込む
pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    Config::from_env().map_err(Into::into)
}
