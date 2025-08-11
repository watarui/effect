//! Mock implementations for testing

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{
    RepositoryResult,
    learning_item::{ItemCounts, LearningItemRepository, LearningItemState},
    review_history::{ReviewHistory, ReviewHistoryRepository},
    statistics::{PerformanceAnalysis, StatisticsRepository, UserLearningStatistics},
    strategy::{LearningStrategy, LearningStrategyRepository},
};

/// Mock implementation of `LearningItemRepository`
#[allow(clippy::module_name_repetitions)]
pub struct MockRepository {
    items: Arc<RwLock<HashMap<(Uuid, Uuid), LearningItemState>>>,
}

impl Default for MockRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MockRepository {
    /// 新しい `MockRepository` を作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl LearningItemRepository for MockRepository {
    async fn find_by_user_and_item(
        &self,
        user_id: Uuid,
        item_id: Uuid,
    ) -> RepositoryResult<Option<LearningItemState>> {
        let items = self.items.read().await;
        Ok(items.get(&(user_id, item_id)).cloned())
    }

    async fn create(&self, state: &LearningItemState) -> RepositoryResult<Uuid> {
        let id = Uuid::new_v4();
        let mut new_state = state.clone();
        new_state.id = id;
        {
            let mut items = self.items.write().await;
            items.insert((state.user_id, state.item_id), new_state);
        }
        Ok(id)
    }

    async fn update(&self, state: &LearningItemState) -> RepositoryResult<()> {
        {
            let mut items = self.items.write().await;
            items.insert((state.user_id, state.item_id), state.clone());
        }
        Ok(())
    }

    async fn get_due_items(
        &self,
        user_id: Uuid,
        as_of: DateTime<Utc>,
        limit: i64,
    ) -> RepositoryResult<Vec<LearningItemState>> {
        let mut due_items: Vec<_> = {
            let items = self.items.read().await;
            items
                .values()
                .filter(|item| {
                    item.user_id == user_id
                        && item.next_review_date.is_some_and(|date| date <= as_of)
                })
                .take(limit.try_into().unwrap_or(usize::MAX))
                .cloned()
                .collect()
        };

        due_items.sort_by(|a, b| a.next_review_date.cmp(&b.next_review_date));
        Ok(due_items)
    }

    async fn get_by_mastery_level(
        &self,
        user_id: Uuid,
        mastery_level: i32,
    ) -> RepositoryResult<Vec<LearningItemState>> {
        let items = self.items.read().await;
        Ok(items
            .values()
            .filter(|item| item.user_id == user_id && item.mastery_level == mastery_level)
            .cloned()
            .collect())
    }

    async fn count_by_user(&self, user_id: Uuid) -> RepositoryResult<ItemCounts> {
        let items = self.items.read().await;
        let user_items: Vec<_> = items
            .values()
            .filter(|item| item.user_id == user_id)
            .collect();

        let result = ItemCounts {
            total:    i64::try_from(user_items.len()).unwrap_or(i64::MAX),
            mastered: i64::try_from(user_items.iter().filter(|i| i.mastery_level == 5).count())
                .unwrap_or(i64::MAX),
            learning: i64::try_from(
                user_items
                    .iter()
                    .filter(|i| (2..=4).contains(&i.mastery_level))
                    .count(),
            )
            .unwrap_or(i64::MAX),
            new:      i64::try_from(user_items.iter().filter(|i| i.mastery_level == 1).count())
                .unwrap_or(i64::MAX),
        };
        drop(items);
        Ok(result)
    }
}

/// Mock implementation of `ReviewHistoryRepository`
#[allow(clippy::module_name_repetitions)]
pub struct MockHistoryRepository {
    histories: Arc<RwLock<Vec<ReviewHistory>>>,
}

impl Default for MockHistoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MockHistoryRepository {
    /// 新しい `MockHistoryRepository` を作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            histories: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl ReviewHistoryRepository for MockHistoryRepository {
    async fn create(&self, history: &ReviewHistory) -> RepositoryResult<Uuid> {
        let id = Uuid::new_v4();
        let mut new_history = history.clone();
        new_history.id = id;
        {
            let mut histories = self.histories.write().await;
            histories.push(new_history);
        }
        Ok(id)
    }

    async fn get_by_user_and_item(
        &self,
        user_id: Uuid,
        item_id: Uuid,
        limit: i64,
    ) -> RepositoryResult<Vec<ReviewHistory>> {
        let mut result: Vec<_> = {
            let histories = self.histories.read().await;
            histories
                .iter()
                .filter(|h| h.user_id == user_id && h.item_id == item_id)
                .take(limit.try_into().unwrap_or(usize::MAX))
                .cloned()
                .collect()
        };

        result.sort_by(|a, b| b.reviewed_at.cmp(&a.reviewed_at));
        Ok(result)
    }

    async fn get_recent_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> RepositoryResult<Vec<ReviewHistory>> {
        let mut result: Vec<_> = {
            let histories = self.histories.read().await;
            histories
                .iter()
                .filter(|h| h.user_id == user_id)
                .cloned()
                .collect()
        };

        result.sort_by(|a, b| b.reviewed_at.cmp(&a.reviewed_at));
        result.truncate(limit.try_into().unwrap_or(usize::MAX));
        Ok(result)
    }

    async fn get_by_session(&self, session_id: Uuid) -> RepositoryResult<Vec<ReviewHistory>> {
        let mut result: Vec<_> = {
            let histories = self.histories.read().await;
            histories
                .iter()
                .filter(|h| h.session_id == Some(session_id))
                .cloned()
                .collect()
        };

        result.sort_by(|a, b| a.reviewed_at.cmp(&b.reviewed_at));
        Ok(result)
    }

    async fn count_in_period(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> RepositoryResult<i64> {
        let count = {
            let histories = self.histories.read().await;
            histories
                .iter()
                .filter(|h| h.user_id == user_id && h.reviewed_at >= from && h.reviewed_at <= to)
                .count()
        };

        Ok(i64::try_from(count).unwrap_or(i64::MAX))
    }
}

/// Mock implementation of `LearningStrategyRepository`
#[allow(clippy::module_name_repetitions)]
pub struct MockStrategyRepository {
    strategies: Arc<RwLock<HashMap<Uuid, LearningStrategy>>>,
}

impl Default for MockStrategyRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MockStrategyRepository {
    /// 新しい `MockStrategyRepository` を作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl LearningStrategyRepository for MockStrategyRepository {
    async fn find_by_user(&self, user_id: Uuid) -> RepositoryResult<Option<LearningStrategy>> {
        let strategies = self.strategies.read().await;
        Ok(strategies.get(&user_id).cloned())
    }

    async fn create(&self, strategy: &LearningStrategy) -> RepositoryResult<Uuid> {
        let id = Uuid::new_v4();
        let mut new_strategy = strategy.clone();
        new_strategy.id = id;
        {
            let mut strategies = self.strategies.write().await;
            strategies.insert(strategy.user_id, new_strategy);
        }
        Ok(id)
    }

    async fn update(&self, strategy: &LearningStrategy) -> RepositoryResult<()> {
        {
            let mut strategies = self.strategies.write().await;
            strategies.insert(strategy.user_id, strategy.clone());
        }
        Ok(())
    }

    async fn get_or_create_default(&self, user_id: Uuid) -> RepositoryResult<LearningStrategy> {
        let strategies = self.strategies.read().await;
        if let Some(strategy) = strategies.get(&user_id) {
            return Ok(strategy.clone());
        }
        drop(strategies);

        let mut strategy = LearningStrategy {
            user_id,
            ..Default::default()
        };
        let id = self.create(&strategy).await?;
        strategy.id = id;
        Ok(strategy)
    }
}

/// Mock implementation of `StatisticsRepository`
#[allow(clippy::module_name_repetitions)]
pub struct MockStatsRepository {
    user_stats: Arc<RwLock<HashMap<Uuid, UserLearningStatistics>>>,
    analyses:   Arc<RwLock<Vec<PerformanceAnalysis>>>,
}

impl Default for MockStatsRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MockStatsRepository {
    /// 新しい `MockStatsRepository` を作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            user_stats: Arc::new(RwLock::new(HashMap::new())),
            analyses:   Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl StatisticsRepository for MockStatsRepository {
    async fn get_user_statistics(
        &self,
        user_id: Uuid,
    ) -> RepositoryResult<Option<UserLearningStatistics>> {
        let stats = self.user_stats.read().await;
        Ok(stats.get(&user_id).cloned())
    }

    async fn update_user_statistics(&self, stats: &UserLearningStatistics) -> RepositoryResult<()> {
        {
            let mut user_stats = self.user_stats.write().await;
            user_stats.insert(stats.user_id, stats.clone());
        }
        Ok(())
    }

    async fn upsert_user_statistics(&self, stats: &UserLearningStatistics) -> RepositoryResult<()> {
        self.update_user_statistics(stats).await
    }

    async fn save_performance_analysis(
        &self,
        analysis: &PerformanceAnalysis,
    ) -> RepositoryResult<Uuid> {
        let id = Uuid::new_v4();
        let mut new_analysis = analysis.clone();
        new_analysis.id = id;
        {
            let mut stored_analyses = self.analyses.write().await;
            stored_analyses.push(new_analysis);
        }
        Ok(id)
    }

    async fn get_latest_performance_analysis(
        &self,
        user_id: Uuid,
    ) -> RepositoryResult<Option<PerformanceAnalysis>> {
        let analyses = self.analyses.read().await;
        Ok(analyses
            .iter()
            .filter(|a| a.user_id == user_id)
            .max_by_key(|a| a.created_at)
            .cloned())
    }

    async fn get_performance_analyses_in_period(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> RepositoryResult<Vec<PerformanceAnalysis>> {
        let analyses = self.analyses.read().await;
        Ok(analyses
            .iter()
            .filter(|a| a.user_id == user_id && a.analyzed_from >= from && a.analyzed_to <= to)
            .cloned()
            .collect())
    }
}
