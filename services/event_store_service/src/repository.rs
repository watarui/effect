//! Event Store リポジトリ実装

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL ベースの Event Store
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    /// 新しいインスタンスを作成
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// イベントを保存
    pub async fn append_events(
        &self,
        stream_id: Uuid,
        stream_type: &str,
        events: Vec<serde_json::Value>,
        expected_version: Option<i64>,
    ) -> Result<i64, EventStoreError> {
        let mut tx = self.pool.begin().await?;

        // 現在のバージョンを取得
        let current_version: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(version) FROM events WHERE stream_id = $1 AND stream_type = $2",
        )
        .bind(stream_id)
        .bind(stream_type)
        .fetch_one(&mut *tx)
        .await?;

        let current_version = current_version.unwrap_or(-1);

        // 楽観的ロックのチェック
        if let Some(expected) = expected_version
            && current_version != expected
        {
            return Err(EventStoreError::VersionConflict {
                expected,
                actual: current_version,
            });
        }

        let mut next_version = current_version;

        // イベントを挿入
        for event in events {
            next_version += 1;
            let event_id = Uuid::new_v4();
            let metadata = serde_json::json!({});

            sqlx::query(
                "INSERT INTO events (event_id, stream_id, stream_type, version, event_type, data, metadata, created_at) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())"
            )
            .bind(event_id)
            .bind(stream_id)
            .bind(stream_type)
            .bind(next_version)
            .bind("Event") // TODO: 実際のイベントタイプを使用
            .bind(&event)
            .bind(&metadata)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(next_version)
    }

    /// イベントを取得
    pub async fn get_events(
        &self,
        stream_id: Uuid,
        stream_type: &str,
        from_version: i64,
        to_version: Option<i64>,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let query = if let Some(to) = to_version {
            sqlx::query_as::<
                _,
                (
                    Uuid,
                    Uuid,
                    String,
                    i64,
                    String,
                    serde_json::Value,
                    serde_json::Value,
                    DateTime<Utc>,
                    i64,
                ),
            >(
                "SELECT event_id, stream_id, stream_type, version, event_type, data, metadata, \
                 created_at, position 
                 FROM events 
                 WHERE stream_id = $1 AND stream_type = $2 AND version >= $3 AND version <= $4 
                 ORDER BY version",
            )
            .bind(stream_id)
            .bind(stream_type)
            .bind(from_version)
            .bind(to)
        } else {
            sqlx::query_as::<
                _,
                (
                    Uuid,
                    Uuid,
                    String,
                    i64,
                    String,
                    serde_json::Value,
                    serde_json::Value,
                    DateTime<Utc>,
                    i64,
                ),
            >(
                "SELECT event_id, stream_id, stream_type, version, event_type, data, metadata, \
                 created_at, position 
                 FROM events 
                 WHERE stream_id = $1 AND stream_type = $2 AND version >= $3 
                 ORDER BY version",
            )
            .bind(stream_id)
            .bind(stream_type)
            .bind(from_version)
        };

        let rows = query.fetch_all(&self.pool).await?;

        let events = rows
            .into_iter()
            .map(|row| StoredEvent {
                event_id:    row.0,
                stream_id:   row.1,
                stream_type: row.2,
                version:     row.3,
                event_type:  row.4,
                data:        row.5,
                metadata:    row.6,
                created_at:  row.7,
                position:    row.8,
            })
            .collect();

        Ok(events)
    }

    /// スナップショットを保存
    pub async fn save_snapshot(
        &self,
        stream_id: Uuid,
        stream_type: &str,
        version: i64,
        data: serde_json::Value,
    ) -> Result<(), EventStoreError> {
        let snapshot_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO snapshots (snapshot_id, stream_id, stream_type, version, data, \
             created_at) 
             VALUES ($1, $2, $3, $4, $5, NOW()) 
             ON CONFLICT (stream_id, stream_type, version) 
             DO UPDATE SET data = EXCLUDED.data, created_at = NOW()",
        )
        .bind(snapshot_id)
        .bind(stream_id)
        .bind(stream_type)
        .bind(version)
        .bind(&data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// スナップショットを取得
    pub async fn get_snapshot(
        &self,
        stream_id: Uuid,
        stream_type: &str,
        max_version: Option<i64>,
    ) -> Result<Option<Snapshot>, EventStoreError> {
        let query = if let Some(max_ver) = max_version {
            sqlx::query_as::<_, (Uuid, Uuid, String, i64, serde_json::Value, DateTime<Utc>)>(
                "SELECT snapshot_id, stream_id, stream_type, version, data, created_at 
                 FROM snapshots 
                 WHERE stream_id = $1 AND stream_type = $2 AND version <= $3 
                 ORDER BY version DESC 
                 LIMIT 1",
            )
            .bind(stream_id)
            .bind(stream_type)
            .bind(max_ver)
        } else {
            sqlx::query_as::<_, (Uuid, Uuid, String, i64, serde_json::Value, DateTime<Utc>)>(
                "SELECT snapshot_id, stream_id, stream_type, version, data, created_at 
                 FROM snapshots 
                 WHERE stream_id = $1 AND stream_type = $2 
                 ORDER BY version DESC 
                 LIMIT 1",
            )
            .bind(stream_id)
            .bind(stream_type)
        };

        let row = query.fetch_optional(&self.pool).await?;

        Ok(row.map(|r| Snapshot {
            snapshot_id: r.0,
            stream_id:   r.1,
            stream_type: r.2,
            version:     r.3,
            data:        r.4,
            created_at:  r.5,
        }))
    }
}

/// 保存されたイベント
#[derive(Debug, Clone)]
pub struct StoredEvent {
    pub event_id:    Uuid,
    pub stream_id:   Uuid,
    pub stream_type: String,
    pub version:     i64,
    pub event_type:  String,
    pub data:        serde_json::Value,
    pub metadata:    serde_json::Value,
    pub created_at:  DateTime<Utc>,
    pub position:    i64,
}

/// スナップショット
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub snapshot_id: Uuid,
    pub stream_id:   Uuid,
    pub stream_type: String,
    pub version:     i64,
    pub data:        serde_json::Value,
    pub created_at:  DateTime<Utc>,
}

/// Event Store のエラー
#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: i64, actual: i64 },

    #[error("Stream not found: {0}")]
    #[allow(dead_code)]
    StreamNotFound(Uuid),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
