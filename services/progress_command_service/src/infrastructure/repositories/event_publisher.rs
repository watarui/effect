//! イベントパブリッシャー実装

use async_trait::async_trait;
use redis::{AsyncCommands, aio::ConnectionManager};

use crate::{
    domain::events::ProgressEvent,
    error::{Error, Result},
    ports::outbound::EventPublisherPort,
};

/// Redis イベントパブリッシャー
pub struct RedisEventPublisher {
    redis_conn: ConnectionManager,
}

impl RedisEventPublisher {
    pub fn new(redis_conn: ConnectionManager) -> Self {
        Self { redis_conn }
    }
}

#[async_trait]
impl EventPublisherPort for RedisEventPublisher {
    async fn publish(&self, event: ProgressEvent) -> Result<()> {
        let event_json =
            serde_json::to_string(&event).map_err(|e| Error::Serialization(e.to_string()))?;

        let channel = "progress_events";

        let mut conn = self.redis_conn.clone();
        conn.publish::<_, _, ()>(channel, event_json)
            .await
            .map_err(|e| Error::PubSub(e.to_string()))?;

        Ok(())
    }
}
