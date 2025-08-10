//! Domain Events Service ライブラリ
//!
//! イベントスキーマ管理とクライアントライブラリを提供

// Clippy の警告を抑制
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(missing_docs)]

pub mod client;
pub mod config;
pub mod grpc;
pub mod registry;
pub mod schemas;
pub mod validator;

// クライアント用の再エクスポート
pub use client::DomainEventsClient;
