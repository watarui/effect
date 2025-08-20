//! Progress ドメインイベント

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Progress イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProgressEvent {
    /// 学習開始
    LearningStarted {
        user_id:   Uuid,
        timestamp: DateTime<Utc>,
    },

    /// アイテム完了
    ItemCompleted {
        user_id:            Uuid,
        vocabulary_item_id: Uuid,
        accuracy:           f32,
        time_spent:         i32,
        timestamp:          DateTime<Utc>,
    },

    /// セッション完了
    SessionCompleted {
        user_id:       Uuid,
        items_count:   i32,
        correct_count: i32,
        duration:      i32,
        timestamp:     DateTime<Utc>,
    },

    /// ストリーク更新
    StreakUpdated {
        user_id:    Uuid,
        new_streak: i32,
        timestamp:  DateTime<Utc>,
    },

    /// アチーブメント獲得
    AchievementUnlocked {
        user_id:        Uuid,
        achievement_id: String,
        timestamp:      DateTime<Utc>,
    },

    /// デイリーゴール達成
    DailyGoalCompleted {
        user_id:   Uuid,
        timestamp: DateTime<Utc>,
    },
}
