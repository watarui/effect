//! Event publisher for Algorithm Service

use std::time::SystemTime;

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
    client: crate::proto::effect::event_store::event_store_service_client::EventStoreServiceClient<
        tonic::transport::Channel,
    >,
}

impl DomainEventsPublisher {
    /// 新しい `DomainEventsPublisher` を作成
    ///
    /// # Errors
    ///
    /// Event Store Service への接続に失敗した場合、`tonic::transport::Error`
    /// を返します
    pub async fn new(
        event_store_url: String,
    ) -> std::result::Result<Self, tonic::transport::Error> {
        let client = crate::proto::effect::event_store::event_store_service_client::EventStoreServiceClient::connect(event_store_url).await?;
        Ok(Self { client })
    }

    /// `AlgorithmEvent` を protobuf イベントに変換
    #[allow(clippy::unused_self)]
    #[allow(clippy::too_many_lines)]
    fn convert_to_proto_event(
        &self,
        event: &AlgorithmEvent,
    ) -> crate::proto::effect::event_store::Event {
        use prost::Message;

        use crate::proto::effect::events::algorithm as proto_events;

        let (event_type, event_data) = match event {
            AlgorithmEvent::ReviewScheduleUpdated {
                event_id,
                user_id,
                item_id,
                next_review_date,
                interval_days,
                easiness_factor,
                repetition_count,
            } => {
                let proto_event = proto_events::ReviewScheduleUpdated {
                    metadata:         Some(crate::proto::effect::common::EventMetadata {
                        event_id:          event_id.to_string(),
                        aggregate_id:      user_id.to_string(),
                        occurred_at:       Some(prost_types::Timestamp::from(SystemTime::now())),
                        version:           1,
                        caused_by_user_id: None,
                        correlation_id:    None,
                        causation_id:      None,
                        trace_context:     None,
                        command_id:        None,
                        source:            Some("algorithm_service".to_string()),
                        schema_version:    Some(1),
                    }),
                    user_id:          user_id.to_string(),
                    item_id:          item_id.to_string(),
                    next_review_date: Some({
                        let system_time: SystemTime = (*next_review_date).into();
                        prost_types::Timestamp::from(system_time)
                    }),
                    interval_days:    *interval_days,
                    easiness_factor:  *easiness_factor,
                    repetition_count: *repetition_count,
                };
                ("ReviewScheduleUpdated", proto_event.encode_to_vec())
            },
            AlgorithmEvent::DifficultyAdjusted {
                event_id,
                user_id,
                item_id,
                old_easiness_factor,
                new_easiness_factor,
                quality_rating,
                adjustment_reason,
            } => {
                let proto_event = proto_events::DifficultyAdjusted {
                    metadata:            Some(crate::proto::effect::common::EventMetadata {
                        event_id:          event_id.to_string(),
                        aggregate_id:      user_id.to_string(),
                        occurred_at:       Some(prost_types::Timestamp::from(SystemTime::now())),
                        version:           1,
                        caused_by_user_id: None,
                        correlation_id:    None,
                        causation_id:      None,
                        trace_context:     None,
                        command_id:        None,
                        source:            Some("algorithm_service".to_string()),
                        schema_version:    Some(1),
                    }),
                    user_id:             user_id.to_string(),
                    item_id:             item_id.to_string(),
                    old_easiness_factor: *old_easiness_factor,
                    new_easiness_factor: *new_easiness_factor,
                    quality_rating:      *quality_rating,
                    adjustment_reason:   adjustment_reason.clone(),
                };
                ("DifficultyAdjusted", proto_event.encode_to_vec())
            },
            AlgorithmEvent::PerformanceAnalyzed {
                event_id,
                user_id,
                accuracy_rate,
                retention_rate: _,
                learning_velocity,
                analysis_timestamp,
            } => {
                let proto_event = proto_events::PerformanceAnalyzed {
                    metadata:    Some(crate::proto::effect::common::EventMetadata {
                        event_id:          event_id.to_string(),
                        aggregate_id:      user_id.to_string(),
                        occurred_at:       Some({
                            let system_time: SystemTime = (*analysis_timestamp).into();
                            prost_types::Timestamp::from(system_time)
                        }),
                        version:           1,
                        caused_by_user_id: None,
                        correlation_id:    None,
                        causation_id:      None,
                        trace_context:     None,
                        command_id:        None,
                        source:            Some("algorithm_service".to_string()),
                        schema_version:    Some(1),
                    }),
                    user_id:     user_id.to_string(),
                    performance: Some(proto_events::LearningPerformance {
                        recent_accuracy:    *accuracy_rate,
                        average_quality:    0.0, // TODO: 実際の値を計算
                        session_count:      0,   // TODO: 実際の値を取得
                        consistency_score:  0.0, // TODO: 実際の値を計算
                        optimal_difficulty: *learning_velocity,
                    }),
                };
                ("PerformanceAnalyzed", proto_event.encode_to_vec())
            },
            AlgorithmEvent::StrategyAdjusted {
                event_id,
                user_id,
                old_strategy_type: _,
                new_strategy_type: _,
                reason,
            } => {
                let proto_event = proto_events::StrategyAdjusted {
                    metadata:   Some(crate::proto::effect::common::EventMetadata {
                        event_id:          event_id.to_string(),
                        aggregate_id:      user_id.to_string(),
                        occurred_at:       Some(prost_types::Timestamp::from(SystemTime::now())),
                        version:           1,
                        caused_by_user_id: None,
                        correlation_id:    None,
                        causation_id:      None,
                        trace_context:     None,
                        command_id:        None,
                        source:            Some("algorithm_service".to_string()),
                        schema_version:    Some(1),
                    }),
                    user_id:    user_id.to_string(),
                    old_ratios: None, // TODO: 実際の戦略比率を設定
                    new_ratios: None, // TODO: 実際の戦略比率を設定
                    reason:     reason.clone(),
                };
                ("StrategyAdjusted", proto_event.encode_to_vec())
            },
            AlgorithmEvent::StatisticsUpdated {
                event_id,
                user_id,
                total_items,
                mastered_items,
                mastery_rate,
                streak_days,
            } => {
                let proto_event = proto_events::StatisticsUpdated {
                    metadata:       Some(crate::proto::effect::common::EventMetadata {
                        event_id:          event_id.to_string(),
                        aggregate_id:      user_id.to_string(),
                        occurred_at:       Some(prost_types::Timestamp::from(SystemTime::now())),
                        version:           1,
                        caused_by_user_id: None,
                        correlation_id:    None,
                        causation_id:      None,
                        trace_context:     None,
                        command_id:        None,
                        source:            Some("algorithm_service".to_string()),
                        schema_version:    Some(1),
                    }),
                    user_id:        user_id.to_string(),
                    total_items:    *total_items,
                    mastered_items: *mastered_items,
                    mastery_rate:   *mastery_rate,
                    streak_days:    *streak_days,
                };
                ("StatisticsUpdated", proto_event.encode_to_vec())
            },
            AlgorithmEvent::ItemReviewed {
                event_id,
                user_id,
                item_id,
                quality_rating,
                response_time_ms,
                is_correct,
            } => {
                let proto_event = proto_events::ItemReviewed {
                    metadata:         Some(crate::proto::effect::common::EventMetadata {
                        event_id:          event_id.to_string(),
                        aggregate_id:      user_id.to_string(),
                        occurred_at:       Some(prost_types::Timestamp::from(SystemTime::now())),
                        version:           1,
                        caused_by_user_id: None,
                        correlation_id:    None,
                        causation_id:      None,
                        trace_context:     None,
                        command_id:        None,
                        source:            Some("algorithm_service".to_string()),
                        schema_version:    Some(1),
                    }),
                    user_id:          user_id.to_string(),
                    item_id:          item_id.to_string(),
                    quality_rating:   *quality_rating,
                    response_time_ms: *response_time_ms,
                    is_correct:       *is_correct,
                };
                ("ItemReviewed", proto_event.encode_to_vec())
            },
        };

        crate::proto::effect::event_store::Event {
            event_type: event_type.to_string(),
            data:       Some(prost_types::Any {
                type_url: format!("type.googleapis.com/effect.events.algorithm.{event_type}"),
                value:    event_data,
            }),
            metadata:   std::collections::HashMap::new(),
        }
    }
}

#[async_trait]
impl EventPublisher for DomainEventsPublisher {
    async fn publish(&self, event: AlgorithmEvent) -> Result<()> {
        use crate::proto::effect::event_store::AppendEventsRequest;

        // イベントのストリーム ID を決定
        let (stream_id, stream_type) = match &event {
            AlgorithmEvent::ReviewScheduleUpdated { user_id, .. }
            | AlgorithmEvent::DifficultyAdjusted { user_id, .. }
            | AlgorithmEvent::PerformanceAnalyzed { user_id, .. }
            | AlgorithmEvent::StrategyAdjusted { user_id, .. }
            | AlgorithmEvent::StatisticsUpdated { user_id, .. }
            | AlgorithmEvent::ItemReviewed { user_id, .. } => {
                (user_id.to_string(), "UserLearning".to_string())
            },
        };

        let proto_event = self.convert_to_proto_event(&event);

        let request = AppendEventsRequest {
            stream_id,
            stream_type,
            events: vec![proto_event],
            expected_version: -1, // 任意のバージョンを許可
        };

        let mut client = self.client.clone();
        client
            .append_events(request)
            .await
            .map_err(|e| Error::PublishError(e.to_string()))?;

        tracing::info!("Published event: {:?}", event);
        Ok(())
    }

    async fn publish_batch(&self, events: Vec<AlgorithmEvent>) -> Result<()> {
        // イベントをストリームごとにグループ化
        let mut stream_events: std::collections::HashMap<
            String,
            Vec<crate::proto::effect::event_store::Event>,
        > = std::collections::HashMap::new();

        for event in &events {
            let stream_id = match event {
                AlgorithmEvent::ReviewScheduleUpdated { user_id, .. }
                | AlgorithmEvent::DifficultyAdjusted { user_id, .. }
                | AlgorithmEvent::PerformanceAnalyzed { user_id, .. }
                | AlgorithmEvent::StrategyAdjusted { user_id, .. }
                | AlgorithmEvent::StatisticsUpdated { user_id, .. }
                | AlgorithmEvent::ItemReviewed { user_id, .. } => user_id.to_string(),
            };

            let proto_event = self.convert_to_proto_event(event);
            stream_events
                .entry(stream_id)
                .or_default()
                .push(proto_event);
        }

        // 各ストリームにイベントを送信
        for (stream_id, proto_events) in stream_events {
            use crate::proto::effect::event_store::AppendEventsRequest;

            let request = AppendEventsRequest {
                stream_id,
                stream_type: "UserLearning".to_string(),
                events: proto_events,
                expected_version: -1,
            };

            let mut client = self.client.clone();
            client
                .append_events(request)
                .await
                .map_err(|e| Error::PublishError(e.to_string()))?;
        }

        tracing::info!("Published {} events in batch", events.len());
        Ok(())
    }
}
