//! Vocabulary Context 共有ライブラリ
//!
//! Vocabulary Context 内の各マイクロサービス間で共有される
//! ドメインモデル、イベント、コマンド、クエリを定義

pub mod commands;
pub mod domain;
pub mod events;
pub mod queries;

// Re-export commonly used items
pub use commands::*;
pub use domain::{VocabularyEntry, VocabularyItem};
pub use events::*;
pub use queries::*;
