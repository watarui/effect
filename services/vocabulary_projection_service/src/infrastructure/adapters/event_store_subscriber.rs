//! Event Store サブスクライバー実装

use async_trait::async_trait;

use crate::{
    domain::events::StoredEvent,
    error::Result,
    ports::outbound::{EventStream, EventSubscriber},
};

/// Event Store サブスクライバー（モック実装）
pub struct EventStoreSubscriber {
    _event_store_url: String,
}

impl EventStoreSubscriber {
    pub fn new(event_store_url: String) -> Self {
        Self {
            _event_store_url: event_store_url,
        }
    }
}

#[async_trait]
impl EventSubscriber for EventStoreSubscriber {
    async fn fetch_events(
        &self,
        _from_position: i64,
        _batch_size: usize,
    ) -> Result<Vec<StoredEvent>> {
        // TODO: 実際の Event Store Service との gRPC 通信を実装
        // 現在は空のベクタを返す
        Ok(vec![])
    }

    async fn subscribe(&self, _from_position: i64) -> Result<EventStream> {
        // TODO: ストリーミング実装
        Ok(EventStream {})
    }
}
