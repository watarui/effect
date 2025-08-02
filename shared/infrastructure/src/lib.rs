//! 共有インフラストラクチャコンポーネント
//!
//! このクレートはデータベース接続、メッセージバス、キャッシュなどの
//! インフラストラクチャコンポーネントを提供します。

pub mod cache;
pub mod config;
pub mod database;
pub mod event_bus;
pub mod event_store;
pub mod repository;

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
pub use database::{Config as DatabaseConfig, Error as DatabaseError, create_pool, health_check};
pub use event_bus::PubSubEventBus;
pub use event_store::{PostgresEventStore, SnapshotStore};
// Re-export hex for macros
pub use hex;
// Re-export repository types
pub use repository::{Entity, Error as RepositoryError, Repository, SoftDeletable};
