# Event Sourcing 設計

## 概要

Event Sourcing は、アプリケーションの状態変更をイベントのシーケンスとして保存するパターンです。
effect では、すべての学習行動をイベントとして記録し、学習履歴の完全な追跡を実現します。

## イベント設計

### イベントの基本構造

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub metadata: EventMetadata,
    pub created_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub user_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}
```

## 主要なドメインイベント

### 単語関連イベント

```rust
// 単語作成
WordCreated {
    word_id: Uuid,
    text: String,
    meaning: String,
    difficulty: u8,
    category: String,
    tags: Vec<String>,
}

// 単語更新
WordUpdated {
    word_id: Uuid,
    changes: HashMap<String, Value>,
}

// 単語削除
WordDeleted {
    word_id: Uuid,
}
```

### 学習セッション関連イベント

```rust
// セッション開始
SessionStarted {
    session_id: Uuid,
    user_id: Uuid,
    word_ids: Vec<Uuid>,
    mode: LearningMode,
}

// 問題回答
QuestionAnswered {
    session_id: Uuid,
    word_id: Uuid,
    is_correct: bool,
    response_time_ms: u32,
    attempt_number: u8,
}

// セッション完了
SessionCompleted {
    session_id: Uuid,
    total_questions: u32,
    correct_answers: u32,
    duration_seconds: u32,
}
```

### 進捗関連イベント

```rust
// 進捗更新
ProgressUpdated {
    user_id: Uuid,
    word_id: Uuid,
    repetition_count: u32,
    easiness_factor: f32,
    interval_days: u32,
    next_review_date: DateTime<Utc>,
}

// ストリーク更新
StreakUpdated {
    user_id: Uuid,
    consecutive_days: u32,
    last_study_date: DateTime<Utc>,
}
```

## Event Store 実装

### PostgreSQL スキーマ

```sql
-- イベントテーブル
CREATE TABLE events (
    event_id UUID PRIMARY KEY,
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version BIGINT NOT NULL
);

-- インデックス
CREATE INDEX idx_events_aggregate ON events(aggregate_id, version);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_created ON events(created_at);

-- スナップショットテーブル
CREATE TABLE snapshots (
    aggregate_id UUID PRIMARY KEY,
    aggregate_type VARCHAR(255) NOT NULL,
    snapshot_data JSONB NOT NULL,
    version BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### イベントストアインターフェース

```rust
#[async_trait]
pub trait EventStore {
    // イベントの保存
    async fn append_events(
        &self,
        aggregate_id: Uuid,
        events: Vec<DomainEvent>,
        expected_version: Option<i64>,
    ) -> Result<()>;

    // イベントの取得
    async fn get_events(
        &self,
        aggregate_id: Uuid,
        from_version: Option<i64>,
    ) -> Result<Vec<DomainEvent>>;

    // スナップショットの保存
    async fn save_snapshot(
        &self,
        aggregate_id: Uuid,
        snapshot: AggregateSnapshot,
    ) -> Result<()>;

    // スナップショットの取得
    async fn get_snapshot(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Option<AggregateSnapshot>>;
}
```

## イベント処理

### イベントハンドラー

```rust
#[async_trait]
pub trait EventHandler {
    type Event;

    async fn handle(&self, event: Self::Event) -> Result<()>;
}

// 実装例
pub struct ProjectionHandler {
    query_repository: Arc<dyn QueryRepository>,
}

#[async_trait]
impl EventHandler for ProjectionHandler {
    type Event = DomainEvent;

    async fn handle(&self, event: Self::Event) -> Result<()> {
        match event.event_type.as_str() {
            "WordCreated" => self.handle_word_created(event).await,
            "SessionCompleted" => self.handle_session_completed(event).await,
            _ => Ok(()),
        }
    }
}
```

## イベントソーシングの利点

1. **完全な監査証跡**: すべての変更履歴を保持
2. **時間旅行**: 任意の時点の状態を再現可能
3. **イベント再生**: バグ修正後の状態再構築
4. **分析**: 学習パターンの詳細分析
5. **統合**: 他システムへのイベント配信

## パフォーマンス最適化

### スナップショット

- 100イベントごとにスナップショットを作成
- 集約の再構築時間を短縮

### イベントの圧縮

- 古いイベントは圧縮して保存
- アクセス頻度の低いイベントはアーカイブ

### プロジェクションの事前計算

- よく使われるクエリ用のRead Modelを事前計算
- 非正規化されたビューの維持
