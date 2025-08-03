# Event Store 共通インターフェース

## 概要

CQRS/Event Sourcing パターンを実装するための共通 Event Store インターフェースです。
Progress Context と Vocabulary Context で使用される Event Store の抽象化を提供します。

## 主要コンポーネント

### EventStore トレイト

すべての Event Store 実装が満たすべき共通インターフェース：

- `append_to_stream` - イベントをストリームに追加
- `read_stream` - ストリームからイベントを読み取り
- `read_all_forward` - 全イベントを前方から読み取り
- `get_stream_info` - ストリーム情報を取得
- `delete_stream` - ストリームを削除（ソフトデリート）
- `save_snapshot` - スナップショットを保存
- `get_snapshot` - スナップショットを取得

### PostgresEventStore

PostgreSQL を使用した Event Store の実装：

```rust
use shared::event_store::{EventStore, PostgresEventStore};
use sqlx::PgPool;

let pool = PgPool::connect("postgres://...").await?;
let event_store = PostgresEventStore::new(pool);

// テーブルを初期化
event_store.init_tables().await?;
```

### EventEnvelope

イベントのメタデータを含むエンベロープ：

```rust
pub struct EventEnvelope<T> {
    pub event_id: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_data: T,
    pub event_version: i64,
    pub sequence_number: i64,
    pub occurred_at: DateTime<Utc>,
    pub metadata: EventMetadata,
}
```

## 使用例

### イベントの永続化

```rust
use shared::event_store::{EventEnvelope, EventMetadata};

// イベントを作成
let event = EventEnvelope {
    event_id: Uuid::new_v4().to_string(),
    aggregate_id: "user-123".to_string(),
    aggregate_type: "User".to_string(),
    event_type: "UserCreated".to_string(),
    event_data: UserCreatedEvent { /* ... */ },
    event_version: 1,
    sequence_number: 0, // 自動採番される
    occurred_at: Utc::now(),
    metadata: EventMetadata {
        correlation_id: Uuid::new_v4().to_string(),
        causation_id: None,
        user_id: "user-123".to_string(),
        source_service: "user-service".to_string(),
        idempotency_key: None,
    },
};

// ストリームに追加
let new_version = event_store
    .append_to_stream("user-123", Some(0), vec![event])
    .await?;
```

### イベントの読み取り

```rust
use shared::event_store::ReadOptions;
use futures::StreamExt;

// ストリームから読み取り
let options = ReadOptions {
    from_version: Some(1),
    to_version: None,
    max_count: Some(100),
    backward: false,
};

let mut stream = event_store
    .read_stream::<UserEvent>("user-123", options)
    .await?;

while let Some(result) = stream.next().await {
    let event = result?;
    println!("Event: {:?}", event);
}
```

### スナップショット

```rust
use shared::event_store::Snapshot;

// スナップショットを保存
let snapshot = Snapshot {
    aggregate_id: "user-123".to_string(),
    aggregate_type: "User".to_string(),
    data: UserAggregate { /* ... */ },
    version: 10,
    created_at: Utc::now(),
};

event_store.save_snapshot(snapshot).await?;

// スナップショットを取得
let snapshot = event_store
    .get_snapshot::<UserAggregate>("user-123", "User")
    .await?;
```

## 楽観的並行性制御

```rust
// 特定バージョンを期待してイベントを追加
let result = event_store
    .append_to_stream("user-123", Some(5), vec![event])
    .await;

match result {
    Err(EventStoreError::ConcurrencyConflict { expected, actual }) => {
        // 競合が発生した場合の処理
        println!("Expected version {}, but was {}", expected, actual);
    }
    Ok(new_version) => {
        println!("New version: {}", new_version);
    }
    Err(e) => return Err(e),
}
```

## データベーススキーマ

### events テーブル

```sql
CREATE TABLE events (
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
);
```

### streams テーブル

```sql
CREATE TABLE streams (
    stream_id VARCHAR(255) PRIMARY KEY,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

### snapshots テーブル

```sql
CREATE TABLE snapshots (
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    version BIGINT NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (aggregate_id, aggregate_type)
);
```

## 更新履歴

- 2025-08-03: 初版作成
