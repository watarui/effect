//! Progress Read Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ユーザー進捗リードモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgress {
    pub user_id:               Uuid,
    pub total_items_learned:   i32,
    pub total_items_mastered:  i32,
    pub total_study_minutes:   i32,
    pub current_streak_days:   i32,
    pub longest_streak_days:   i32,
    pub last_study_date:       Option<DateTime<Utc>>,
    pub achievements_unlocked: Vec<String>,
    pub created_at:            DateTime<Utc>,
    pub updated_at:            DateTime<Utc>,
}

/// 日次進捗リードモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyProgress {
    pub user_id:         Uuid,
    pub date:            chrono::NaiveDate,
    pub items_learned:   i32,
    pub items_reviewed:  i32,
    pub items_mastered:  i32,
    pub correct_answers: i32,
    pub total_answers:   i32,
    pub study_minutes:   i32,
    pub sessions_count:  i32,
    pub goal_completed:  bool,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

/// 週次進捗リードモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyProgress {
    pub user_id:          Uuid,
    pub week_start_date:  chrono::NaiveDate,
    pub week_end_date:    chrono::NaiveDate,
    pub items_learned:    i32,
    pub items_reviewed:   i32,
    pub items_mastered:   i32,
    pub study_minutes:    i32,
    pub study_days:       i32,
    pub goals_completed:  i32,
    pub average_accuracy: f32,
    pub created_at:       DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,
}

/// 語彙アイテム進捗リードモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyItemProgress {
    pub user_id:            Uuid,
    pub vocabulary_item_id: Uuid,
    pub attempts_count:     i32,
    pub correct_count:      i32,
    pub last_attempt_date:  DateTime<Utc>,
    pub last_accuracy:      f32,
    pub average_accuracy:   f32,
    pub mastery_level:      i32,
    pub time_spent_seconds: i32,
    pub created_at:         DateTime<Utc>,
    pub updated_at:         DateTime<Utc>,
}

/// アチーブメントリードモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub user_id:        Uuid,
    pub achievement_id: String,
    pub name:           String,
    pub description:    String,
    pub category:       String,
    pub unlocked_at:    DateTime<Utc>,
    pub progress:       i32,
    pub target:         i32,
}

/// プロジェクション状態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionState {
    pub projection_name: String,
    pub last_position:   i64,
    pub last_event_id:   Option<Uuid>,
    pub updated_at:      DateTime<Utc>,
}
