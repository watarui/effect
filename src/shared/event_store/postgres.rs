// PostgreSQL Event Store 実装

use super::*;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde_json;
use sqlx::{PgPool, Row};
use std::pin::Pin;
use uuid::Uuid;

/// PostgreSQL ベースの Event Store 実装
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// テーブルを初期化
    pub async fn init_tables(&self) -> Result<(), sqlx::Error> {
        // イベントストアテーブル
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                sequence_number BIGSERIAL PRIMARY KEY,
                event_id UUID NOT NULL UNIQUE,
                stream_id VARCHAR(255) NOT NULL,
                aggregate_id VARCHAR(255) NOT NULL,
                aggregate_type VARCHAR(100) NOT NULL,
                event_type VARCHAR(100) NOT NULL,
                event_data JSONB NOT NULL,
                event_version BIGINT NOT NULL,
                occurred_at TIMESTAMPTZ NOT NULL,
                metadata JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                INDEX idx_stream_id_version (stream_id, event_version),
                INDEX idx_aggregate_id (aggregate_id),
                INDEX idx_occurred_at (occurred_at),
                UNIQUE (stream_id, event_version)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        // ストリーム情報テーブル
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS streams (
                stream_id VARCHAR(255) PRIMARY KEY,
                version BIGINT NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                deleted_at TIMESTAMPTZ
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        // スナップショットテーブル
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS snapshots (
                aggregate_id VARCHAR(255) NOT NULL,
                aggregate_type VARCHAR(100) NOT NULL,
                version BIGINT NOT NULL,
                data JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (aggregate_id, aggregate_type)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn append_to_stream<T>(
        &self,
        stream_id: &str,
        expected_version: Option<i64>,
        events: Vec<EventEnvelope<T>>,
    ) -> Result<i64, EventStoreError>
    where
        T: Serialize + Send + Sync,
    {
        let mut tx = self.pool.begin().await
            .map_err(|e| EventStoreError::ConnectionError(e.to_string()))?;
        
        // 現在のバージョンを取得
        let current_version = sqlx::query!(
            r#"
            SELECT version FROM streams 
            WHERE stream_id = $1 AND deleted_at IS NULL
            FOR UPDATE
            "#,
            stream_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| EventStoreError::InternalError(e.to_string()))?
        .map(|r| r.version);
        
        // バージョンチェック
        if let Some(expected) = expected_version {
            match current_version {
                Some(actual) if actual != expected => {
                    return Err(EventStoreError::ConcurrencyConflict { expected, actual });
                }
                None if expected != -1 => {
                    return Err(EventStoreError::StreamNotFound(stream_id.to_string()));
                }
                _ => {}
            }
        }
        
        let mut new_version = current_version.unwrap_or(-1);
        
        // イベントを挿入
        for event in &events {
            new_version += 1;
            
            let event_data = serde_json::to_value(&event.event_data)
                .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
            
            let metadata = serde_json::to_value(&event.metadata)
                .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
            
            sqlx::query!(
                r#"
                INSERT INTO events (
                    event_id, stream_id, aggregate_id, aggregate_type,
                    event_type, event_data, event_version, occurred_at, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                Uuid::parse_str(&event.event_id).unwrap(),
                stream_id,
                event.aggregate_id,
                event.aggregate_type,
                event.event_type,
                event_data,
                new_version,
                event.occurred_at,
                metadata
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| EventStoreError::InternalError(e.to_string()))?;
        }
        
        // ストリーム情報を更新
        if current_version.is_some() {
            sqlx::query!(
                r#"
                UPDATE streams 
                SET version = $2, updated_at = NOW() 
                WHERE stream_id = $1
                "#,
                stream_id,
                new_version
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| EventStoreError::InternalError(e.to_string()))?;
        } else {
            sqlx::query!(
                r#"
                INSERT INTO streams (stream_id, version) 
                VALUES ($1, $2)
                "#,
                stream_id,
                new_version
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| EventStoreError::InternalError(e.to_string()))?;
        }
        
        tx.commit().await
            .map_err(|e| EventStoreError::InternalError(e.to_string()))?;
        
        Ok(new_version)
    }
    
    async fn read_stream<T>(
        &self,
        stream_id: &str,
        options: ReadOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<EventEnvelope<T>, EventStoreError>> + Send>>, EventStoreError>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let from_version = options.from_version.unwrap_or(0);
        let to_version = options.to_version.unwrap_or(i64::MAX);
        let limit = options.max_count.unwrap_or(1000) as i64;
        
        let query = if options.backward {
            sqlx::query!(
                r#"
                SELECT 
                    event_id, stream_id, aggregate_id, aggregate_type,
                    event_type, event_data, event_version, sequence_number,
                    occurred_at, metadata
                FROM events
                WHERE stream_id = $1 
                    AND event_version >= $2 
                    AND event_version <= $3
                ORDER BY event_version DESC
                LIMIT $4
                "#,
                stream_id,
                from_version,
                to_version,
                limit
            )
        } else {
            sqlx::query!(
                r#"
                SELECT 
                    event_id, stream_id, aggregate_id, aggregate_type,
                    event_type, event_data, event_version, sequence_number,
                    occurred_at, metadata
                FROM events
                WHERE stream_id = $1 
                    AND event_version >= $2 
                    AND event_version <= $3
                ORDER BY event_version ASC
                LIMIT $4
                "#,
                stream_id,
                from_version,
                to_version,
                limit
            )
        };
        
        let stream = query
            .fetch(&self.pool)
            .map(move |result| {
                result
                    .map_err(|e| EventStoreError::InternalError(e.to_string()))
                    .and_then(|row| {
                        let event_data: T = serde_json::from_value(row.event_data)
                            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
                        
                        let metadata: EventMetadata = serde_json::from_value(row.metadata)
                            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
                        
                        Ok(EventEnvelope {
                            event_id: row.event_id.to_string(),
                            aggregate_id: row.aggregate_id,
                            aggregate_type: row.aggregate_type,
                            event_type: row.event_type,
                            event_data,
                            event_version: row.event_version,
                            sequence_number: row.sequence_number,
                            occurred_at: row.occurred_at,
                            metadata,
                        })
                    })
            });
        
        Ok(Box::pin(stream))
    }
    
    async fn read_all_forward<T>(
        &self,
        from_position: i64,
        max_count: Option<usize>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<EventEnvelope<T>, EventStoreError>> + Send>>, EventStoreError>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let limit = max_count.unwrap_or(1000) as i64;
        
        let stream = sqlx::query!(
            r#"
            SELECT 
                event_id, stream_id, aggregate_id, aggregate_type,
                event_type, event_data, event_version, sequence_number,
                occurred_at, metadata
            FROM events
            WHERE sequence_number > $1
            ORDER BY sequence_number ASC
            LIMIT $2
            "#,
            from_position,
            limit
        )
        .fetch(&self.pool)
        .map(move |result| {
            result
                .map_err(|e| EventStoreError::InternalError(e.to_string()))
                .and_then(|row| {
                    let event_data: T = serde_json::from_value(row.event_data)
                        .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
                    
                    let metadata: EventMetadata = serde_json::from_value(row.metadata)
                        .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
                    
                    Ok(EventEnvelope {
                        event_id: row.event_id.to_string(),
                        aggregate_id: row.aggregate_id,
                        aggregate_type: row.aggregate_type,
                        event_type: row.event_type,
                        event_data,
                        event_version: row.event_version,
                        sequence_number: row.sequence_number,
                        occurred_at: row.occurred_at,
                        metadata,
                    })
                })
        });
        
        Ok(Box::pin(stream))
    }
    
    async fn get_stream_info(&self, stream_id: &str) -> Result<Option<StreamInfo>, EventStoreError> {
        let info = sqlx::query!(
            r#"
            SELECT stream_id, version, created_at, updated_at
            FROM streams
            WHERE stream_id = $1 AND deleted_at IS NULL
            "#,
            stream_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EventStoreError::InternalError(e.to_string()))?
        .map(|row| StreamInfo {
            stream_id: row.stream_id,
            version: row.version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        });
        
        Ok(info)
    }
    
    async fn delete_stream(&self, stream_id: &str, expected_version: Option<i64>) -> Result<(), EventStoreError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| EventStoreError::ConnectionError(e.to_string()))?;
        
        // バージョンチェック
        if let Some(expected) = expected_version {
            let actual = sqlx::query!(
                r#"
                SELECT version FROM streams 
                WHERE stream_id = $1 AND deleted_at IS NULL
                FOR UPDATE
                "#,
                stream_id
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| EventStoreError::InternalError(e.to_string()))?
            .map(|r| r.version);
            
            match actual {
                Some(v) if v != expected => {
                    return Err(EventStoreError::ConcurrencyConflict { expected, actual: v });
                }
                None => return Err(EventStoreError::StreamNotFound(stream_id.to_string())),
                _ => {}
            }
        }
        
        // ソフトデリート
        sqlx::query!(
            r#"
            UPDATE streams 
            SET deleted_at = NOW() 
            WHERE stream_id = $1
            "#,
            stream_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| EventStoreError::InternalError(e.to_string()))?;
        
        tx.commit().await
            .map_err(|e| EventStoreError::InternalError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn save_snapshot<T>(
        &self,
        snapshot: Snapshot<T>,
    ) -> Result<(), EventStoreError>
    where
        T: Serialize + Send + Sync,
    {
        let data = serde_json::to_value(&snapshot.data)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
        
        sqlx::query!(
            r#"
            INSERT INTO snapshots (aggregate_id, aggregate_type, version, data, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (aggregate_id, aggregate_type) DO UPDATE SET
                version = EXCLUDED.version,
                data = EXCLUDED.data,
                created_at = EXCLUDED.created_at
            "#,
            snapshot.aggregate_id,
            snapshot.aggregate_type,
            snapshot.version,
            data,
            snapshot.created_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| EventStoreError::InternalError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn get_snapshot<T>(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
    ) -> Result<Option<Snapshot<T>>, EventStoreError>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let snapshot = sqlx::query!(
            r#"
            SELECT version, data, created_at
            FROM snapshots
            WHERE aggregate_id = $1 AND aggregate_type = $2
            "#,
            aggregate_id,
            aggregate_type
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EventStoreError::InternalError(e.to_string()))?
        .map(|row| {
            let data: T = serde_json::from_value(row.data)
                .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
            
            Ok(Snapshot {
                aggregate_id: aggregate_id.to_string(),
                aggregate_type: aggregate_type.to_string(),
                data,
                version: row.version,
                created_at: row.created_at,
            })
        })
        .transpose()?;
        
        Ok(snapshot)
    }
}