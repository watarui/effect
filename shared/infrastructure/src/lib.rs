//! 共有インフラストラクチャコンポーネント
//!
//! このクレートはキャッシュと設定管理の基本的な
//! インフラストラクチャコンポーネントを提供します。
//!
//! 注意: 他のインフラストラクチャコンポーネントは個別のクレートに
//! 移動されました：
//! - database → shared-database
//! - `event_bus` → shared-event-bus
//! - `event_store` → shared-event-store
//! - repository → shared-repository

/// キャッシュモジュール
pub mod cache;
/// 設定管理モジュール
pub mod config;

// Re-export commonly used types
pub use cache::{Client as CacheClient, Error as CacheError};
pub use config::{
    Environment,
    Error as ConfigError,
    get_env,
    get_env_or,
    get_env_parse,
    get_env_parse_or,
};
// Re-export hex for macros
pub use hex;
