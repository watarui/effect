//! アウトバウンドポート

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::{domain::*, error::Result};

/// イベントストア読み取りポート
#[async_trait]
pub trait EventStoreReader: Send + Sync {
    /// 指定位置からイベントを読み取り
    async fn read_events(&self, from_position: i64, batch_size: usize) -> Result<Vec<Event>>;

    /// 特定のストリームのイベントを読み取り
    async fn read_stream_events(&self, stream_id: &str, from_version: i64) -> Result<Vec<Event>>;
}

/// Read Model リポジトリポート
#[async_trait]
pub trait ReadModelRepository: Send + Sync {
    // ユーザー進捗
    async fn save_user_progress(&self, progress: &UserProgress) -> Result<()>;
    async fn get_user_progress(&self, user_id: Uuid) -> Result<Option<UserProgress>>;

    // 日次進捗
    async fn save_daily_progress(&self, progress: &DailyProgress) -> Result<()>;
    async fn get_daily_progress(
        &self,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<DailyProgress>>;

    // 週次進捗
    async fn save_weekly_progress(&self, progress: &WeeklyProgress) -> Result<()>;
    async fn get_weekly_progress(
        &self,
        user_id: Uuid,
        week_start: NaiveDate,
    ) -> Result<Option<WeeklyProgress>>;

    // 語彙アイテム進捗
    async fn save_vocabulary_item_progress(&self, progress: &VocabularyItemProgress) -> Result<()>;
    async fn get_vocabulary_item_progress(
        &self,
        user_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<VocabularyItemProgress>>;

    // アチーブメント
    async fn save_achievement(&self, achievement: &Achievement) -> Result<()>;
    async fn get_user_achievements(&self, user_id: Uuid) -> Result<Vec<Achievement>>;
}

/// プロジェクション状態ストアポート
#[async_trait]
pub trait ProjectionStateStore: Send + Sync {
    /// プロジェクション状態を保存
    async fn save_state(&self, state: &ProjectionState) -> Result<()>;

    /// プロジェクション状態を取得
    async fn get_state(&self, projection_name: &str) -> Result<Option<ProjectionState>>;

    /// すべてのプロジェクション状態を取得
    async fn get_all_states(&self) -> Result<Vec<ProjectionState>>;
}

/// イベント
#[derive(Debug, Clone)]
pub struct Event {
    pub event_id:      Uuid,
    pub stream_id:     String,
    pub event_type:    String,
    pub event_data:    serde_json::Value,
    pub event_version: i64,
    pub position:      i64,
    pub occurred_at:   DateTime<Utc>,
}
