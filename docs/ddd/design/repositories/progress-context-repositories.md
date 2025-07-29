# Progress Context - リポジトリインターフェース

## 概要

Progress Context は純粋な CQRS Read Model であり、他のコンテキストから発行されるイベントを集約して、様々な統計情報を提供します。

このコンテキストの特徴：

- **集約なし**：全てがイベントから生成される Read Model
- **イベントソーシング**：イベントの履歴から状態を再構築
- **結果整合性**：リアルタイム性より正確性を優先

## EventStore

イベントの永続化と読み取りを担当する特別なリポジトリです。

### インターフェース定義

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// イベントストア
#[async_trait]
pub trait EventStore: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    /// イベントを追記
    async fn append_events(
        &self,
        stream_id: &str,
        events: Vec<DomainEvent>,
        expected_version: Option<u64>,
    ) -> Result<(), Self::Error>;
    
    /// ストリームからイベントを読み取り
    async fn read_events(
        &self,
        stream_id: &str,
        from_version: u64,
        to_version: Option<u64>,
    ) -> Result<Vec<PersistedEvent>, Self::Error>;
    
    /// グローバルイベントストリームを読み取り
    async fn read_all_events(
        &self,
        from_position: u64,
        limit: u32,
    ) -> Result<Vec<PersistedEvent>, Self::Error>;
    
    /// イベントタイプでフィルタリング
    async fn read_events_by_type(
        &self,
        event_type: &str,
        from_timestamp: DateTime<Utc>,
        to_timestamp: Option<DateTime<Utc>>,
        limit: u32,
    ) -> Result<Vec<PersistedEvent>, Self::Error>;
    
    /// スナップショットの保存
    async fn save_snapshot(
        &self,
        stream_id: &str,
        snapshot: Snapshot,
    ) -> Result<(), Self::Error>;
    
    /// 最新のスナップショットを取得
    async fn get_latest_snapshot(
        &self,
        stream_id: &str,
    ) -> Result<Option<Snapshot>, Self::Error>;
}
```

### 関連する型定義

```rust
/// 永続化されたイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedEvent {
    pub event_id: EventId,
    pub stream_id: String,
    pub stream_version: u64,
    pub global_position: u64,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub metadata: EventMetadata,
    pub created_at: DateTime<Utc>,
}

/// イベントメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub user_id: Option<UserId>,
    pub timestamp: DateTime<Utc>,
}

/// スナップショット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub stream_id: String,
    pub version: u64,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// ドメインイベント（各コンテキストから収集）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    // Learning Context
    SessionStarted {
        session_id: SessionId,
        user_id: UserId,
        item_count: u32,
    },
    CorrectAnswerGiven {
        session_id: SessionId,
        item_id: ItemId,
        response_time_ms: u64,
    },
    SessionCompleted {
        session_id: SessionId,
        score: u32,
        duration_seconds: u64,
    },
    
    // Learning Algorithm Context
    ReviewProcessed {
        user_id: UserId,
        item_id: ItemId,
        new_interval: u32,
        easiness_factor: f32,
    },
    
    // User Context
    UserCreated {
        user_id: UserId,
        email: String,
    },
}
```

## Read Model リポジトリ

### UserProgressRepository

ユーザーの学習進捗情報を提供する Read Model リポジトリです。

```rust
/// ユーザー進捗の Read Model
#[async_trait]
pub trait UserProgressRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    /// ユーザーの現在の進捗を取得
    async fn get_current_progress(
        &self,
        user_id: &UserId,
    ) -> Result<Option<UserProgress>, Self::Error>;
    
    /// 進捗を更新（イベントハンドラーから呼ばれる）
    async fn update_progress(
        &self,
        progress: &UserProgress,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub struct UserProgress {
    pub user_id: UserId,
    pub total_items_learned: u64,
    pub total_sessions_completed: u64,
    pub total_study_time_minutes: u64,
    pub current_streak: u32,
    pub accuracy_rate: f64,
    pub mastery_stats: MasteryStats,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MasteryStats {
    pub unlearned: u64,
    pub learning: u64,
    pub short_term_mastered: u64,
    pub long_term_mastered: u64,
}
```

### DailyStatsRepository

日次統計情報を提供する Read Model リポジトリです。

```rust
/// 日次統計の Read Model
#[async_trait]
pub trait DailyStatsRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    /// 特定日の統計を取得
    async fn get_daily_stats(
        &self,
        user_id: &UserId,
        date: chrono::NaiveDate,
    ) -> Result<Option<DailyStats>, Self::Error>;
    
    /// 期間の統計を取得
    async fn get_stats_range(
        &self,
        user_id: &UserId,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<DailyStats>, Self::Error>;
    
    /// 統計を更新（イベントハンドラーから呼ばれる）
    async fn update_daily_stats(
        &self,
        stats: &DailyStats,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub struct DailyStats {
    pub user_id: UserId,
    pub date: chrono::NaiveDate,
    pub sessions_completed: u32,
    pub items_learned: u32,
    pub items_reviewed: u32,
    pub study_time_minutes: u32,
    pub accuracy_rate: f64,
    pub new_mastery_count: u32,
}
```

## イベントプロジェクション

### プロジェクションの実装例

```rust
use tokio::sync::mpsc;

/// イベントプロジェクションマネージャー
pub struct ProjectionManager {
    event_store: Arc<dyn EventStore>,
    projections: Vec<Box<dyn Projection>>,
    checkpoint_store: Arc<dyn CheckpointStore>,
}

impl ProjectionManager {
    /// プロジェクションを実行
    pub async fn run(&self) -> Result<()> {
        let last_position = self.checkpoint_store
            .get_last_position("progress-context")
            .await?
            .unwrap_or(0);
        
        // イベントストリームを読み取り
        let mut position = last_position;
        loop {
            let events = self.event_store
                .read_all_events(position, 100)
                .await?;
            
            if events.is_empty() {
                // 新しいイベントを待つ
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            
            // 各プロジェクションにイベントを適用
            for event in &events {
                for projection in &self.projections {
                    projection.handle(event).await?;
                }
                position = event.global_position + 1;
            }
            
            // チェックポイントを保存
            self.checkpoint_store
                .save_position("progress-context", position)
                .await?;
        }
    }
}

/// プロジェクションインターフェース
#[async_trait]
pub trait Projection: Send + Sync {
    async fn handle(&self, event: &PersistedEvent) -> Result<()>;
}
```

### UserProgress プロジェクションの実装例

```rust
pub struct UserProgressProjection {
    repository: Arc<dyn UserProgressRepository>,
}

#[async_trait]
impl Projection for UserProgressProjection {
    async fn handle(&self, event: &PersistedEvent) -> Result<()> {
        let domain_event: DomainEvent = serde_json::from_value(event.event_data.clone())?;
        
        match domain_event {
            DomainEvent::SessionCompleted { session_id, score, duration_seconds } => {
                // ユーザー ID をメタデータから取得
                let user_id = event.metadata.user_id.as_ref()
                    .ok_or(ProjectionError::MissingUserId)?;
                
                // 現在の進捗を取得または新規作成
                let mut progress = self.repository
                    .get_current_progress(user_id)
                    .await?
                    .unwrap_or_else(|| UserProgress::new(user_id.clone()));
                
                // 統計を更新
                progress.total_sessions_completed += 1;
                progress.total_study_time_minutes += duration_seconds / 60;
                progress.last_updated = Utc::now();
                
                // 保存
                self.repository.update_progress(&progress).await?;
            }
            
            DomainEvent::CorrectAnswerGiven { item_id, .. } => {
                // 正答率の更新など
            }
            
            _ => {} // 他のイベントは無視
        }
        
        Ok(())
    }
}
```

## 実装上の考慮事項

### 1. イベントストアの実装

```rust
// PostgreSQL での実装例
CREATE TABLE event_store (
    event_id UUID PRIMARY KEY,
    stream_id TEXT NOT NULL,
    stream_version BIGINT NOT NULL,
    global_position BIGSERIAL,
    event_type TEXT NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(stream_id, stream_version)
);

-- インデックス
CREATE INDEX idx_global_position ON event_store(global_position);
CREATE INDEX idx_stream_id ON event_store(stream_id);
CREATE INDEX idx_event_type_created ON event_store(event_type, created_at);
```

### 2. スナップショット戦略

```rust
/// スナップショットポリシー
pub struct SnapshotPolicy {
    /// 何イベントごとにスナップショットを作成するか
    pub event_interval: u64,
    
    /// スナップショットの保持数
    pub max_snapshots: u32,
}

impl Default for SnapshotPolicy {
    fn default() -> Self {
        Self {
            event_interval: 100,  // 100イベントごと
            max_snapshots: 3,     // 最新3つ保持
        }
    }
}
```

### 3. エラーハンドリング

```rust
/// Progress Context 固有のリポジトリエラー
#[derive(Debug, thiserror::Error)]
pub enum ProgressRepositoryError {
    #[error("Event store error: {0}")]
    EventStore(String),
    
    #[error("Projection error: {0}")]
    Projection(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Checkpoint error: {0}")]
    Checkpoint(String),
    
    #[error("Consistency error: {0}")]
    Consistency(String),
}
```

### 4. 結果整合性の管理

```rust
/// プロジェクションの遅延監視
pub struct ProjectionMonitor {
    max_lag_seconds: u64,
}

impl ProjectionMonitor {
    pub async fn check_lag(&self, projection_name: &str) -> Result<LagStatus> {
        let current_time = Utc::now();
        let last_processed = self.get_last_processed_time(projection_name).await?;
        
        let lag = current_time - last_processed;
        
        if lag.num_seconds() > self.max_lag_seconds as i64 {
            return Ok(LagStatus::Critical(lag));
        }
        
        Ok(LagStatus::Normal(lag))
    }
}

#[derive(Debug)]
pub enum LagStatus {
    Normal(chrono::Duration),
    Critical(chrono::Duration),
}
```

## 更新履歴

- 2025-07-28: 初版作成（イベントソーシングと CQRS Read Model の設計）
- 2025-07-29: MVP 向けに簡潔化（ランキング、バッジ、項目統計、週間・月間サマリーを削除、ストリークは current_streak のみ残す）
