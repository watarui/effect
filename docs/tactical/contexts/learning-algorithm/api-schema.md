# Learning Algorithm Context - API スキーマ

## 概要

Learning Algorithm Context の内部 gRPC API スキーマ定義です。他のコンテキスト（主に Learning Context）からの呼び出しに特化した設計となっています。すべてのコンテキスト間通信は gRPC で行われます。

## Protocol Buffers 定義

### 基本型定義

```protobuf
syntax = "proto3";

package learning_algorithm;

import "google/protobuf/timestamp.proto";
import "google/protobuf/duration.proto";

// 品質評価
enum Quality {
  QUALITY_BLACKOUT = 0;      // 完全に忘れた
  QUALITY_INCORRECT = 1;      // 不正解
  QUALITY_HARD = 2;           // 正解だが困難
  QUALITY_GOOD = 3;           // 通常の正解
  QUALITY_EASY = 4;           // 簡単に正解
  QUALITY_PERFECT = 5;        // 即座に正解
}

// 復習タイプ
enum ReviewType {
  REVIEW_TYPE_NEW = 0;        // 新規項目
  REVIEW_TYPE_LEARNING = 1;   // 学習中
  REVIEW_TYPE_REVIEW = 2;     // 通常復習
  REVIEW_TYPE_OVERDUE = 3;    // 期限切れ
}

// 難易度レベル
enum DifficultyLevel {
  DIFFICULTY_EASY = 0;
  DIFFICULTY_MEDIUM = 1;
  DIFFICULTY_HARD = 2;
}

// 調整方向
enum AdjustmentDirection {
  DIRECTION_DECREASE = 0;
  DIRECTION_MAINTAIN = 1;
  DIRECTION_INCREASE = 2;
}
```

### メッセージ型

```protobuf
// 学習項目
message LearningItem {
  string item_id = 1;
  int32 priority = 2;
  ReviewType review_type = 3;
  double easiness_factor = 4;
  google.protobuf.Timestamp last_review_date = 5;
  int32 overdue_days = 6;
}

// セッション設定
message SessionConfig {
  int32 item_count = 1;
  double new_item_ratio = 2;
  string difficulty_variance = 3;
  int32 time_limit_minutes = 4;
}

// パフォーマンス指標
message PerformanceMetrics {
  double accuracy_rate = 1;
  double average_quality = 2;
  int32 total_reviews = 3;
  int32 mastered_items = 4;
  double learning_velocity = 5;
  double retention_rate = 6;
}

// スケジュール項目
message ScheduleItem {
  google.protobuf.Timestamp date = 1;
  int32 item_count = 2;
  int32 review_items = 3;
  int32 new_items = 4;
  int32 estimated_minutes = 5;
}
```

## サービス定義

### LearningAlgorithmService

```protobuf
service LearningAlgorithmService {
  // コマンド
  rpc RecordReview(RecordReviewRequest) returns (RecordReviewResponse);
  rpc SelectItems(SelectItemsRequest) returns (SelectItemsResponse);
  rpc AnalyzePerformance(AnalyzePerformanceRequest) returns (AnalyzePerformanceResponse);
  rpc AdjustDifficulty(AdjustDifficultyRequest) returns (AdjustDifficultyResponse);
  
  // クエリ
  rpc GetNextItems(GetNextItemsRequest) returns (GetNextItemsResponse);
  rpc GetReviewSchedule(GetReviewScheduleRequest) returns (GetReviewScheduleResponse);
  rpc GetPerformanceStats(GetPerformanceStatsRequest) returns (GetPerformanceStatsResponse);
  rpc GetLearningProgress(GetLearningProgressRequest) returns (GetLearningProgressResponse);
}
```

## リクエスト/レスポンス定義

### RecordReview

**リクエスト**:

```protobuf
message RecordReviewRequest {
  string user_id = 1;
  string item_id = 2;
  bool is_correct = 3;
  int32 response_time_ms = 4;
  map<string, string> review_context = 5;
}
```

**レスポンス**:

```protobuf
message RecordReviewResponse {
  Quality calculated_quality = 1;
  double new_easiness_factor = 2;
  int32 interval_days = 3;
  google.protobuf.Timestamp next_review_date = 4;
}
```

### SelectItems

**リクエスト**:

```protobuf
message SelectItemsRequest {
  string user_id = 1;
  SessionConfig session_config = 2;
}
```

**レスポンス**:

```protobuf
message SelectItemsResponse {
  repeated LearningItem items = 1;
  repeated string recommended_order = 2;
  int32 estimated_duration_minutes = 3;
}
```

### GetPerformanceStats

**リクエスト**:

```protobuf
message GetPerformanceStatsRequest {
  string user_id = 1;
  string period = 2; // "7d", "30d", "90d", "all"
  repeated string metrics = 3;
}
```

**レスポンス**:

```protobuf
message GetPerformanceStatsResponse {
  PerformanceMetrics overall_stats = 1;
  map<string, double> trend_data = 2;
  int32 performance_score = 3;
  repeated string recommendations = 4;
}
```

## イベント定義

### Pub/Sub メッセージ

```protobuf
// 復習記録イベント
message ReviewRecordedEvent {
  string event_id = 1;
  google.protobuf.Timestamp occurred_at = 2;
  string user_id = 3;
  string item_id = 4;
  Quality quality = 5;
  int32 response_time_ms = 6;
  bool is_correct = 7;
}

// スケジュール更新イベント
message ReviewScheduledEvent {
  string event_id = 1;
  google.protobuf.Timestamp occurred_at = 2;
  string user_id = 3;
  string item_id = 4;
  google.protobuf.Timestamp next_review_date = 5;
  int32 interval_days = 6;
}

// 難易度調整イベント
message DifficultyAdjustedEvent {
  string event_id = 1;
  google.protobuf.Timestamp occurred_at = 2;
  string user_id = 3;
  double old_easiness_factor = 4;
  double new_easiness_factor = 5;
  string adjustment_reason = 6;
}
```

## エラー定義

```protobuf
// エラーコード
enum ErrorCode {
  ERROR_NONE = 0;
  ERROR_RECORD_NOT_FOUND = 1;
  ERROR_INVALID_QUALITY = 2;
  ERROR_INSUFFICIENT_DATA = 3;
  ERROR_INVALID_CONFIGURATION = 4;
}

// エラーレスポンス
message ErrorResponse {
  ErrorCode code = 1;
  string message = 2;
  map<string, string> details = 3;
}
```

## 使用例

### Learning Context からの呼び出し例

#### 復習結果の記録

```rust
// Learning Context 内のコード例
use tonic::Request;
use learning_algorithm::{
    learning_algorithm_service_client::LearningAlgorithmServiceClient,
    RecordReviewRequest,
};

// Learning Algorithm Context への gRPC クライアント接続
let mut client = LearningAlgorithmServiceClient::connect(
    "http://learning-algorithm:50051"
).await?;

// 復習結果を Learning Algorithm Context に送信
let request = Request::new(RecordReviewRequest {
    user_id: "user123".into(),
    item_id: "item456".into(),
    is_correct: true,
    response_time_ms: 2500,
    review_context: HashMap::new(),
});

// Learning Algorithm Context の RecordReview RPC を呼び出し
let response = client.record_review(request).await?;
println!("Next review in {} days", response.get_ref().interval_days);
```

#### 項目選定

```rust
// Learning Context が学習セッション開始時に呼び出す
let request = Request::new(SelectItemsRequest {
    user_id: "user123".into(),
    session_config: Some(SessionConfig {
        item_count: 20,
        new_item_ratio: 0.2,
        difficulty_variance: "medium".into(),
        time_limit_minutes: 0,
    }),
});

// Learning Algorithm Context から最適な項目リストを取得
let response = client.select_items(request).await?;
for item in &response.get_ref().items {
    println!("Item {}: priority {}", item.item_id, item.priority);
}
```

## パフォーマンス要件

| RPC メソッド | レスポンスタイム | スループット |
|-------------|----------------|--------------|
| RecordReview | <50ms | 1000 req/s |
| SelectItems | <100ms | 500 req/s |
| GetPerformanceStats | <500ms | 100 req/s |
| その他のクエリ | <200ms | 200 req/s |

## セキュリティ

### 認証・認可

- 内部サービス間通信のみ（外部公開なし）
- mTLS による相互認証
- サービスアカウントによる認可

### データ保護

- ユーザーIDによるデータ分離
- 個人情報は最小限
- 監査ログの記録

## バージョニング

### 互換性の維持

- 後方互換性の保証
- フィールドの追加は可能
- フィールドの削除は非推奨化後に実施
- 新バージョンは別エンドポイント

### 現在のバージョン

- v1: 現在の安定版（SM-2 アルゴリズム）
- v2: 計画中（FSRS アルゴリズム対応）
