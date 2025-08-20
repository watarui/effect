//! Progress 値オブジェクト

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 語彙進捗
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VocabularyProgress {
    pub learned_items:  i32,
    pub mastered_items: i32,
    pub reviewed_items: i32,
    pub streak_days:    i32,
    pub item_progress:  HashMap<Uuid, ItemProgress>,
}

impl VocabularyProgress {
    pub fn update_item(&mut self, item_id: Uuid, accuracy: f32) {
        let progress = self.item_progress.entry(item_id).or_default();
        progress.attempts += 1;
        if accuracy >= 0.8 {
            progress.correct_count += 1;
        }
        progress.last_accuracy = accuracy;
    }
}

/// アイテム進捗
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemProgress {
    pub attempts:      i32,
    pub correct_count: i32,
    pub last_accuracy: f32,
}

/// 日次進捗統計
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyProgressStats {
    pub items_learned:    i32,
    pub items_reviewed:   i32,
    pub items_mastered:   i32,
    pub correct_count:    i32,
    pub total_time_spent: i32,
    pub session_count:    i32,
    pub goal_completed:   bool,
}

/// 週次進捗統計
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeeklyProgressStats {
    pub items_learned:   i32,
    pub items_reviewed:  i32,
    pub items_mastered:  i32,
    pub correct_count:   i32,
    pub study_minutes:   i32,
    pub goals_completed: i32,
}

/// 全体進捗統計
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TotalProgressStats {
    pub total_items_learned: i32,
    pub total_sessions:      i32,
    pub total_time_spent:    i32,
    pub current_streak:      i32,
    pub longest_streak:      i32,
    pub achievements_count:  i32,
}
