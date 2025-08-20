//! 値オブジェクト

use serde::{Deserialize, Serialize};

/// 学習統計
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StudyStats {
    pub total_time_minutes:   i32,
    pub sessions_count:       i32,
    pub average_session_time: i32,
    pub completion_rate:      f32,
}

/// 正答率統計
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccuracyStats {
    pub total_attempts:   i32,
    pub correct_answers:  i32,
    pub accuracy_rate:    f32,
    pub improvement_rate: f32,
}

/// マスタリーレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MasteryLevel {
    Beginner,
    Learning,
    Familiar,
    Proficient,
    Mastered,
}

impl MasteryLevel {
    pub fn from_accuracy(accuracy: f32, attempts: i32) -> Self {
        match (accuracy, attempts) {
            (a, t) if t >= 10 && a >= 0.95 => Self::Mastered,
            (a, t) if t >= 5 && a >= 0.85 => Self::Proficient,
            (a, t) if t >= 3 && a >= 0.70 => Self::Familiar,
            (_, t) if t >= 1 => Self::Learning,
            _ => Self::Beginner,
        }
    }
}

/// ストリーク状態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakInfo {
    pub current_days:     i32,
    pub longest_days:     i32,
    pub last_active_date: Option<chrono::NaiveDate>,
    pub is_active:        bool,
}
