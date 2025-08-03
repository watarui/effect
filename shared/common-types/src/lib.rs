//! 一時的な互換性レイヤー
//!
//! このモジュールは移行期間中の後方互換性のために存在します。
//! 新しいコードでは直接 shared-kernel を使用してください。

// shared-kernel の内容を再エクスポート
pub use shared_kernel::*;
