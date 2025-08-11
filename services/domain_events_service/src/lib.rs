//! Domain Events Service ライブラリ
//!
//! イベントスキーマ管理とクライアントライブラリを提供

pub mod client;
pub mod config;
pub mod grpc;
pub mod registry;
pub mod schemas;
pub mod validator;

// クライアント用の再エクスポート
pub use client::Client;
