//! Event publisher for Algorithm Service

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

/// Event publisher error types
#[derive(Debug, Error)]
pub enum Error {
    /// イベント発行エラー
    #[error("Failed to publish event: {0}")]
    PublishError(String),

    /// シリアライズエラー
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for event publisher operations
pub type Result<T> = std::result::Result<T, Error>;

/// Event types for Algorithm Service
#[derive(Debug, Clone)]
pub enum AlgorithmEvent {
    /// Review schedule updated event
    ReviewScheduleUpdated {
        /// イベント ID
        event_id:         Uuid,
        /// ユーザー ID
        user_id:          Uuid,
        /// 学習項目 ID
        item_id:          Uuid,
        /// 次回復習日時
        next_review_date: DateTime<Utc>,
        /// 復習間隔（日数）
        interval_days:    u32,
        /// `EasyFactor`
        easiness_factor:  f32,
        /// 復習回数
        repetition_count: u32,
    },

    /// Difficulty adjusted event
    DifficultyAdjusted {
        /// イベント ID
        event_id:            Uuid,
        /// ユーザー ID
        user_id:             Uuid,
        /// 学習項目 ID
        item_id:             Uuid,
        /// 旧 `EasyFactor`
        old_easiness_factor: f32,
        /// 新 `EasyFactor`
        new_easiness_factor: f32,
        /// 品質評価
        quality_rating:      u32,
        /// 調整理由
        adjustment_reason:   String,
    },

    /// Performance analyzed event
    PerformanceAnalyzed {
        /// イベント ID
        event_id:           Uuid,
        /// ユーザー ID
        user_id:            Uuid,
        /// 正答率
        accuracy_rate:      f32,
        /// 定着率
        retention_rate:     f32,
        /// 学習速度
        learning_velocity:  f32,
        /// 分析タイムスタンプ
        analysis_timestamp: DateTime<Utc>,
    },

    /// Strategy adjusted event
    StrategyAdjusted {
        /// イベント ID
        event_id:          Uuid,
        /// ユーザー ID
        user_id:           Uuid,
        /// 旧戦略タイプ
        old_strategy_type: i32,
        /// 新戦略タイプ
        new_strategy_type: i32,
        /// 変更理由
        reason:            String,
    },

    /// Statistics updated event
    StatisticsUpdated {
        /// イベント ID
        event_id:       Uuid,
        /// ユーザー ID
        user_id:        Uuid,
        /// 総項目数
        total_items:    u32,
        /// 習得済み項目数
        mastered_items: u32,
        /// 習得率
        mastery_rate:   f32,
        /// 連続学習日数
        streak_days:    u32,
    },

    /// Item reviewed event
    ItemReviewed {
        /// イベント ID
        event_id:         Uuid,
        /// ユーザー ID
        user_id:          Uuid,
        /// 学習項目 ID
        item_id:          Uuid,
        /// 品質評価
        quality_rating:   u32,
        /// 応答時間（ミリ秒）
        response_time_ms: u32,
        /// 正解かどうか
        is_correct:       bool,
    },
}

/// Event publisher trait
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish an event
    async fn publish(&self, event: AlgorithmEvent) -> Result<()>;

    /// Publish multiple events in a batch
    async fn publish_batch(&self, events: Vec<AlgorithmEvent>) -> Result<()>;
}

/// Mock implementation for testing
pub struct MockPublisher {
    published_events: std::sync::Arc<tokio::sync::Mutex<Vec<AlgorithmEvent>>>,
}

impl Default for MockPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPublisher {
    /// 新しい `MockPublisher` を作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            published_events: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// 発行されたイベントを取得（テスト用）
    pub async fn get_published_events(&self) -> Vec<AlgorithmEvent> {
        self.published_events.lock().await.clone()
    }
}

#[async_trait]
impl EventPublisher for MockPublisher {
    async fn publish(&self, event: AlgorithmEvent) -> Result<()> {
        {
            let mut events = self.published_events.lock().await;
            events.push(event);
        }
        Ok(())
    }

    async fn publish_batch(&self, events: Vec<AlgorithmEvent>) -> Result<()> {
        {
            let mut published = self.published_events.lock().await;
            published.extend(events);
        }
        Ok(())
    }
}

/// Domain Events Service を使った実装
pub struct DomainEventsPublisher {
    // TODO: Domain Events Service のクライアントを追加
}

impl Default for DomainEventsPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainEventsPublisher {
    /// 新しい `DomainEventsPublisher` を作成
    #[must_use]
    pub const fn new() -> Self {
        Self {
            // TODO: クライアントを初期化
        }
    }
}

#[async_trait]
impl EventPublisher for DomainEventsPublisher {
    async fn publish(&self, event: AlgorithmEvent) -> Result<()> {
        // TODO: Domain Events Service に送信
        tracing::info!("Publishing event: {:?}", event);
        Ok(())
    }

    async fn publish_batch(&self, events: Vec<AlgorithmEvent>) -> Result<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
}
