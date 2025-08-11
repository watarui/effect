//! Repository for learning strategies

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{RepositoryError, RepositoryResult};

/// Learning strategy entity
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct LearningStrategy {
    /// 一意識別子
    pub id:                    Uuid,
    /// ユーザー ID
    pub user_id:               Uuid,
    /// 戦略タイプ（1: 高速, 2: バランス, 3: 深い理解）
    pub strategy_type:         i32,
    /// 1日の目標項目数
    pub daily_target_items:    i32,
    /// 1日の新規項目数
    pub new_items_per_day:     i32,
    /// 難易度閾値
    pub difficulty_threshold:  f32,
    /// 学習速度係数
    pub learning_speed_factor: f32,
    /// 定着優先度
    pub retention_priority:    f32,
    /// 適応的スケジューリング有効/無効
    pub adaptive_scheduling:   bool,
    /// バージョン（楽観的ロック用）
    pub version:               i64,
    /// 最終調整日時
    pub last_adjusted_at:      Option<DateTime<Utc>>,
    /// 作成日時
    pub created_at:            DateTime<Utc>,
    /// 更新日時
    pub updated_at:            DateTime<Utc>,
}

impl Default for LearningStrategy {
    fn default() -> Self {
        Self {
            id:                    Uuid::new_v4(),
            user_id:               Uuid::nil(),
            strategy_type:         2, // Balanced
            daily_target_items:    20,
            new_items_per_day:     5,
            difficulty_threshold:  0.7,
            learning_speed_factor: 1.0,
            retention_priority:    0.8,
            adaptive_scheduling:   true,
            version:               1,
            last_adjusted_at:      None,
            created_at:            Utc::now(),
            updated_at:            Utc::now(),
        }
    }
}

/// Repository trait for learning strategies
#[async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait LearningStrategyRepository: Send + Sync {
    /// Find a strategy by user ID
    async fn find_by_user(&self, user_id: Uuid) -> RepositoryResult<Option<LearningStrategy>>;

    /// Create a new strategy
    async fn create(&self, strategy: &LearningStrategy) -> RepositoryResult<Uuid>;

    /// Update an existing strategy
    async fn update(&self, strategy: &LearningStrategy) -> RepositoryResult<()>;

    /// Get or create default strategy for user
    async fn get_or_create_default(&self, user_id: Uuid) -> RepositoryResult<LearningStrategy>;
}

/// `PostgreSQL` implementation of `LearningStrategyRepository`
pub struct PostgresLearningStrategyRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl PostgresLearningStrategyRepository {
    /// 新しい `PostgresLearningStrategyRepository` を作成
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LearningStrategyRepository for PostgresLearningStrategyRepository {
    async fn find_by_user(&self, user_id: Uuid) -> RepositoryResult<Option<LearningStrategy>> {
        let result = sqlx::query_as!(
            LearningStrategy,
            r#"
            SELECT 
                id, user_id, strategy_type, daily_target_items,
                new_items_per_day, difficulty_threshold, learning_speed_factor,
                retention_priority, adaptive_scheduling, version,
                last_adjusted_at, created_at, updated_at
            FROM learning_strategies
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn create(&self, strategy: &LearningStrategy) -> RepositoryResult<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO learning_strategies (
                id, user_id, strategy_type, daily_target_items,
                new_items_per_day, difficulty_threshold, learning_speed_factor,
                retention_priority, adaptive_scheduling, last_adjusted_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
            )
            "#,
            id,
            strategy.user_id,
            strategy.strategy_type,
            strategy.daily_target_items,
            strategy.new_items_per_day,
            strategy.difficulty_threshold,
            strategy.learning_speed_factor,
            strategy.retention_priority,
            strategy.adaptive_scheduling,
            strategy.last_adjusted_at
        )
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    async fn update(&self, strategy: &LearningStrategy) -> RepositoryResult<()> {
        let result = sqlx::query!(
            r#"
            UPDATE learning_strategies SET
                strategy_type = $3,
                daily_target_items = $4,
                new_items_per_day = $5,
                difficulty_threshold = $6,
                learning_speed_factor = $7,
                retention_priority = $8,
                adaptive_scheduling = $9,
                last_adjusted_at = $10,
                version = version + 1,
                updated_at = NOW()
            WHERE user_id = $1 AND id = $2 AND version = $11
            "#,
            strategy.user_id,
            strategy.id,
            strategy.strategy_type,
            strategy.daily_target_items,
            strategy.new_items_per_day,
            strategy.difficulty_threshold,
            strategy.learning_speed_factor,
            strategy.retention_priority,
            strategy.adaptive_scheduling,
            Utc::now(),
            strategy.version
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

    async fn get_or_create_default(&self, user_id: Uuid) -> RepositoryResult<LearningStrategy> {
        // まず既存のものを探す
        if let Some(strategy) = self.find_by_user(user_id).await? {
            return Ok(strategy);
        }

        // なければデフォルトを作成
        let mut strategy = LearningStrategy {
            user_id,
            ..Default::default()
        };
        let id = self.create(&strategy).await?;
        strategy.id = id;

        Ok(strategy)
    }
}
