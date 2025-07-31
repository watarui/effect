//! タイムスタンプユーティリティ
//!
//! このモジュールはタイムスタンプ関連の型とユーティリティを含みます。

use chrono::{DateTime, Utc};

/// アプリケーション全体で使用される標準タイムスタンプ型
pub type Timestamp = DateTime<Utc>;

/// 現在時刻で新しいタイムスタンプを作成
#[must_use]
pub fn now() -> Timestamp {
    Utc::now()
}
