//! Progress 集約

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{events::ProgressEvent, value_objects::*};

/// Progress 集約ルート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub user_id:      Uuid,
    pub vocabulary:   VocabularyProgress,
    pub daily_stats:  DailyProgressStats,
    pub weekly_stats: WeeklyProgressStats,
    pub total_stats:  TotalProgressStats,
    pub version:      i64,
    pub updated_at:   DateTime<Utc>,
}

impl Progress {
    /// 新規作成
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            vocabulary: VocabularyProgress::default(),
            daily_stats: DailyProgressStats::default(),
            weekly_stats: WeeklyProgressStats::default(),
            total_stats: TotalProgressStats::default(),
            version: 0,
            updated_at: Utc::now(),
        }
    }

    /// イベントを適用
    pub fn apply(&mut self, event: &ProgressEvent) {
        match event {
            ProgressEvent::LearningStarted { timestamp, .. } => {
                self.daily_stats.session_count += 1;
                self.updated_at = *timestamp;
            },
            ProgressEvent::ItemCompleted {
                vocabulary_item_id,
                accuracy,
                time_spent,
                timestamp,
                ..
            } => {
                // 語彙アイテムの進捗を更新
                self.vocabulary.update_item(*vocabulary_item_id, *accuracy);

                // 日次統計を更新
                self.daily_stats.items_learned += 1;
                self.daily_stats.total_time_spent += time_spent;
                if *accuracy >= 0.8 {
                    self.daily_stats.correct_count += 1;
                }

                // 全体統計を更新
                self.total_stats.total_items_learned += 1;
                self.total_stats.total_time_spent += time_spent;

                self.updated_at = *timestamp;
            },
            ProgressEvent::SessionCompleted {
                items_count,
                correct_count,
                duration,
                timestamp,
                ..
            } => {
                // セッション統計を更新
                self.daily_stats.items_learned += items_count;
                self.daily_stats.correct_count += correct_count;
                self.daily_stats.total_time_spent += duration;

                self.total_stats.total_sessions += 1;
                self.total_stats.total_items_learned += items_count;
                self.total_stats.total_time_spent += duration;

                self.updated_at = *timestamp;
            },
            ProgressEvent::StreakUpdated {
                new_streak,
                timestamp,
                ..
            } => {
                self.total_stats.current_streak = *new_streak;
                if *new_streak > self.total_stats.longest_streak {
                    self.total_stats.longest_streak = *new_streak;
                }
                self.updated_at = *timestamp;
            },
            ProgressEvent::AchievementUnlocked { timestamp, .. } => {
                self.total_stats.achievements_count += 1;
                self.updated_at = *timestamp;
            },
            ProgressEvent::DailyGoalCompleted { timestamp, .. } => {
                self.daily_stats.goal_completed = true;
                self.updated_at = *timestamp;
            },
        }

        self.version += 1;
    }

    /// 日次リセット
    pub fn reset_daily(&mut self) {
        self.daily_stats = DailyProgressStats::default();
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// 週次リセット
    pub fn reset_weekly(&mut self) {
        self.weekly_stats = WeeklyProgressStats::default();
        self.updated_at = Utc::now();
        self.version += 1;
    }
}
