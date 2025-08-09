# AI Integration Context - API スキーマ

## 概要

AI Integration Context は gRPC による内部通信と WebSocket/SSE によるリアルタイム通知を提供します。外部 AI サービスの API は Anti-Corruption Layer で抽象化されます。

## Protocol Buffers 定義

### 基本型定義

```protobuf
syntax = "proto3";

package ai_integration;

import "google/protobuf/timestamp.proto";
import "google/protobuf/duration.proto";

// タスクタイプ
enum TaskType {
  TASK_TYPE_UNSPECIFIED = 0;
  TASK_TYPE_TEXT_GENERATION = 1;
  TASK_TYPE_IMAGE_GENERATION = 2;
  TASK_TYPE_CHAT_COMPLETION = 3;
  TASK_TYPE_TEST_CUSTOMIZATION = 4;
}

// タスクステータス
enum TaskStatus {
  TASK_STATUS_UNSPECIFIED = 0;
  TASK_STATUS_PENDING = 1;
  TASK_STATUS_PROCESSING = 2;
  TASK_STATUS_COMPLETED = 3;
  TASK_STATUS_FAILED = 4;
  TASK_STATUS_CANCELLED = 5;
}

// プロバイダー
enum Provider {
  PROVIDER_UNSPECIFIED = 0;
  PROVIDER_GEMINI = 1;
  PROVIDER_OPENAI = 2;
  PROVIDER_CLAUDE = 3;
  PROVIDER_UNSPLASH = 4;
}

// 優先度
enum Priority {
  PRIORITY_LOW = 0;
  PRIORITY_NORMAL = 1;
  PRIORITY_HIGH = 2;
  PRIORITY_URGENT = 3;
}
```

### メッセージ型

```protobuf
// タスク情報
message Task {
  string task_id = 1;
  TaskType task_type = 2;
  TaskStatus status = 3;
  string requested_by = 4;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp started_at = 6;
  google.protobuf.Timestamp completed_at = 7;
  map<string, string> metadata = 8;
}

// 生成リクエスト
message GenerationRequest {
  TaskType task_type = 1;
  string content = 2;
  Priority priority = 3;
  map<string, string> parameters = 4;
}

// 生成結果
message GenerationResult {
  string task_id = 1;
  bool success = 2;
  string content = 3;
  Provider provider_used = 4;
  int32 tokens_used = 5;
  double cost_estimate = 6;
  google.protobuf.Duration processing_time = 7;
}

// チャットメッセージ
message ChatMessage {
  string message_id = 1;
  string role = 2;  // "user" or "assistant"
  string content = 3;
  google.protobuf.Timestamp timestamp = 4;
  int32 tokens_used = 5;
}

// プロバイダー状態
message ProviderStatus {
  Provider provider = 1;
  bool available = 2;
  string circuit_breaker_state = 3;
  double success_rate = 4;
  int32 rate_limit_remaining = 5;
}
```

## サービス定義

### AIIntegrationService

```protobuf
service AIIntegrationService {
  // コマンド
  rpc CreateTask(CreateTaskRequest) returns (CreateTaskResponse);
  rpc CancelTask(CancelTaskRequest) returns (CancelTaskResponse);
  rpc RetryTask(RetryTaskRequest) returns (RetryTaskResponse);
  
  // チャット
  rpc StartChatSession(StartChatRequest) returns (StartChatResponse);
  rpc SendChatMessage(ChatMessageRequest) returns (ChatMessageResponse);
  rpc CloseChatSession(CloseSessionRequest) returns (CloseSessionResponse);
  
  // クエリ
  rpc GetTaskStatus(GetTaskStatusRequest) returns (GetTaskStatusResponse);
  rpc GetTaskResult(GetTaskResultRequest) returns (GetTaskResultResponse);
  rpc GetQueueStatus(GetQueueStatusRequest) returns (GetQueueStatusResponse);
  rpc GetProviderStatus(GetProviderStatusRequest) returns (GetProviderStatusResponse);
  rpc GetUsageStats(GetUsageStatsRequest) returns (GetUsageStatsResponse);
  
  // ストリーミング
  rpc SubscribeToTaskUpdates(SubscribeRequest) returns (stream TaskUpdate);
}
```

## リクエスト/レスポンス定義

### CreateTask

**リクエスト**:

```protobuf
message CreateTaskRequest {
  TaskType task_type = 1;
  string content = 2;
  string requested_by = 3;
  Priority priority = 4;
  map<string, string> metadata = 5;
}
```

**レスポンス**:

```protobuf
message CreateTaskResponse {
  string task_id = 1;
  TaskStatus initial_status = 2;
  int32 queue_position = 3;
  google.protobuf.Duration estimated_wait_time = 4;
}
```

### GetTaskStatus

**リクエスト**:

```protobuf
message GetTaskStatusRequest {
  string task_id = 1;
}
```

**レスポンス**:

```protobuf
message GetTaskStatusResponse {
  Task task = 1;
  int32 progress_percentage = 2;
  google.protobuf.Timestamp estimated_completion = 3;
  string current_step = 4;
}
```

## WebSocket/SSE イベント

### リアルタイム通知フォーマット

```json
{
  "event_type": "task_progress",
  "task_id": "task_123",
  "timestamp": "2025-08-09T10:00:00Z",
  "data": {
    "status": "processing",
    "progress": 50,
    "message": "Generating content..."
  }
}
```

### イベントタイプ

| イベント | 説明 | データ |
|---------|------|--------|
| task_created | タスク作成 | task_id, queue_position |
| task_started | 処理開始 | task_id, provider |
| task_progress | 進捗更新 | task_id, progress |
| task_completed | 完了 | task_id, result |
| task_failed | 失敗 | task_id, error |
| chat_message | チャット応答 | session_id, message |

## プロバイダー API 抽象化

### 統一インターフェース

各プロバイダーの差異を吸収する共通インターフェース：

```rust
// 概念的な例（実装の指針）
trait AIProvider {
    fn generate_text(&self, prompt: String) -> Result<String>;
    fn generate_image(&self, description: String) -> Result<ImageUrl>;
    fn chat_completion(&self, messages: Vec<Message>) -> Result<String>;
}
```

### プロバイダー別設定

**Gemini (第一選択)**:

- モデル: gemini-pro, gemini-pro-vision
- 特徴: 高速、コスト効率的

**OpenAI (第二選択)**:

- モデル: gpt-5, dall-e-3
- 特徴: 最新モデル、高品質

**Claude (第三選択)**:

- モデル: claude-3-opus
- 特徴: 長文コンテキスト

**画像素材サービス**:

- Unsplash API: 無料、高品質写真
- その他の無料サービス

## エラー定義

```protobuf
enum ErrorCode {
  ERROR_NONE = 0;
  ERROR_INVALID_REQUEST = 1;
  ERROR_RATE_LIMIT = 2;
  ERROR_TIMEOUT = 3;
  ERROR_PROVIDER_ERROR = 4;
  ERROR_INSUFFICIENT_CREDITS = 5;
  ERROR_CONTENT_FILTERED = 6;
  ERROR_CIRCUIT_BREAKER_OPEN = 7;
}

message ErrorDetail {
  ErrorCode code = 1;
  string message = 2;
  bool retryable = 3;
  google.protobuf.Duration retry_after = 4;
}
```

## 使用例

### タスク作成と結果取得

```rust
// Vocabulary Context からの呼び出し例
use ai_integration::{
    ai_integration_service_client::AIIntegrationServiceClient,
    CreateTaskRequest, TaskType,
};

// タスク作成
let request = CreateTaskRequest {
    task_type: TaskType::TextGeneration as i32,
    content: "Generate detailed information for vocabulary item: 'ephemeral'".into(),
    requested_by: "vocabulary-context".into(),
    priority: Priority::Normal as i32,
    metadata: HashMap::new(),
};

let response = client.create_task(request).await?;
let task_id = response.get_ref().task_id.clone();

// WebSocket で進捗を監視
// ...

// 結果取得
let result = client.get_task_result(GetTaskResultRequest {
    task_id: task_id.clone(),
}).await?;
```

## パフォーマンス要件

| RPC メソッド | レスポンスタイム | スループット |
|-------------|----------------|--------------|
| CreateTask | <100ms | 1000 req/s |
| GetTaskStatus | <50ms | 2000 req/s |
| GetTaskResult | <100ms | 1000 req/s |
| WebSocket通知 | <500ms | - |

## セキュリティ

### 認証・認可

- 内部サービス間: mTLS
- WebSocket: JWT トークン
- API キー: Secret Manager で管理

### データ保護

- PII の検出と除去
- リクエスト/レスポンスのサニタイズ
- 監査ログの記録
