//! Shared Kernel - 全コンテキストで共有される最小限の概念
//!
//! このモジュールには、すべての Bounded Context で同じ意味と形式を持つ
//! 識別子、値オブジェクト、基本的な型定義のみを含めます。
//! ビジネスロジックは含めず、データ構造のみを定義します。

pub mod ids;
pub mod timestamp;
pub mod value_objects;

// Re-export commonly used items
pub use ids::*;
pub use timestamp::*;
pub use value_objects::*;

/// Shared Kernel のバージョン情報
pub const SHARED_KERNEL_VERSION: &str = "0.1.0";
