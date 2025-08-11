//! Repository for user learning statistics

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::RepositoryResult;

/// User learning statistics entity
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct UserLearningStatistics {
    /// 一意識別子
    pub id: Uuid,
    /// ユーザー ID
    pub user_id: Uuid,
    /// 総項目数
    pub total_items: i32,
    /// 習得済み項目数
    pub mastered_items: i32,
    /// 学習中項目数
    pub learning_items: i32,
    /// 新規項目数
    pub new_items: i32,
    /// 総セッション数
    pub total_sessions: i32,
    /// 総復習回数
    pub total_reviews: i32,
    /// 全体正答率
    pub overall_accuracy: f32,
    /// 平均セッション時間（秒）
    pub average_session_duration_seconds: i32,
    /// 一日平均復習数
    pub daily_review_average: f32,
    /// 現在の連続学習日数
    pub current_streak_days: i32,
    /// 最長連続学習日数
    pub longest_streak_days: i32,
    /// 最終計算日時
    pub last_calculated_at: DateTime<Utc>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
}

/// Performance analysis entity
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    /// 一意識別子
    pub id:                     Uuid,
    /// ユーザー ID
    pub user_id:                Uuid,
    /// 分析開始日時
    pub analyzed_from:          DateTime<Utc>,
    /// 分析終了日時
    pub analyzed_to:            DateTime<Utc>,
    /// 正答率の傾向
    pub accuracy_trend:         f32,
    /// 回答速度の傾向
    pub speed_trend:            f32,
    /// 定着率の傾向
    pub retention_trend:        f32,
    /// 問題のあるカテゴリ（JSON）
    pub problematic_categories: serde_json::Value,
    /// 得意なカテゴリ（JSON）
    pub strong_categories:      serde_json::Value,
    /// アクティブな時間帯（JSON）
    pub active_hours:           serde_json::Value,
    /// 一貫性スコア
    pub consistency_score:      f32,
    /// 予測習得日数
    pub predicted_mastery_days: Option<i32>,
    /// バーンアウトリスク
    pub burnout_risk:           f32,
    /// 作成日時
    pub created_at:             DateTime<Utc>,
}

/// Repository trait for statistics
#[async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait StatisticsRepository: Send + Sync {
    /// Get statistics for a user
    async fn get_user_statistics(
        &self,
        user_id: Uuid,
    ) -> RepositoryResult<Option<UserLearningStatistics>>;

    /// Update user statistics
    async fn update_user_statistics(&self, stats: &UserLearningStatistics) -> RepositoryResult<()>;

    /// Create or update user statistics
    async fn upsert_user_statistics(&self, stats: &UserLearningStatistics) -> RepositoryResult<()>;

    /// Save performance analysis
    async fn save_performance_analysis(
        &self,
        analysis: &PerformanceAnalysis,
    ) -> RepositoryResult<Uuid>;

    /// Get latest performance analysis
    async fn get_latest_performance_analysis(
        &self,
        user_id: Uuid,
    ) -> RepositoryResult<Option<PerformanceAnalysis>>;

    /// Get performance analyses in period
    async fn get_performance_analyses_in_period(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> RepositoryResult<Vec<PerformanceAnalysis>>;
}

/// `PostgreSQL` implementation of `StatisticsRepository`
pub struct PostgresRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl PostgresRepository {
    /// 新しい `PostgresRepository` を作成
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StatisticsRepository for PostgresRepository {
    async fn get_user_statistics(
        &self,
        user_id: Uuid,
    ) -> RepositoryResult<Option<UserLearningStatistics>> {
        let result = sqlx::query_as!(
            UserLearningStatistics,
            r#"
            SELECT 
                id, user_id, total_items, mastered_items, learning_items,
                new_items, total_sessions, total_reviews, overall_accuracy,
                average_session_duration_seconds, daily_review_average,
                current_streak_days, longest_streak_days,
                last_calculated_at, created_at, updated_at
            FROM user_learning_statistics
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update_user_statistics(&self, stats: &UserLearningStatistics) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            UPDATE user_learning_statistics SET
                total_items = $3,
                mastered_items = $4,
                learning_items = $5,
                new_items = $6,
                total_sessions = $7,
                total_reviews = $8,
                overall_accuracy = $9,
                average_session_duration_seconds = $10,
                daily_review_average = $11,
                current_streak_days = $12,
                longest_streak_days = $13,
                last_calculated_at = NOW(),
                updated_at = NOW()
            WHERE user_id = $1 AND id = $2
            "#,
            stats.user_id,
            stats.id,
            stats.total_items,
            stats.mastered_items,
            stats.learning_items,
            stats.new_items,
            stats.total_sessions,
            stats.total_reviews,
            stats.overall_accuracy,
            stats.average_session_duration_seconds,
            stats.daily_review_average,
            stats.current_streak_days,
            stats.longest_streak_days
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn upsert_user_statistics(&self, stats: &UserLearningStatistics) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_learning_statistics (
                id, user_id, total_items, mastered_items, learning_items,
                new_items, total_sessions, total_reviews, overall_accuracy,
                average_session_duration_seconds, daily_review_average,
                current_streak_days, longest_streak_days
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
            )
            ON CONFLICT (user_id) DO UPDATE SET
                total_items = EXCLUDED.total_items,
                mastered_items = EXCLUDED.mastered_items,
                learning_items = EXCLUDED.learning_items,
                new_items = EXCLUDED.new_items,
                total_sessions = EXCLUDED.total_sessions,
                total_reviews = EXCLUDED.total_reviews,
                overall_accuracy = EXCLUDED.overall_accuracy,
                average_session_duration_seconds = EXCLUDED.average_session_duration_seconds,
                daily_review_average = EXCLUDED.daily_review_average,
                current_streak_days = EXCLUDED.current_streak_days,
                longest_streak_days = EXCLUDED.longest_streak_days,
                last_calculated_at = NOW(),
                updated_at = NOW()
            "#,
            Uuid::new_v4(),
            stats.user_id,
            stats.total_items,
            stats.mastered_items,
            stats.learning_items,
            stats.new_items,
            stats.total_sessions,
            stats.total_reviews,
            stats.overall_accuracy,
            stats.average_session_duration_seconds,
            stats.daily_review_average,
            stats.current_streak_days,
            stats.longest_streak_days
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save_performance_analysis(
        &self,
        analysis: &PerformanceAnalysis,
    ) -> RepositoryResult<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO performance_analyses (
                id, user_id, analyzed_from, analyzed_to,
                accuracy_trend, speed_trend, retention_trend,
                problematic_categories, strong_categories, active_hours,
                consistency_score, predicted_mastery_days, burnout_risk
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
            )
            "#,
            id,
            analysis.user_id,
            analysis.analyzed_from,
            analysis.analyzed_to,
            analysis.accuracy_trend,
            analysis.speed_trend,
            analysis.retention_trend,
            analysis.problematic_categories,
            analysis.strong_categories,
            analysis.active_hours,
            analysis.consistency_score,
            analysis.predicted_mastery_days,
            analysis.burnout_risk
        )
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    async fn get_latest_performance_analysis(
        &self,
        user_id: Uuid,
    ) -> RepositoryResult<Option<PerformanceAnalysis>> {
        let result = sqlx::query_as!(
            PerformanceAnalysis,
            r#"
            SELECT 
                id, user_id, analyzed_from, analyzed_to,
                accuracy_trend, speed_trend, retention_trend,
                problematic_categories, strong_categories, active_hours,
                consistency_score, predicted_mastery_days, burnout_risk,
                created_at
            FROM performance_analyses
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn get_performance_analyses_in_period(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> RepositoryResult<Vec<PerformanceAnalysis>> {
        let analyses = sqlx::query_as!(
            PerformanceAnalysis,
            r#"
            SELECT 
                id, user_id, analyzed_from, analyzed_to,
                accuracy_trend, speed_trend, retention_trend,
                problematic_categories, strong_categories, active_hours,
                consistency_score, predicted_mastery_days, burnout_risk,
                created_at
            FROM performance_analyses
            WHERE user_id = $1 
                AND analyzed_from >= $2 
                AND analyzed_to <= $3
            ORDER BY created_at DESC
            "#,
            user_id,
            from,
            to
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(analyses)
    }
}
