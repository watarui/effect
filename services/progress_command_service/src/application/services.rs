//! アプリケーションサービス

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::Progress,
    error::Result,
    ports::{inbound::ProgressQueryPort, outbound::*},
};

/// 進捗照会サービス
pub struct ProgressQueryService {
    event_store:    Arc<dyn EventStorePort>,
    snapshot_store: Arc<dyn SnapshotStorePort>,
}

impl ProgressQueryService {
    pub fn new(
        event_store: Arc<dyn EventStorePort>,
        snapshot_store: Arc<dyn SnapshotStorePort>,
    ) -> Self {
        Self {
            event_store,
            snapshot_store,
        }
    }
}

#[async_trait]
impl ProgressQueryPort for ProgressQueryService {
    async fn get_user_progress(&self, user_id: Uuid) -> Result<Progress> {
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
}
