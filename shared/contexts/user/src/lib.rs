//! User Context 共有ライブラリ
//!
//! User Context 内の各マイクロサービス間で共有される
//! ドメインモデル、イベント、コマンド、クエリを定義

pub mod events;

// Re-export commonly used items
pub use events::*;
