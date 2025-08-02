//! 全ての境界づけられたコンテキストで共有される共通型
//!
//! このモジュールは Effect アプリケーション全体で使用される
//! ID 値オブジェクトやその他の共通型を含みます。

mod error;
mod ids;
mod timestamp;

// Re-export all public types
pub use error::{DomainError, DomainResult};
pub use ids::{EntryId, EventId, ItemId, SessionId, UserId};
pub use timestamp::{JstExt, Timestamp, now};
