//! イベントストア実装
//!
//! このモジュールは Event Sourcing パターンのための
//! `PostgreSQL` ベースのイベントストアを提供します。

use async_trait::async_trait;
use domain_events::{DomainEvent, EventError, EventStore};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// `PostgreSQL` ベースのイベントストア実装
#[allow(clippy::module_name_repetitions)] // 具体的な実装名
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    /// 新しいイベントストアインスタンスを作成
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // PgPool cannot be used in const fn
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// イベントストアテーブルを初期化
    ///
    /// # Errors
    ///
    /// テーブル作成に失敗した場合はエラーを返す
    pub async fn initialize(&self) -> Result<(), EventError> {
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS events (
                event_id UUID PRIMARY KEY,
                stream_id VARCHAR(255) NOT NULL,
                sequence_number BIGINT NOT NULL,
                event_type VARCHAR(100) NOT NULL,
                event_data JSONB NOT NULL,
                metadata JSONB NOT NULL,
                occurred_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(stream_id, sequence_number)
            );
            
            CREATE INDEX IF NOT EXISTS idx_events_stream_id ON events(stream_id);
            CREATE INDEX IF NOT EXISTS idx_events_occurred_at ON events(occurred_at);
            CREATE INDEX IF NOT EXISTS idx_events_event_type ON events(event_type);
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| EventError::Store(format!("Failed to initialize event store: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    /// ストリームにイベントを追加
    async fn append(&self, stream_id: &str, events: Vec<DomainEvent>) -> Result<(), EventError> {
        // トランザクション開始
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| EventError::Store(format!("Failed to begin transaction: {e}")))?;

        // 現在の最大シーケンス番号を取得
        let current_sequence: Option<i64> =
            sqlx::query_scalar("SELECT MAX(sequence_number) FROM events WHERE stream_id = $1")
                .bind(stream_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| EventError::Store(format!("Failed to get sequence number: {e}")))?;

        let mut sequence_number = current_sequence.unwrap_or(0);

        // 各イベントを保存
        for event in events {
            sequence_number += 1;

            let event_id = event.metadata().event_id.as_uuid();
            let event_type = event.event_type();
            let event_data = serde_json::to_value(&event)
                .map_err(|e| EventError::Store(format!("Failed to serialize event: {e}")))?;
            let metadata = serde_json::to_value(event.metadata())
                .map_err(|e| EventError::Store(format!("Failed to serialize metadata: {e}")))?;
            let occurred_at = event.metadata().occurred_at;

            sqlx::query(
                r"
                INSERT INTO events (
                    event_id, stream_id, sequence_number, event_type,
                    event_data, metadata, occurred_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                ",
            )
            .bind(event_id)
            .bind(stream_id)
            .bind(sequence_number)
            .bind(event_type)
            .bind(event_data)
            .bind(metadata)
            .bind(occurred_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| EventError::Store(format!("Failed to insert event: {e}")))?;
        }

        // トランザクションコミット
        tx.commit()
            .await
            .map_err(|e| EventError::Store(format!("Failed to commit transaction: {e}")))?;

        Ok(())
    }

    /// ストリームからイベントを読み込む
    async fn read(&self, stream_id: &str, from: usize) -> Result<Vec<DomainEvent>, EventError> {
        let rows = sqlx::query(
            r"
            SELECT event_data
            FROM events
            WHERE stream_id = $1 AND sequence_number >= $2
            ORDER BY sequence_number ASC
            ",
        )
        .bind(stream_id)
        .bind(i64::try_from(from).unwrap_or(i64::MAX))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventError::Store(format!("Failed to read events: {e}")))?;

        let mut events = Vec::new();
        for row in rows {
            let event_data: JsonValue = row.get("event_data");
            let event: DomainEvent = serde_json::from_value(event_data).map_err(|e| {
                EventError::Deserialization(format!("Failed to deserialize event: {e}"))
            })?;
            events.push(event);
        }

        Ok(events)
    }
}

/// イベントのスナップショット管理
pub struct SnapshotStore {
    pool: PgPool,
}

impl SnapshotStore {
    /// 新しいスナップショットストアを作成
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // PgPool cannot be used in const fn
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// スナップショットテーブルを初期化
    ///
    /// # Errors
    ///
    /// テーブル作成に失敗した場合はエラーを返す
    pub async fn initialize(&self) -> Result<(), EventError> {
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS snapshots (
                snapshot_id UUID PRIMARY KEY,
                stream_id VARCHAR(255) NOT NULL,
                sequence_number BIGINT NOT NULL,
                aggregate_type VARCHAR(100) NOT NULL,
                aggregate_data JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(stream_id, sequence_number)
            );
            
            CREATE INDEX IF NOT EXISTS idx_snapshots_stream_id ON snapshots(stream_id);
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| EventError::Store(format!("Failed to initialize snapshot store: {e}")))?;

        Ok(())
    }

    /// スナップショットを保存
    ///
    /// # Errors
    ///
    /// 保存に失敗した場合はエラーを返す
    pub async fn save<T>(
        &self,
        stream_id: &str,
        sequence_number: i64,
        aggregate: &T,
    ) -> Result<(), EventError>
    where
        T: serde::Serialize + Send + Sync,
    {
        let snapshot_id = Uuid::new_v4();
        let aggregate_type = std::any::type_name::<T>();
        let aggregate_data = serde_json::to_value(aggregate)
            .map_err(|e| EventError::Store(format!("Failed to serialize aggregate: {e}")))?;

        sqlx::query(
            r"
            INSERT INTO snapshots (
                snapshot_id, stream_id, sequence_number,
                aggregate_type, aggregate_data
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (stream_id, sequence_number) DO UPDATE
            SET aggregate_data = EXCLUDED.aggregate_data,
                created_at = NOW()
            ",
        )
        .bind(snapshot_id)
        .bind(stream_id)
        .bind(sequence_number)
        .bind(aggregate_type)
        .bind(aggregate_data)
        .execute(&self.pool)
        .await
        .map_err(|e| EventError::Store(format!("Failed to save snapshot: {e}")))?;

        Ok(())
    }

    /// 最新のスナップショットを取得
    ///
    /// # Errors
    ///
    /// 取得に失敗した場合はエラーを返す
    pub async fn get_latest<T>(&self, stream_id: &str) -> Result<Option<(i64, T)>, EventError>
    where
        T: serde::de::DeserializeOwned,
    {
        let row = sqlx::query(
            r"
            SELECT sequence_number, aggregate_data
            FROM snapshots
            WHERE stream_id = $1
            ORDER BY sequence_number DESC
            LIMIT 1
            ",
        )
        .bind(stream_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EventError::Store(format!("Failed to get snapshot: {e}")))?;

        match row {
            Some(row) => {
                let sequence_number: i64 = row.get("sequence_number");
                let aggregate_data: JsonValue = row.get("aggregate_data");
                let aggregate: T = serde_json::from_value(aggregate_data).map_err(|e| {
                    EventError::Deserialization(format!("Failed to deserialize snapshot: {e}"))
                })?;
                Ok(Some((sequence_number, aggregate)))
            },
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    // 実際のデータベース接続テストは integration tests で実施
}
