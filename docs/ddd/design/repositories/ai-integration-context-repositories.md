# AI Integration Context - リポジトリインターフェース

## 概要

AI Integration Context には 2 つの主要な集約が存在します：

- `AIGenerationTask`：AI による各種生成タスクの管理
- `ChatSession`：深掘りチャット機能のセッション管理

このコンテキストは Anti-Corruption Layer パターンを実装し、外部 AI サービスとの統合を担当します。

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

// ChatSession
// - (user_id, item_id) - UNIQUE, ユーザー×項目のセッション
// - (user_id, last_activity) - アクティブセッション検索
// - (status, last_activity) - クリーンアップ用
// - (message_count DESC) - 大量メッセージセッション
```

### 2. トランザクション境界

```rust
// チャットメッセージ追加の例
pub async fn add_chat_message(
    repo: &dyn ChatSessionRepository,
    session_id: ChatSessionId,
    message: String,
) -> Result<ChatResponse> {
    // 1. セッションの更新（トランザクション1）
    let mut session = repo.find_by_id(&session_id).await?
        .ok_or(DomainError::NotFound)?;
    
    // ユーザーメッセージを追加
    session.add_user_message(message.clone())?;
    repo.save(&session).await?;
    
    // 2. AI レスポンスの生成（トランザクション外）
    let ai_response = generate_ai_response(&session).await?;
    
    // 3. AI レスポンスの保存（トランザクション2）
    session.add_assistant_message(ai_response.content.clone())?;
    repo.save(&session).await?;
    
    Ok(ai_response)
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
    
    #[error("Rate limit exceeded for user {0}")]
    RateLimitExceeded(UserId),
    
    #[error("Maximum message count reached for session {0}")]
    MaxMessagesReached(ChatSessionId),
    
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),
    
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
