//! リポジトリ実装

use std::sync::Arc;

use async_trait::async_trait;
use shared_error::{DomainError, DomainResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::aggregates::VocabularyEntry,
    ports::outbound::{EventBus, EventStore, VocabularyRepository},
};

/// Event Store ベースのリポジトリ実装
pub struct EventStoreVocabularyRepository {
    event_store: Arc<dyn EventStore>,
    event_bus:   Arc<dyn EventBus>,
    read_store:  PgPool, // 読み取り用の補助ストア
}

impl EventStoreVocabularyRepository {
    /// 新しいリポジトリを作成
    pub fn new(
        event_store: Arc<dyn EventStore>,
        event_bus: Arc<dyn EventBus>,
        read_store: PgPool,
    ) -> Self {
        Self {
            event_store,
            event_bus,
            read_store,
        }
    }
}

#[async_trait]
impl VocabularyRepository for EventStoreVocabularyRepository {
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<VocabularyEntry>> {
        // Event Store から集約を読み込み
        self.event_store.load_aggregate(id).await
    }

    async fn find_by_word(&self, word: &str) -> DomainResult<Option<VocabularyEntry>> {
        // 読み取り用の補助ストアから検索
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT entry_id FROM vocabulary_entries WHERE word = $1 LIMIT 1",
        )
        .bind(word)
        .fetch_optional(&self.read_store)
        .await
        .map_err(|e| DomainError::Internal(format!("Database error: {e}")))?;

        match row {
            Some((entry_id,)) => {
                let entry_id: Uuid = entry_id
                    .parse()
                    .map_err(|e| DomainError::Internal(format!("Invalid UUID: {e}")))?;
                self.find_by_id(entry_id).await
            },
            None => Ok(None),
        }
    }

    async fn save(&self, entry: &mut VocabularyEntry) -> DomainResult<()> {
        // 保留中のイベントを取得
        let events = entry.take_events();

        if events.is_empty() {
            // イベントがない場合は何もしない
            return Ok(());
        }

        // Event Store にイベントを保存
        self.event_store
            .save_aggregate(entry.id(), events.clone(), Some(entry.version()))
            .await?;

        // Event Bus にイベントを発行
        self.event_bus.publish(events).await?;

        Ok(())
    }

    async fn delete(&self, _id: Uuid) -> DomainResult<()> {
        // Event Sourcing では削除の代わりに削除イベントを記録することが多い
        // ここでは簡易的に NotImplemented を返す
        Err(DomainError::Internal(
            "Delete operation is not implemented for Event Store".to_string(),
        ))
    }
}
