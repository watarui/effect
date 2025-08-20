//! イベントハンドラー

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::{
    domain::*,
    error::Result,
    ports::outbound::{Event, ReadModelRepository},
};

/// Progress イベントハンドラー
pub struct ProgressEventHandler {
    repository: Arc<dyn ReadModelRepository>,
}

impl ProgressEventHandler {
    pub fn new(repository: Arc<dyn ReadModelRepository>) -> Self {
        Self { repository }
    }

    /// イベントを処理
    pub async fn handle_event(&self, event: &Event) -> Result<()> {
        match event.event_type.as_str() {
            "LearningStarted" => self.handle_learning_started(event).await,
            "ItemCompleted" => self.handle_item_completed(event).await,
            "SessionCompleted" => self.handle_session_completed(event).await,
            "StreakUpdated" => self.handle_streak_updated(event).await,
            "AchievementUnlocked" => self.handle_achievement_unlocked(event).await,
            "DailyGoalCompleted" => self.handle_daily_goal_completed(event).await,
            _ => Ok(()), // 未知のイベントタイプは無視
        }
    }

    async fn handle_learning_started(&self, event: &Event) -> Result<()> {
        let user_id: Uuid =
            serde_json::from_value(event.event_data.get("user_id").unwrap().clone()).unwrap();

        // ユーザー進捗を取得または作成
        let mut progress = self
            .repository
            .get_user_progress(user_id)
            .await?
            .unwrap_or_else(|| UserProgress {
                user_id,
                total_items_learned: 0,
                total_items_mastered: 0,
                total_study_minutes: 0,
                current_streak_days: 0,
                longest_streak_days: 0,
                last_study_date: None,
                achievements_unlocked: Vec::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });

        progress.last_study_date = Some(event.occurred_at);
        progress.updated_at = Utc::now();

        self.repository.save_user_progress(&progress).await?;
        Ok(())
    }

    async fn handle_item_completed(&self, event: &Event) -> Result<()> {
        let user_id: Uuid =
            serde_json::from_value(event.event_data.get("user_id").unwrap().clone()).unwrap();
        let vocabulary_item_id: Uuid =
            serde_json::from_value(event.event_data.get("vocabulary_item_id").unwrap().clone())
                .unwrap();
        let accuracy: f32 =
            serde_json::from_value(event.event_data.get("accuracy").unwrap().clone()).unwrap();
        let time_spent: i32 =
            serde_json::from_value(event.event_data.get("time_spent").unwrap().clone()).unwrap();

        // 語彙アイテム進捗を更新
        let mut item_progress = self
            .repository
            .get_vocabulary_item_progress(user_id, vocabulary_item_id)
            .await?
            .unwrap_or_else(|| VocabularyItemProgress {
                user_id,
                vocabulary_item_id,
                attempts_count: 0,
                correct_count: 0,
                last_attempt_date: event.occurred_at,
                last_accuracy: 0.0,
                average_accuracy: 0.0,
                mastery_level: 0,
                time_spent_seconds: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });

        item_progress.attempts_count += 1;
        if accuracy >= 0.8 {
            item_progress.correct_count += 1;
        }
        item_progress.last_attempt_date = event.occurred_at;
        item_progress.last_accuracy = accuracy;
        item_progress.average_accuracy =
            (item_progress.average_accuracy * (item_progress.attempts_count - 1) as f32 + accuracy)
                / item_progress.attempts_count as f32;
        item_progress.time_spent_seconds += time_spent;
        item_progress.mastery_level = MasteryLevel::from_accuracy(
            item_progress.average_accuracy,
            item_progress.attempts_count,
        ) as i32;
        item_progress.updated_at = Utc::now();

        self.repository
            .save_vocabulary_item_progress(&item_progress)
            .await?;

        // ユーザー進捗も更新
        if let Some(mut user_progress) = self.repository.get_user_progress(user_id).await? {
            user_progress.total_items_learned += 1;
            if item_progress.mastery_level >= MasteryLevel::Mastered as i32 {
                user_progress.total_items_mastered += 1;
            }
            user_progress.total_study_minutes += time_spent / 60;
            user_progress.updated_at = Utc::now();
            self.repository.save_user_progress(&user_progress).await?;
        }

        // 日次進捗を更新
        let date = event.occurred_at.date_naive();
        let mut daily_progress = self
            .repository
            .get_daily_progress(user_id, date)
            .await?
            .unwrap_or_else(|| DailyProgress {
                user_id,
                date,
                items_learned: 0,
                items_reviewed: 0,
                items_mastered: 0,
                correct_answers: 0,
                total_answers: 0,
                study_minutes: 0,
                sessions_count: 0,
                goal_completed: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });

        daily_progress.items_learned += 1;
        daily_progress.total_answers += 1;
        if accuracy >= 0.8 {
            daily_progress.correct_answers += 1;
        }
        daily_progress.study_minutes += time_spent / 60;
        daily_progress.updated_at = Utc::now();

        self.repository.save_daily_progress(&daily_progress).await?;

        Ok(())
    }

    async fn handle_session_completed(&self, event: &Event) -> Result<()> {
        let user_id: Uuid =
            serde_json::from_value(event.event_data.get("user_id").unwrap().clone()).unwrap();
        let _items_count: i32 =
            serde_json::from_value(event.event_data.get("items_count").unwrap().clone()).unwrap();
        let _correct_count: i32 =
            serde_json::from_value(event.event_data.get("correct_count").unwrap().clone()).unwrap();
        let _duration: i32 =
            serde_json::from_value(event.event_data.get("duration").unwrap().clone()).unwrap();

        // 日次進捗を更新
        let date = event.occurred_at.date_naive();
        if let Some(mut daily_progress) = self.repository.get_daily_progress(user_id, date).await? {
            daily_progress.sessions_count += 1;
            daily_progress.updated_at = Utc::now();
            self.repository.save_daily_progress(&daily_progress).await?;
        }

        Ok(())
    }

    async fn handle_streak_updated(&self, event: &Event) -> Result<()> {
        let user_id: Uuid =
            serde_json::from_value(event.event_data.get("user_id").unwrap().clone()).unwrap();
        let new_streak: i32 =
            serde_json::from_value(event.event_data.get("new_streak").unwrap().clone()).unwrap();

        if let Some(mut progress) = self.repository.get_user_progress(user_id).await? {
            progress.current_streak_days = new_streak;
            if new_streak > progress.longest_streak_days {
                progress.longest_streak_days = new_streak;
            }
            progress.updated_at = Utc::now();
            self.repository.save_user_progress(&progress).await?;
        }

        Ok(())
    }

    async fn handle_achievement_unlocked(&self, event: &Event) -> Result<()> {
        let user_id: Uuid =
            serde_json::from_value(event.event_data.get("user_id").unwrap().clone()).unwrap();
        let achievement_id: String =
            serde_json::from_value(event.event_data.get("achievement_id").unwrap().clone())
                .unwrap();

        let achievement = Achievement {
            user_id,
            achievement_id: achievement_id.clone(),
            name: achievement_id.clone(), // TODO: アチーブメント詳細情報を取得
            description: String::new(),
            category: String::from("general"),
            unlocked_at: event.occurred_at,
            progress: 100,
            target: 100,
        };

        self.repository.save_achievement(&achievement).await?;

        // ユーザー進捗も更新
        if let Some(mut progress) = self.repository.get_user_progress(user_id).await?
            && !progress.achievements_unlocked.contains(&achievement_id)
        {
            progress.achievements_unlocked.push(achievement_id);
            progress.updated_at = Utc::now();
            self.repository.save_user_progress(&progress).await?;
        }

        Ok(())
    }

    async fn handle_daily_goal_completed(&self, event: &Event) -> Result<()> {
        let user_id: Uuid =
            serde_json::from_value(event.event_data.get("user_id").unwrap().clone()).unwrap();

        let date = event.occurred_at.date_naive();
        if let Some(mut daily_progress) = self.repository.get_daily_progress(user_id, date).await? {
            daily_progress.goal_completed = true;
            daily_progress.updated_at = Utc::now();
            self.repository.save_daily_progress(&daily_progress).await?;
        }

        Ok(())
    }
}
