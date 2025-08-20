//! イベント処理サービス

use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::{
    application::event_handlers::EventHandler,
    config::Config,
    domain::projections::{ProjectionCheckpoint, ProjectionState},
    error::Result,
    ports::{
        inbound::{EventProcessorUseCase, ProcessorStatus},
        outbound::{EventSubscriber, ProjectionStateRepository, ReadModelRepository},
    },
};

/// イベントプロセッサー
pub struct EventProcessor<E, R, P>
where
    E: EventSubscriber,
    R: ReadModelRepository,
    P: ProjectionStateRepository,
{
    config:           Config,
    event_subscriber: Arc<E>,
    event_handler:    Arc<EventHandler<R>>,
    state_repository: Arc<P>,
    read_repository:  Arc<R>,
    is_running:       Arc<RwLock<bool>>,
}

impl<E, R, P> EventProcessor<E, R, P>
where
    E: EventSubscriber,
    R: ReadModelRepository + Clone,
    P: ProjectionStateRepository,
{
    pub fn new(
        config: Config,
        event_subscriber: E,
        read_repository: R,
        state_repository: P,
    ) -> Self {
        let event_handler = EventHandler::new(read_repository.clone());

        Self {
            config,
            event_subscriber: Arc::new(event_subscriber),
            event_handler: Arc::new(event_handler),
            state_repository: Arc::new(state_repository),
            read_repository: Arc::new(read_repository),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// イベント処理ループ
    pub async fn process_events(&self) -> Result<()> {
        info!(
            "Starting event processor for projection: {}",
            self.config.projection.name
        );

        // 実行フラグを設定
        *self.is_running.write().await = true;

        // プロジェクション状態を取得または初期化
        let mut state = self.get_or_init_state().await?;

        while *self.is_running.read().await {
            match self.process_batch(&mut state).await {
                Ok(events_processed) => {
                    if events_processed > 0 {
                        debug!("Processed {} events", events_processed);
                    }
                },
                Err(e) => {
                    error!("Error processing events: {}", e);
                    state.record_error(e.to_string());
                    self.state_repository
                        .record_error(&self.config.projection.name, &e.to_string())
                        .await
                        .ok();

                    // エラー時は少し待機
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                },
            }

            // ポーリング間隔で待機
            tokio::time::sleep(tokio::time::Duration::from_millis(
                self.config.event_store.polling_interval_ms,
            ))
            .await;
        }

        info!("Event processor stopped");
        Ok(())
    }

    async fn get_or_init_state(&self) -> Result<ProjectionState> {
        match self
            .state_repository
            .get_state(&self.config.projection.name)
            .await?
        {
            Some(state) => {
                info!(
                    "Resuming projection from position {}",
                    state.last_processed_position
                );
                Ok(state)
            },
            None => {
                info!("Initializing new projection");
                let state = ProjectionState::new(self.config.projection.name.clone());

                let mut tx = self.read_repository.begin_transaction().await?;
                self.state_repository.save_state(&mut tx, &state).await?;
                tx.commit().await?;

                Ok(state)
            },
        }
    }

    async fn process_batch(&self, state: &mut ProjectionState) -> Result<usize> {
        let events = self
            .event_subscriber
            .fetch_events(
                state.last_processed_position,
                self.config.event_store.batch_size,
            )
            .await?;

        if events.is_empty() {
            return Ok(0);
        }

        let mut tx = self.read_repository.begin_transaction().await?;
        let mut events_processed = 0;

        for event in &events {
            self.event_handler.handle_event(&mut tx, event).await?;
            events_processed += 1;

            // チェックポイント間隔に達したら保存
            if events_processed % self.config.projection.checkpoint_interval == 0 {
                let checkpoint = ProjectionCheckpoint::new(
                    self.config.projection.name.clone(),
                    event.position,
                    Some(event.event_id),
                    events_processed as i32,
                );
                self.state_repository
                    .save_checkpoint(&mut tx, &checkpoint)
                    .await?;
            }
        }

        // 最後のイベント情報で状態を更新
        if let Some(last_event) = events.last() {
            state.update_position(last_event.position, Some(last_event.event_id));
            self.state_repository.save_state(&mut tx, state).await?;
        }

        tx.commit().await?;
        Ok(events_processed)
    }
}

#[async_trait::async_trait]
impl<E, R, P> EventProcessorUseCase for EventProcessor<E, R, P>
where
    E: EventSubscriber,
    R: ReadModelRepository + Clone,
    P: ProjectionStateRepository,
{
    async fn start_processing(&self) -> Result<()> {
        self.process_events().await
    }

    async fn stop_processing(&self) -> Result<()> {
        info!("Stopping event processor");
        *self.is_running.write().await = false;
        Ok(())
    }

    async fn get_status(&self) -> Result<ProcessorStatus> {
        let state = self
            .state_repository
            .get_state(&self.config.projection.name)
            .await?
            .unwrap_or_else(|| ProjectionState::new(self.config.projection.name.clone()));

        Ok(ProcessorStatus {
            is_running:              *self.is_running.read().await,
            last_processed_position: state.last_processed_position,
            events_processed_total:  0, // TODO: 実装
            error_count:             state.error_count as u32,
        })
    }
}
