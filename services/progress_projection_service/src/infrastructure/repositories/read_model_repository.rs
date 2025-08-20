//! Read Model リポジトリ実装

use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::*,
    error::{Error, Result},
    ports::outbound::ReadModelRepository,
};

/// PostgreSQL Read Model リポジトリ
pub struct PostgresReadModelRepository {
    pool: PgPool,
}

impl PostgresReadModelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReadModelRepository for PostgresReadModelRepository {
    async fn save_user_progress(&self, progress: &UserProgress) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_progress (
                user_id, total_items_learned, total_items_mastered,
                total_study_minutes, current_streak_days, longest_streak_days,
                last_study_date, achievements_unlocked, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (user_id)
            DO UPDATE SET
                total_items_learned = EXCLUDED.total_items_learned,
                total_items_mastered = EXCLUDED.total_items_mastered,
                total_study_minutes = EXCLUDED.total_study_minutes,
                current_streak_days = EXCLUDED.current_streak_days,
                longest_streak_days = EXCLUDED.longest_streak_days,
                last_study_date = EXCLUDED.last_study_date,
                achievements_unlocked = EXCLUDED.achievements_unlocked,
                updated_at = EXCLUDED.updated_at
            "#,
            progress.user_id,
            progress.total_items_learned,
            progress.total_items_mastered,
            progress.total_study_minutes,
            progress.current_streak_days,
            progress.longest_streak_days,
            progress.last_study_date,
            &progress.achievements_unlocked,
            progress.created_at,
            progress.updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    async fn get_user_progress(&self, user_id: Uuid) -> Result<Option<UserProgress>> {
        let record = sqlx::query!(
            r#"
            SELECT * FROM user_progress WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(record.map(|r| UserProgress {
            user_id:               r.user_id,
            total_items_learned:   r.total_items_learned,
            total_items_mastered:  r.total_items_mastered,
            total_study_minutes:   r.total_study_minutes,
            current_streak_days:   r.current_streak_days,
            longest_streak_days:   r.longest_streak_days,
            last_study_date:       r.last_study_date,
            achievements_unlocked: r.achievements_unlocked,
            created_at:            r.created_at,
            updated_at:            r.updated_at,
        }))
    }

    async fn save_daily_progress(&self, progress: &DailyProgress) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO daily_progress (
                user_id, date, items_learned, items_reviewed, items_mastered,
                correct_answers, total_answers, study_minutes, sessions_count,
                goal_completed, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (user_id, date)
            DO UPDATE SET
                items_learned = EXCLUDED.items_learned,
                items_reviewed = EXCLUDED.items_reviewed,
                items_mastered = EXCLUDED.items_mastered,
                correct_answers = EXCLUDED.correct_answers,
                total_answers = EXCLUDED.total_answers,
                study_minutes = EXCLUDED.study_minutes,
                sessions_count = EXCLUDED.sessions_count,
                goal_completed = EXCLUDED.goal_completed,
                updated_at = EXCLUDED.updated_at
            "#,
            progress.user_id,
            progress.date,
            progress.items_learned,
            progress.items_reviewed,
            progress.items_mastered,
            progress.correct_answers,
            progress.total_answers,
            progress.study_minutes,
            progress.sessions_count,
            progress.goal_completed,
            progress.created_at,
            progress.updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    async fn get_daily_progress(
        &self,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<DailyProgress>> {
        let record = sqlx::query!(
            r#"
            SELECT * FROM daily_progress WHERE user_id = $1 AND date = $2
            "#,
            user_id,
            date
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(record.map(|r| DailyProgress {
            user_id:         r.user_id,
            date:            r.date,
            items_learned:   r.items_learned,
            items_reviewed:  r.items_reviewed,
            items_mastered:  r.items_mastered,
            correct_answers: r.correct_answers,
            total_answers:   r.total_answers,
            study_minutes:   r.study_minutes,
            sessions_count:  r.sessions_count,
            goal_completed:  r.goal_completed,
            created_at:      r.created_at,
            updated_at:      r.updated_at,
        }))
    }

    async fn save_weekly_progress(&self, progress: &WeeklyProgress) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO weekly_progress (
                user_id, week_start_date, week_end_date, items_learned, items_reviewed,
                items_mastered, study_minutes, study_days, goals_completed,
                average_accuracy, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (user_id, week_start_date)
            DO UPDATE SET
                week_end_date = EXCLUDED.week_end_date,
                items_learned = EXCLUDED.items_learned,
                items_reviewed = EXCLUDED.items_reviewed,
                items_mastered = EXCLUDED.items_mastered,
                study_minutes = EXCLUDED.study_minutes,
                study_days = EXCLUDED.study_days,
                goals_completed = EXCLUDED.goals_completed,
                average_accuracy = EXCLUDED.average_accuracy,
                updated_at = EXCLUDED.updated_at
            "#,
            progress.user_id,
            progress.week_start_date,
            progress.week_end_date,
            progress.items_learned,
            progress.items_reviewed,
            progress.items_mastered,
            progress.study_minutes,
            progress.study_days,
            progress.goals_completed,
            progress.average_accuracy,
            progress.created_at,
            progress.updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    async fn get_weekly_progress(
        &self,
        user_id: Uuid,
        week_start: NaiveDate,
    ) -> Result<Option<WeeklyProgress>> {
        let record = sqlx::query!(
            r#"
            SELECT * FROM weekly_progress WHERE user_id = $1 AND week_start_date = $2
            "#,
            user_id,
            week_start
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(record.map(|r| WeeklyProgress {
            user_id:          r.user_id,
            week_start_date:  r.week_start_date,
            week_end_date:    r.week_end_date,
            items_learned:    r.items_learned,
            items_reviewed:   r.items_reviewed,
            items_mastered:   r.items_mastered,
            study_minutes:    r.study_minutes,
            study_days:       r.study_days,
            goals_completed:  r.goals_completed,
            average_accuracy: r.average_accuracy,
            created_at:       r.created_at,
            updated_at:       r.updated_at,
        }))
    }

    async fn save_vocabulary_item_progress(&self, progress: &VocabularyItemProgress) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO vocabulary_item_progress (
                user_id, vocabulary_item_id, attempts_count, correct_count,
                last_attempt_date, last_accuracy, average_accuracy, mastery_level,
                time_spent_seconds, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (user_id, vocabulary_item_id)
            DO UPDATE SET
                attempts_count = EXCLUDED.attempts_count,
                correct_count = EXCLUDED.correct_count,
                last_attempt_date = EXCLUDED.last_attempt_date,
                last_accuracy = EXCLUDED.last_accuracy,
                average_accuracy = EXCLUDED.average_accuracy,
                mastery_level = EXCLUDED.mastery_level,
                time_spent_seconds = EXCLUDED.time_spent_seconds,
                updated_at = EXCLUDED.updated_at
            "#,
            progress.user_id,
            progress.vocabulary_item_id,
            progress.attempts_count,
            progress.correct_count,
            progress.last_attempt_date,
            progress.last_accuracy,
            progress.average_accuracy,
            progress.mastery_level,
            progress.time_spent_seconds,
            progress.created_at,
            progress.updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    async fn get_vocabulary_item_progress(
        &self,
        user_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<VocabularyItemProgress>> {
        let record = sqlx::query!(
            r#"
            SELECT * FROM vocabulary_item_progress 
            WHERE user_id = $1 AND vocabulary_item_id = $2
            "#,
            user_id,
            item_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(record.map(|r| VocabularyItemProgress {
            user_id:            r.user_id,
            vocabulary_item_id: r.vocabulary_item_id,
            attempts_count:     r.attempts_count,
            correct_count:      r.correct_count,
            last_attempt_date:  r.last_attempt_date,
            last_accuracy:      r.last_accuracy,
            average_accuracy:   r.average_accuracy,
            mastery_level:      r.mastery_level,
            time_spent_seconds: r.time_spent_seconds,
            created_at:         r.created_at,
            updated_at:         r.updated_at,
        }))
    }

    async fn save_achievement(&self, achievement: &Achievement) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO achievements (
                user_id, achievement_id, name, description, category,
                unlocked_at, progress, target
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (user_id, achievement_id)
            DO UPDATE SET
                unlocked_at = EXCLUDED.unlocked_at,
                progress = EXCLUDED.progress
            "#,
            achievement.user_id,
            achievement.achievement_id,
            achievement.name,
            achievement.description,
            achievement.category,
            achievement.unlocked_at,
            achievement.progress,
            achievement.target
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    async fn get_user_achievements(&self, user_id: Uuid) -> Result<Vec<Achievement>> {
        let records = sqlx::query!(
            r#"
            SELECT * FROM achievements WHERE user_id = $1
            ORDER BY unlocked_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(records
            .into_iter()
            .map(|r| Achievement {
                user_id:        r.user_id,
                achievement_id: r.achievement_id,
                name:           r.name,
                description:    r.description,
                category:       r.category,
                unlocked_at:    r.unlocked_at,
                progress:       r.progress,
                target:         r.target,
            })
            .collect())
    }
}
