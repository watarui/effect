//! イベントスキーマ定義モジュール
//!
//! 各 Bounded Context のイベントスキーマを管理

pub mod ai;
pub mod algorithm;
pub mod learning;
pub mod user;
pub mod vocabulary;

use std::collections::HashMap;

/// すべてのイベントスキーマを初期化
#[must_use]
#[allow(dead_code)]
pub fn initialize() -> HashMap<String, String> {
    let mut schemas = HashMap::new();

    // Vocabulary Context のスキーマ
    schemas.extend(vocabulary::get_schemas());

    // Learning Context のスキーマ
    schemas.extend(learning::get_schemas());

    // User Context のスキーマ
    schemas.extend(user::get_schemas());

    // Algorithm Context のスキーマ
    schemas.extend(algorithm::get_schemas());

    // AI Context のスキーマ
    schemas.extend(ai::get_schemas());

    schemas
}
