//! コマンドハンドラー実装

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    domain::{Progress, commands::ProgressCommand, events::ProgressEvent},
    error::Result,
    ports::{inbound::CommandHandler, outbound::*},
};

/// コマンドハンドラー実装
pub struct ProgressCommandHandler {
    event_store:     Arc<dyn EventStorePort>,
    snapshot_store:  Arc<dyn SnapshotStorePort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl ProgressCommandHandler {
    pub fn new(
        event_store: Arc<dyn EventStorePort>,
        snapshot_store: Arc<dyn SnapshotStorePort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            event_store,
            snapshot_store,
            event_publisher,
        }
    }

    async fn load_aggregate(&self, user_id: Uuid) -> Result<Progress> {
        let stream_id = format!("progress-{}", user_id);

        // スナップショットから読み込み
        let mut progress =
            if let Some(snapshot) = self.snapshot_store.get_latest_snapshot(user_id).await? {
                snapshot
            } else {
                Progress::new(user_id)
            };

        // スナップショット以降のイベントを適用
        let events = if progress.version > 0 {
            self.event_store
                .get_events_from(stream_id, progress.version)
                .await?
        } else {
            self.event_store.get_events(stream_id).await?
        };

        for event in events {
            progress.apply(&event);
        }

        Ok(progress)
    }

    async fn save_event(&self, user_id: Uuid, event: ProgressEvent) -> Result<()> {
        let stream_id = format!("progress-{}", user_id);

        // イベントを保存
        self.event_store
            .save_event(stream_id, event.clone())
            .await?;

        // イベントを発行
        self.event_publisher.publish(event).await?;

        Ok(())
    }
}

#[async_trait]
impl CommandHandler for ProgressCommandHandler {
    async fn handle(&self, command: ProgressCommand) -> Result<()> {
        let event = match command {
            ProgressCommand::StartLearning { user_id } => ProgressEvent::LearningStarted {
                user_id,
                timestamp: Utc::now(),
            },
            ProgressCommand::RecordItemCompletion {
                user_id,
                vocabulary_item_id,
                accuracy,
                time_spent,
            } => ProgressEvent::ItemCompleted {
                user_id,
                vocabulary_item_id,
                accuracy,
                time_spent,
                timestamp: Utc::now(),
            },
            ProgressCommand::RecordSessionCompletion {
                user_id,
                items_count,
                correct_count,
                duration,
            } => ProgressEvent::SessionCompleted {
                user_id,
                items_count,
                correct_count,
                duration,
                timestamp: Utc::now(),
            },
            ProgressCommand::UpdateStreak {
                user_id,
                new_streak,
            } => ProgressEvent::StreakUpdated {
                user_id,
                new_streak,
                timestamp: Utc::now(),
            },
            ProgressCommand::UnlockAchievement {
                user_id,
                achievement_id,
            } => ProgressEvent::AchievementUnlocked {
                user_id,
                achievement_id,
                timestamp: Utc::now(),
            },
            ProgressCommand::CompleteDailyGoal { user_id } => ProgressEvent::DailyGoalCompleted {
                user_id,
                timestamp: Utc::now(),
            },
            ProgressCommand::ResetDaily { user_id } => {
                let mut progress = self.load_aggregate(user_id).await?;
                progress.reset_daily();

                // スナップショットを保存
                self.snapshot_store.save_snapshot(user_id, progress).await?;

                return Ok(());
            },
            ProgressCommand::ResetWeekly { user_id } => {
                let mut progress = self.load_aggregate(user_id).await?;
                progress.reset_weekly();

                // スナップショットを保存
                self.snapshot_store.save_snapshot(user_id, progress).await?;

                return Ok(());
            },
        };

        // ユーザーIDを取得
        let user_id = match &event {
            ProgressEvent::LearningStarted { user_id, .. }
            | ProgressEvent::ItemCompleted { user_id, .. }
            | ProgressEvent::SessionCompleted { user_id, .. }
            | ProgressEvent::StreakUpdated { user_id, .. }
            | ProgressEvent::AchievementUnlocked { user_id, .. }
            | ProgressEvent::DailyGoalCompleted { user_id, .. } => *user_id,
        };

        // イベントを保存・発行
        self.save_event(user_id, event).await?;

        // 定期的にスナップショットを作成
        let progress = self.load_aggregate(user_id).await?;
        if progress.version % 10 == 0 {
            self.snapshot_store.save_snapshot(user_id, progress).await?;
        }

        Ok(())
    }
}
