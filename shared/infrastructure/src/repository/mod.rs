//! リポジトリパターンの基底実装
//!
//! このモジュールは全てのリポジトリが共通で使用する
//! 基底トレイトと実装を提供します。

pub mod base;
pub mod entity;
pub mod error;
pub mod id;
pub mod postgres;
pub mod transaction;

// Re-export commonly used types
pub use base::{Repository, SoftDeletable};
pub use entity::{Entity, SoftDeletable as EntitySoftDeletable};
pub use error::{Error, Result};
pub use id::Bytes;
