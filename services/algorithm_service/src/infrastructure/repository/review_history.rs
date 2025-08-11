//! Repository for review history

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::RepositoryResult;

/// Review history entity
#[derive(Debug, Clone)]
pub struct ReviewHistory {
    /// 一意識別子
    pub id:               Uuid,
    /// ユーザー ID
    pub user_id:          Uuid,
    /// 学習項目 ID
    pub item_id:          Uuid,
    /// 復習日時
    pub reviewed_at:      DateTime<Utc>,
    /// 判定結果
    pub judgment:         i32,
    /// 応答時間（ミリ秒）
    pub response_time_ms: i32,
    /// 復習間隔（日数）
    pub interval_days:    i32,
    /// `EasyFactor`
    pub easiness_factor:  f32,
    /// セッション ID
    pub session_id:       Option<Uuid>,
    /// 作成日時
    pub created_at:       DateTime<Utc>,
}

/// Repository trait for review history
#[async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait ReviewHistoryRepository: Send + Sync {
    /// Create a new review history record
    async fn create(&self, history: &ReviewHistory) -> RepositoryResult<Uuid>;

    /// Get review history for a user and item
    async fn get_by_user_and_item(
        &self,
        user_id: Uuid,
        item_id: Uuid,
        limit: i64,
    ) -> RepositoryResult<Vec<ReviewHistory>>;

    /// Get recent reviews for a user
    async fn get_recent_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> RepositoryResult<Vec<ReviewHistory>>;

    /// Get reviews by session
    async fn get_by_session(&self, session_id: Uuid) -> RepositoryResult<Vec<ReviewHistory>>;

    /// Count reviews in a time period
    async fn count_in_period(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> RepositoryResult<i64>;
}

/// `PostgreSQL` implementation of `ReviewHistoryRepository`
pub struct PostgresReviewHistoryRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl PostgresReviewHistoryRepository {
    /// 新しい `PostgresReviewHistoryRepository` を作成
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReviewHistoryRepository for PostgresReviewHistoryRepository {
    async fn create(&self, history: &ReviewHistory) -> RepositoryResult<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO review_histories (
                id, user_id, item_id, reviewed_at, judgment,
                response_time_ms, interval_days, easiness_factor, session_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9
            )
            "#,
            id,
            history.user_id,
            history.item_id,
            history.reviewed_at,
            history.judgment,
            history.response_time_ms,
            history.interval_days,
            history.easiness_factor,
            history.session_id
        )
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    async fn get_by_user_and_item(
        &self,
        user_id: Uuid,
        item_id: Uuid,
        limit: i64,
    ) -> RepositoryResult<Vec<ReviewHistory>> {
        let histories = sqlx::query_as!(
            ReviewHistory,
            r#"
            SELECT 
                id, user_id, item_id, reviewed_at, judgment,
                response_time_ms, interval_days, easiness_factor,
                session_id, created_at
            FROM review_histories
            WHERE user_id = $1 AND item_id = $2
            ORDER BY reviewed_at DESC
            LIMIT $3
            "#,
            user_id,
            item_id,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(histories)
    }

    async fn get_recent_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> RepositoryResult<Vec<ReviewHistory>> {
        let histories = sqlx::query_as!(
            ReviewHistory,
            r#"
            SELECT 
                id, user_id, item_id, reviewed_at, judgment,
                response_time_ms, interval_days, easiness_factor,
                session_id, created_at
            FROM review_histories
            WHERE user_id = $1
            ORDER BY reviewed_at DESC
            LIMIT $2
            "#,
            user_id,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(histories)
    }

    async fn get_by_session(&self, session_id: Uuid) -> RepositoryResult<Vec<ReviewHistory>> {
        let histories = sqlx::query_as!(
            ReviewHistory,
            r#"
            SELECT 
                id, user_id, item_id, reviewed_at, judgment,
                response_time_ms, interval_days, easiness_factor,
                session_id, created_at
            FROM review_histories
            WHERE session_id = $1
            ORDER BY reviewed_at ASC
            "#,
            session_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(histories)
    }

    async fn count_in_period(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> RepositoryResult<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM review_histories
            WHERE user_id = $1 
                AND reviewed_at >= $2 
                AND reviewed_at <= $3
            "#,
            user_id,
            from,
            to
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }
}
