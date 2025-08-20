//! アプリケーションサービス

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tracing::{error, info};

use super::event_handlers::ProgressEventHandler;
use crate::{
    domain::ProjectionState,
    error::Result,
    ports::{
        inbound::EventProcessor,
        outbound::{EventStoreReader, ProjectionStateStore},
    },
};

/// イベントプロセッサー実装
pub struct EventProcessorService {
    event_store:   Arc<dyn EventStoreReader>,
    state_store:   Arc<dyn ProjectionStateStore>,
    event_handler: Arc<ProgressEventHandler>,
    batch_size:    usize,
}

impl EventProcessorService {
    pub fn new(
        event_store: Arc<dyn EventStoreReader>,
        state_store: Arc<dyn ProjectionStateStore>,
        event_handler: Arc<ProgressEventHandler>,
        batch_size: usize,
    ) -> Self {
        Self {
            event_store,
            state_store,
            event_handler,
            batch_size,
        }
    }

    async fn get_or_create_projection_state(
        &self,
        projection_name: &str,
    ) -> Result<ProjectionState> {
        Ok(self
            .state_store
            .get_state(projection_name)
            .await?
            .unwrap_or_else(|| ProjectionState {
                projection_name: projection_name.to_string(),
                last_position:   0,
                last_event_id:   None,
                updated_at:      Utc::now(),
            }))
    }
}

#[async_trait]
impl EventProcessor for EventProcessorService {
    async fn process_events(&self) -> Result<()> {
        let projection_name = "progress_projection";

        // 現在の位置を取得
        let mut state = self.get_or_create_projection_state(projection_name).await?;

        // イベントを読み込み
        let events = self
            .event_store
            .read_events(state.last_position, self.batch_size)
            .await?;

        if events.is_empty() {
            return Ok(());
        }

        info!(
            "Processing {} events from position {}",
            events.len(),
            state.last_position
        );

        // 各イベントを処理
        for event in &events {
            match self.event_handler.handle_event(event).await {
                Ok(_) => {
                    state.last_position = event.position;
                    state.last_event_id = Some(event.event_id);
                },
                Err(e) => {
                    error!("Failed to handle event {}: {}", event.event_id, e);
                    // エラーが発生した場合も位置を更新してスキップ
                    state.last_position = event.position;
                    state.last_event_id = Some(event.event_id);
                },
            }
        }

        // 状態を保存
        state.updated_at = Utc::now();
        self.state_store.save_state(&state).await?;

        info!("Processed events up to position {}", state.last_position);
        Ok(())
    }

    async fn rebuild_projection(&self, projection_name: &str) -> Result<()> {
        info!("Rebuilding projection: {}", projection_name);

        // 状態をリセット
        let mut state = ProjectionState {
            projection_name: projection_name.to_string(),
            last_position:   0,
            last_event_id:   None,
            updated_at:      Utc::now(),
        };

        self.state_store.save_state(&state).await?;

        // TODO: Read Model のデータをクリア

        // 最初からイベントを処理
        loop {
            let events = self
                .event_store
                .read_events(state.last_position, self.batch_size)
                .await?;

            if events.is_empty() {
                break;
            }

            for event in &events {
                match self.event_handler.handle_event(event).await {
                    Ok(_) => {
                        state.last_position = event.position;
                        state.last_event_id = Some(event.event_id);
                    },
                    Err(e) => {
                        error!(
                            "Failed to handle event during rebuild {}: {}",
                            event.event_id, e
                        );
                        state.last_position = event.position;
                        state.last_event_id = Some(event.event_id);
                    },
                }
            }

            state.updated_at = Utc::now();
            self.state_store.save_state(&state).await?;
        }

        info!(
            "Projection rebuild completed at position {}",
            state.last_position
        );
        Ok(())
    }
}
