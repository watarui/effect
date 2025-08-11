//! Repository for learning item states

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{RepositoryError, RepositoryResult};

/// Learning item state entity
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct LearningItemState {
    /// 一意識別子
    pub id: Uuid,
    /// ユーザー ID
    pub user_id: Uuid,
    /// 学習項目 ID
    pub item_id: Uuid,
    /// `EasyFactor` (難易度係数)
    pub easiness_factor: f32,
    /// 復習回数
    pub repetition_number: i32,
    /// 復習間隔（日数）
    pub interval_days: i32,
    /// 習熟レベル
    pub mastery_level: i32,
    /// 定着率
    pub retention_rate: f32,
    /// 次回復習日時
    pub next_review_date: Option<DateTime<Utc>>,
    /// 最終復習日時
    pub last_reviewed_at: Option<DateTime<Utc>>,
    /// 総復習回数
    pub total_reviews: i32,
    /// 正解回数
    pub correct_count: i32,
    /// 不正解回数
    pub incorrect_count: i32,
    /// 平均応答時間（ミリ秒）
    pub average_response_time_ms: f32,
    /// 難易度レベル
    pub difficulty_level: i32,
    /// 問題のある項目かどうか
    pub is_problematic: bool,
    /// 楽観的ロック用バージョン
    pub version: i64,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
}

/// Repository trait for learning item states
#[async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait LearningItemRepository: Send + Sync {
    /// Find a learning item state by user and item
    async fn find_by_user_and_item(
        &self,
        user_id: Uuid,
        item_id: Uuid,
    ) -> RepositoryResult<Option<LearningItemState>>;

    /// Create a new learning item state
    async fn create(&self, state: &LearningItemState) -> RepositoryResult<Uuid>;

    /// Update an existing learning item state
    async fn update(&self, state: &LearningItemState) -> RepositoryResult<()>;

    /// Get due items for a user
    async fn get_due_items(
        &self,
        user_id: Uuid,
        as_of: DateTime<Utc>,
        limit: i64,
    ) -> RepositoryResult<Vec<LearningItemState>>;

    /// Get items by mastery level
    async fn get_by_mastery_level(
        &self,
        user_id: Uuid,
        mastery_level: i32,
    ) -> RepositoryResult<Vec<LearningItemState>>;

    /// Count items by user
    async fn count_by_user(&self, user_id: Uuid) -> RepositoryResult<ItemCounts>;

    /// Find all learning items for a user
    async fn find_by_user(&self, user_id: Uuid) -> RepositoryResult<Vec<LearningItemState>>;
}

/// Item counts by category
#[derive(Debug, Clone)]
pub struct ItemCounts {
    /// 総項目数
    pub total:    i32,
    /// 習得済み項目数
    pub mastered: i32,
    /// 学習中項目数
    pub learning: i32,
    /// 新規項目数
    pub new:      i32,
}

/// `PostgreSQL` implementation of `LearningItemRepository`
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
impl LearningItemRepository for PostgresRepository {
    async fn find_by_user_and_item(
        &self,
        user_id: Uuid,
        item_id: Uuid,
    ) -> RepositoryResult<Option<LearningItemState>> {
        let result = sqlx::query_as!(
            LearningItemState,
            r#"
            SELECT 
                id, user_id, item_id, easiness_factor, repetition_number,
                interval_days, mastery_level, retention_rate, next_review_date,
                last_reviewed_at, total_reviews, correct_count, incorrect_count,
                average_response_time_ms, difficulty_level, is_problematic,
                version, created_at, updated_at
            FROM learning_item_states
            WHERE user_id = $1 AND item_id = $2
            "#,
            user_id,
            item_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn create(&self, state: &LearningItemState) -> RepositoryResult<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO learning_item_states (
                id, user_id, item_id, easiness_factor, repetition_number,
                interval_days, mastery_level, retention_rate, next_review_date,
                last_reviewed_at, total_reviews, correct_count, incorrect_count,
                average_response_time_ms, difficulty_level, is_problematic
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
            )
            "#,
            id,
            state.user_id,
            state.item_id,
            state.easiness_factor,
            state.repetition_number,
            state.interval_days,
            state.mastery_level,
            state.retention_rate,
            state.next_review_date,
            state.last_reviewed_at,
            state.total_reviews,
            state.correct_count,
            state.incorrect_count,
            state.average_response_time_ms,
            state.difficulty_level,
            state.is_problematic
        )
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    async fn update(&self, state: &LearningItemState) -> RepositoryResult<()> {
        let result = sqlx::query!(
            r#"
            UPDATE learning_item_states SET
                easiness_factor = $3,
                repetition_number = $4,
                interval_days = $5,
                mastery_level = $6,
                retention_rate = $7,
                next_review_date = $8,
                last_reviewed_at = $9,
                total_reviews = $10,
                correct_count = $11,
                incorrect_count = $12,
                average_response_time_ms = $13,
                difficulty_level = $14,
                is_problematic = $15,
                version = version + 1,
                updated_at = NOW()
            WHERE user_id = $1 AND item_id = $2 AND version = $16
            "#,
            state.user_id,
            state.item_id,
            state.easiness_factor,
            state.repetition_number,
            state.interval_days,
            state.mastery_level,
            state.retention_rate,
            state.next_review_date,
            state.last_reviewed_at,
            state.total_reviews,
            state.correct_count,
            state.incorrect_count,
            state.average_response_time_ms,
            state.difficulty_level,
            state.is_problematic,
            state.version
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Conflict(
                "Optimistic lock conflict".to_string(),
            ));
        }

        Ok(())
    }

    async fn get_due_items(
        &self,
        user_id: Uuid,
        as_of: DateTime<Utc>,
        limit: i64,
    ) -> RepositoryResult<Vec<LearningItemState>> {
        let items = sqlx::query_as!(
            LearningItemState,
            r#"
            SELECT 
                id, user_id, item_id, easiness_factor, repetition_number,
                interval_days, mastery_level, retention_rate, next_review_date,
                last_reviewed_at, total_reviews, correct_count, incorrect_count,
                average_response_time_ms, difficulty_level, is_problematic,
                version, created_at, updated_at
            FROM learning_item_states
            WHERE user_id = $1 
                AND next_review_date IS NOT NULL
                AND next_review_date <= $2
            ORDER BY next_review_date ASC
            LIMIT $3
            "#,
            user_id,
            as_of,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    async fn get_by_mastery_level(
        &self,
        user_id: Uuid,
        mastery_level: i32,
    ) -> RepositoryResult<Vec<LearningItemState>> {
        let items = sqlx::query_as!(
            LearningItemState,
            r#"
            SELECT 
                id, user_id, item_id, easiness_factor, repetition_number,
                interval_days, mastery_level, retention_rate, next_review_date,
                last_reviewed_at, total_reviews, correct_count, incorrect_count,
                average_response_time_ms, difficulty_level, is_problematic,
                version, created_at, updated_at
            FROM learning_item_states
            WHERE user_id = $1 AND mastery_level = $2
            ORDER BY last_reviewed_at DESC
            "#,
            user_id,
            mastery_level
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    async fn count_by_user(&self, user_id: Uuid) -> RepositoryResult<ItemCounts> {
        let row = sqlx::query!(
            r#"
            SELECT 
                COUNT(*)::INTEGER as "total!",
                COUNT(*) FILTER (WHERE mastery_level = 5)::INTEGER as "mastered!",
                COUNT(*) FILTER (WHERE mastery_level BETWEEN 2 AND 4)::INTEGER as "learning!",
                COUNT(*) FILTER (WHERE mastery_level = 1)::INTEGER as "new!"
            FROM learning_item_states
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ItemCounts {
            total:    row.total,
            mastered: row.mastered,
            learning: row.learning,
            new:      row.new,
        })
    }

    async fn find_by_user(&self, user_id: Uuid) -> RepositoryResult<Vec<LearningItemState>> {
        let items = sqlx::query_as!(
            LearningItemState,
            r#"
            SELECT 
                id, user_id, item_id, easiness_factor, repetition_number,
                interval_days, mastery_level, retention_rate, next_review_date,
                last_reviewed_at, total_reviews, correct_count, incorrect_count,
                average_response_time_ms, difficulty_level, is_problematic,
                version, created_at, updated_at
            FROM learning_item_states
            WHERE user_id = $1
            ORDER BY item_id
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }
}
