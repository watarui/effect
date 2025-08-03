use serde::{Deserialize, Serialize};

/// コースタイプ（全コンテキストで共通の意味）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CourseType {
    Ielts,
    Toefl,
    Toeic,
    Eiken,
    GeneralEnglish,
}

/// CEFR レベル（全コンテキストで共通の意味）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CefrLevel {
    A1, // Beginner
    A2, // Elementary
    B1, // Intermediate
    B2, // Upper Intermediate
    C1, // Advanced
    C2, // Proficient
}

/// 言語コード（ISO 639-1 形式）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageCode(String);

impl LanguageCode {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 反応タイプ（学習セッションでの反応）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Correct,
    Incorrect,
    Skipped,
}

/// マスタリーステータス（学習状態）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MasteryStatus {
    Unknown,   // 未学習
    Tested,    // テスト済み（1回以上正解）
    ShortTerm, // 短期記憶に定着
    LongTerm,  // 長期記憶に定着
}
