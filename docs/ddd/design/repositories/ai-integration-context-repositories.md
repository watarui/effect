# AI Integration Context - リポジトリインターフェース

## 概要

AI Integration Context には 3 つの主要な集約が存在します：

- `AIGenerationTask`：AI による各種生成タスクの管理
- `ChatSession`：深掘りチャット機能のセッション管理
- `TaskQueue`：非同期タスクキューの管理

このコンテキストは Anti-Corruption Layer パターンを実装し、外部 AI サービスとの統合を担当します。
完全非同期処理により、大量の AI 要求を効率的に処理し、WebSocket/SSE によるリアルタイム通知を提供します。

## AIGenerationTaskRepository

AI 生成タスクの永続化を担当するリポジトリです。

### インターフェース定義

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// AI 生成タスクのリポジトリ
#[async_trait]
pub trait AIGenerationTaskRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    // ===== 基本的な CRUD 操作 =====
    
    /// ID でタスクを取得
    async fn find_by_id(&self, id: &TaskId) -> Result<Option<AIGenerationTask>, Self::Error>;
    
    /// タスクを保存（新規作成または更新）
    async fn save(&self, task: &AIGenerationTask) -> Result<(), Self::Error>;
    
    /// タスクを削除（通常は論理削除）
    async fn delete(&self, id: &TaskId) -> Result<(), Self::Error>;
    
    // ===== ステータス別クエリ =====
    
    /// 未処理のタスクを取得
    async fn find_pending_tasks(
        &self,
        limit: u32,
    ) -> Result<Vec<AIGenerationTask>, Self::Error>;
    
    /// 処理中のタスクを取得
    async fn find_processing_tasks(
        &self,
        provider: Option<&str>,
    ) -> Result<Vec<AIGenerationTask>, Self::Error>;
    
    /// 失敗したタスクを取得（リトライ用）
    async fn find_failed_tasks(
        &self,
        max_retries: u32,
        older_than: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<AIGenerationTask>, Self::Error>;
    
    /// タイムアウトしたタスクを取得
    async fn find_timed_out_tasks(
        &self,
        timeout_duration: chrono::Duration,
    ) -> Result<Vec<AIGenerationTask>, Self::Error>;
    
    // ===== ユーザー関連クエリ =====
    
    /// ユーザーのタスクを取得
    async fn find_by_user(
        &self,
        user_id: &UserId,
        page_request: &PageRequest,
    ) -> Result<Page<AIGenerationTask>, Self::Error>;
    
    /// ユーザーのアクティブなタスク数を取得
    async fn count_active_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<u64, Self::Error>;
    
    // ===== タスクタイプ別クエリ =====
    
    /// タスクタイプ別に取得
    async fn find_by_type(
        &self,
        task_type: &TaskType,
        status: Option<&TaskStatus>,
        limit: u32,
    ) -> Result<Vec<AIGenerationTask>, Self::Error>;
    
    /// 関連エンティティで取得
    async fn find_by_related_entity(
        &self,
        entity_type: &RelatedEntityType,
        entity_id: &str,
    ) -> Result<Vec<AIGenerationTask>, Self::Error>;
    
    // ===== キューイング関連 =====
    
    /// 次に処理すべきタスクを取得（キューから取り出し）
    async fn dequeue_next_task(
        &self,
        worker_id: &WorkerId,
    ) -> Result<Option<AIGenerationTask>, Self::Error>;
    
    /// タスクをキューに戻す（処理失敗時）
    async fn requeue_task(
        &self,
        task_id: &TaskId,
        delay: Option<chrono::Duration>,
    ) -> Result<(), Self::Error>;
    
    /// ワーカーが処理中のタスクを取得
    async fn find_tasks_by_worker(
        &self,
        worker_id: &WorkerId,
    ) -> Result<Vec<AIGenerationTask>, Self::Error>;
}
```

### 使用例

```rust
// アプリケーションサービスでの使用例
pub struct ProcessAITaskUseCase<R: AIGenerationTaskRepository> {
    repository: Arc<R>,
    ai_service: Arc<dyn AIServiceAdapter>,
    event_bus: Arc<dyn EventBus>,
}

impl<R: AIGenerationTaskRepository> ProcessAITaskUseCase<R> {
    pub async fn execute(&self, task_id: TaskId) -> Result<()> {
        // タスクを取得
        let mut task = self.repository
            .find_by_id(&task_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        
        // 処理中に更新
        task.start_processing();
        self.repository.save(&task).await?;
        
        // AI サービスを呼び出し
        let result = match &task.task_type() {
            TaskType::ItemInfoGeneration { item_id, .. } => {
                self.ai_service.generate_item_info(item_id).await
            }
            TaskType::ImageGeneration { description, .. } => {
                self.ai_service.generate_image(description).await
            }
            _ => return Err(DomainError::UnsupportedTaskType),
        };
        
        // 結果を保存
        match result {
            Ok(response) => {
                task.complete_with_success(response);
                self.event_bus.publish(TaskCompleted {
                    task_id: task.id().clone(),
                    result: task.response().clone(),
                }).await?;
            }
            Err(e) => {
                task.fail_with_error(e.to_string());
                if task.should_retry() {
                    self.event_bus.publish(TaskRetryScheduled {
                        task_id: task.id().clone(),
                        retry_count: task.retry_count(),
                    }).await?;
                }
            }
        }
        
        self.repository.save(&task).await?;
        Ok(())
    }
}
```

## ChatSessionRepository

深掘りチャットセッションを管理するリポジトリです。

### インターフェース定義

```rust
/// チャットセッションのリポジトリ
#[async_trait]
pub trait ChatSessionRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    // ===== 基本的な CRUD 操作 =====
    
    /// ID でセッションを取得
    async fn find_by_id(&self, id: &ChatSessionId) -> Result<Option<ChatSession>, Self::Error>;
    
    /// セッションを保存（新規作成または更新）
    async fn save(&self, session: &ChatSession) -> Result<(), Self::Error>;
    
    /// セッションを削除（論理削除推奨）
    async fn delete(&self, id: &ChatSessionId) -> Result<(), Self::Error>;
    
    // ===== ユーザー関連クエリ =====
    
    /// ユーザーのアクティブなセッションを取得
    async fn find_active_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<ChatSession>, Self::Error>;
    
    /// ユーザーと項目のセッションを取得
    async fn find_by_user_and_item(
        &self,
        user_id: &UserId,
        item_id: &ItemId,
    ) -> Result<Option<ChatSession>, Self::Error>;
    
    /// ユーザーのセッション履歴を取得
    async fn find_by_user_paginated(
        &self,
        user_id: &UserId,
        page_request: &PageRequest,
    ) -> Result<Page<ChatSession>, Self::Error>;
}
```

### 補助的な型定義

```rust
/// タスクタイプ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskType {
    ItemInfoGeneration {
        item_id: ItemId,
        entry_id: EntryId,
    },
    TestCustomization {
        session_id: SessionId,
        instruction: String,
    },
    ImageGeneration {
        item_id: ItemId,
        description: String,
        style: ImageStyle,
    },
}

/// タスクステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// 関連エンティティタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelatedEntityType {
    VocabularyItem,
    VocabularyEntry,
    LearningSession,
}

```

## TaskQueueRepository

非同期タスクキューの管理を担当するリポジトリです。

### インターフェース定義

```rust
/// タスクキューのリポジトリ
#[async_trait]
pub trait TaskQueueRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    // ===== 基本的な CRUD 操作 =====
    
    /// ID でキューを取得
    async fn find_by_id(&self, id: &QueueId) -> Result<Option<TaskQueue>, Self::Error>;
    
    /// キューを保存（新規作成または更新）
    async fn save(&self, queue: &TaskQueue) -> Result<(), Self::Error>;
    
    /// キューを削除
    async fn delete(&self, id: &QueueId) -> Result<(), Self::Error>;
    
    // ===== キュー管理 =====
    
    /// タイプ別にキューを取得
    async fn find_by_type(
        &self,
        queue_type: &QueueType,
    ) -> Result<Vec<TaskQueue>, Self::Error>;
    
    /// アクティブなキューを全て取得
    async fn find_active_queues(&self) -> Result<Vec<TaskQueue>, Self::Error>;
    
    /// キューの統計情報を取得
    async fn get_queue_stats(
        &self,
        queue_id: &QueueId,
    ) -> Result<QueueStats, Self::Error>;
    
    // ===== タスク操作 =====
    
    /// タスクをキューに追加
    async fn enqueue_task(
        &self,
        queue_id: &QueueId,
        task_id: &TaskId,
        priority: TaskPriority,
    ) -> Result<(), Self::Error>;
    
    /// 優先度付きでタスクを取得
    async fn dequeue_task_with_priority(
        &self,
        queue_id: &QueueId,
        worker_id: &WorkerId,
    ) -> Result<Option<QueuedTask>, Self::Error>;
    
    /// 期限切れタスクを取得
    async fn find_expired_tasks(
        &self,
        queue_id: &QueueId,
        expiry_duration: chrono::Duration,
    ) -> Result<Vec<QueuedTask>, Self::Error>;
    
    // ===== ワーカー管理 =====
    
    /// ワーカーを登録
    async fn register_worker(
        &self,
        worker_id: &WorkerId,
        queue_id: &QueueId,
    ) -> Result<(), Self::Error>;
    
    /// ワーカーのハートビートを更新
    async fn update_worker_heartbeat(
        &self,
        worker_id: &WorkerId,
    ) -> Result<(), Self::Error>;
    
    /// 無応答のワーカーを検出
    async fn find_stale_workers(
        &self,
        timeout: chrono::Duration,
    ) -> Result<Vec<WorkerId>, Self::Error>;
}
```

### 補助的な型定義（TaskQueue 関連）

```rust
/// キュータイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueType {
    Standard,      // FIFO
    Priority,      // 優先度付き
    RateLimited,   // レート制限付き
}

/// タスク優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    High = 3,
    Normal = 2,
    Low = 1,
}

/// キューに入っているタスク
#[derive(Debug, Clone)]
pub struct QueuedTask {
    pub task_id: TaskId,
    pub priority: TaskPriority,
    pub queued_at: DateTime<Utc>,
    pub attempts: u32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub assigned_worker: Option<WorkerId>,
}

/// キューの統計情報
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub pending_count: u64,
    pub processing_count: u64,
    pub completed_count: u64,
    pub failed_count: u64,
    pub average_wait_time_ms: u64,
    pub average_processing_time_ms: u64,
}

/// ワーカー ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkerId(String);
```

## NotificationRepository

リアルタイム通知の管理を担当するリポジトリです。

### インターフェース定義

```rust
/// 通知管理のリポジトリ
#[async_trait]
pub trait NotificationRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    // ===== WebSocket/SSE セッション管理 =====
    
    /// 通知セッションを登録
    async fn register_session(
        &self,
        session_id: &NotificationSessionId,
        user_id: &UserId,
        connection_type: ConnectionType,
    ) -> Result<(), Self::Error>;
    
    /// セッションを削除
    async fn unregister_session(
        &self,
        session_id: &NotificationSessionId,
    ) -> Result<(), Self::Error>;
    
    /// ユーザーのアクティブなセッションを取得
    async fn find_active_sessions_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<NotificationSession>, Self::Error>;
    
    // ===== 通知履歴 =====
    
    /// 通知を保存
    async fn save_notification(
        &self,
        notification: &TaskNotification,
    ) -> Result<(), Self::Error>;
    
    /// ユーザーの通知履歴を取得
    async fn find_notifications_by_user(
        &self,
        user_id: &UserId,
        since: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<TaskNotification>, Self::Error>;
    
    /// 未配信の通知を取得
    async fn find_undelivered_notifications(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<TaskNotification>, Self::Error>;
    
    /// 通知を配信済みとしてマーク
    async fn mark_as_delivered(
        &self,
        notification_id: &NotificationId,
        session_id: &NotificationSessionId,
    ) -> Result<(), Self::Error>;
}
```

### 補助的な型定義（通知関連）

```rust
/// 接続タイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    WebSocket,
    ServerSentEvents,
}

/// 通知セッション
#[derive(Debug, Clone)]
pub struct NotificationSession {
    pub session_id: NotificationSessionId,
    pub user_id: UserId,
    pub connection_type: ConnectionType,
    pub connected_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

/// タスク通知
#[derive(Debug, Clone)]
pub struct TaskNotification {
    pub notification_id: NotificationId,
    pub task_id: TaskId,
    pub user_id: UserId,
    pub event_type: TaskEventType,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

/// タスクイベントタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskEventType {
    TaskCreated,
    TaskStarted,
    TaskProgress,
    TaskCompleted,
    TaskFailed,
    TaskRetrying,
}
```

## 実装上の考慮事項

### 1. パフォーマンス最適化

```rust
// インデックスの推奨
// AIGenerationTask
// - (status, created_at) - 未処理タスクの取得
// - (user_id, created_at) - ユーザー別タスク
// - (task_type, status) - タイプ別フィルタリング
// - (retry_count, status, updated_at) - リトライ候補
// - (provider, status) - プロバイダー別管理
// - (assigned_worker, status) - ワーカー別タスク管理

// ChatSession
// - (user_id, item_id) - UNIQUE, ユーザー×項目のセッション
// - (user_id, last_activity) - アクティブセッション検索
// - (status, last_activity) - クリーンアップ用
// - (message_count DESC) - 大量メッセージセッション

// TaskQueue
// - (queue_type, status) - キュータイプ別検索
// - (pending_tasks_count DESC) - 負荷分散用

// QueuedTask
// - (queue_id, priority DESC, queued_at) - 優先度付きデキュー
// - (assigned_worker, last_attempt_at) - タイムアウト検出
// - (attempts, queued_at) - リトライ管理

// NotificationSession
// - (user_id, connection_type) - ユーザー別セッション
// - (last_heartbeat) - タイムアウト検出用

// TaskNotification
// - (user_id, created_at DESC) - ユーザー別通知履歴
// - (delivered_at IS NULL, user_id) - 未配信通知検索
```

### 2. トランザクション境界

```rust
// 非同期タスク作成の例
pub async fn create_ai_generation_task(
    task_repo: &dyn AIGenerationTaskRepository,
    queue_repo: &dyn TaskQueueRepository,
    notification_repo: &dyn NotificationRepository,
    request: GenerateItemInfoRequest,
) -> Result<TaskId> {
    // 1. タスクの作成と保存（トランザクション1）
    let task = AIGenerationTask::new(
        TaskType::ItemInfoGeneration {
            item_id: request.item_id,
            entry_id: request.entry_id,
        },
        request.user_id,
    )?;
    let task_id = task.id().clone();
    task_repo.save(&task).await?;
    
    // 2. キューへの追加（トランザクション2）
    let queue = queue_repo.find_by_type(&QueueType::Standard).await?
        .first()
        .ok_or(DomainError::NoAvailableQueue)?;
    queue_repo.enqueue_task(
        queue.id(),
        &task_id,
        TaskPriority::Normal,
    ).await?;
    
    // 3. 通知の作成（トランザクション3）
    let notification = TaskNotification::new(
        task_id.clone(),
        request.user_id,
        TaskEventType::TaskCreated,
        "AI生成タスクを受け付けました".to_string(),
    );
    notification_repo.save_notification(&notification).await?;
    
    // 4. リアルタイム通知の送信（トランザクション外）
    send_realtime_notification(&notification).await?;
    
    Ok(task_id)
}

// ワーカーによるタスク処理の例
pub async fn process_next_task(
    task_repo: &dyn AIGenerationTaskRepository,
    queue_repo: &dyn TaskQueueRepository,
    worker_id: WorkerId,
) -> Result<()> {
    // 1. タスクのデキュー（トランザクション1）
    let queued_task = queue_repo
        .dequeue_task_with_priority(&queue_id, &worker_id)
        .await?
        .ok_or(DomainError::NoTaskAvailable)?;
    
    // 2. タスクステータスの更新（トランザクション2）
    let mut task = task_repo
        .find_by_id(&queued_task.task_id)
        .await?
        .ok_or(DomainError::NotFound)?;
    task.start_processing(worker_id.clone());
    task_repo.save(&task).await?;
    
    // 3. AI 処理の実行（トランザクション外）
    let result = execute_ai_task(&task).await;
    
    // 4. 結果の保存（トランザクション3）
    match result {
        Ok(response) => {
            task.complete_with_success(response);
            task_repo.save(&task).await?;
        }
        Err(e) if e.is_retryable() => {
            task.fail_with_retry(e.to_string());
            task_repo.save(&task).await?;
            // キューに再投入
            queue_repo.requeue_task(
                &task.id(),
                Some(Duration::seconds(60)),
            ).await?;
        }
        Err(e) => {
            task.fail_permanently(e.to_string());
            task_repo.save(&task).await?;
        }
    }
    
    Ok(())
}
```

### 3. エラーハンドリング

```rust
/// AI Integration Context 固有のリポジトリエラー
#[derive(Debug, thiserror::Error)]
pub enum AIRepositoryError {
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),
    
    #[error("Session not found: {0}")]
    SessionNotFound(ChatSessionId),
    
    #[error("Queue not found: {0}")]
    QueueNotFound(QueueId),
    
    #[error("No available queue for type: {0:?}")]
    NoAvailableQueue(QueueType),
    
    #[error("Worker not found: {0}")]
    WorkerNotFound(WorkerId),
    
    #[error("Rate limit exceeded for user {0}")]
    RateLimitExceeded(UserId),
    
    #[error("Maximum message count reached for session {0}")]
    MaxMessagesReached(ChatSessionId),
    
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),
    
    #[error("Task already assigned to worker: {0}")]
    TaskAlreadyAssigned(WorkerId),
    
    #[error("Queue is full: {0}")]
    QueueFull(QueueId),
    
    #[error("Notification session not found: {0}")]
    NotificationSessionNotFound(NotificationSessionId),
    
    #[error("Database error: {0}")]
    Database(String),
}
```

### 4. AI サービスとの統合

```rust
/// AI サービスアダプターインターフェース
#[async_trait]
pub trait AIServiceAdapter: Send + Sync {
    /// 項目情報を生成
    async fn generate_item_info(
        &self,
        item_id: &ItemId,
    ) -> Result<ItemInfoResponse, AIServiceError>;
    
    /// チャットレスポンスを生成
    async fn generate_chat_response(
        &self,
        session: &ChatSession,
        message: &str,
    ) -> Result<ChatResponse, AIServiceError>;
    
    /// 画像を生成
    async fn generate_image(
        &self,
        description: &str,
    ) -> Result<ImageResponse, AIServiceError>;
    
    /// テストをカスタマイズ
    async fn customize_test(
        &self,
        instruction: &str,
        items: &[ItemSummary],
    ) -> Result<CustomizationResponse, AIServiceError>;
}
```

## 更新履歴

- 2025-07-28: 初版作成（Anti-Corruption Layer パターンを含む設計）
- 2025-07-29: MVP 向けに簡潔化（統計、分析、トークン管理、セッション管理機能を削除）
- 2025-07-30: 非同期処理対応（TaskQueueRepository、NotificationRepository を追加、AIGenerationTaskRepository にキューイング機能を追加）
