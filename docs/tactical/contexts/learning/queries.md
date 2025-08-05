# Learning Context - クエリ定義

## 概要

Learning Context で使用されるクエリの定義です。Read Model から学習情報を効率的に取得します。

## クエリ一覧

### セッション関連

| クエリ名 | 説明 | キャッシュ |
|----------|------|-----------|
| GetActiveSession | アクティブなセッションを取得 | なし |
| GetSessionById | ID でセッションを取得 | 5分 |
| GetSessionHistory | セッション履歴を取得 | 5分 |
| GetSessionStats | セッション統計を取得 | 5分 |

### 学習記録関連

| クエリ名 | 説明 | キャッシュ |
|----------|------|-----------|
| GetUserItemRecord | ユーザーの項目学習記録を取得 | 5分 |
| GetMasteryStatus | 習熟状態別の項目を取得 | 5分 |
| GetDueForReview | 復習期限の項目を取得 | なし |
| GetLearningProgress | 学習進捗を取得 | 5分 |

### 分析関連

| クエリ名 | 説明 | キャッシュ |
|----------|------|-----------|
| GetLearningCurve | 学習曲線データを取得 | 1時間 |
| GetAccuracyTrend | 正答率の推移を取得 | 1時間 |
| GetSessionPatterns | セッションパターンを分析 | 1時間 |

## クエリ詳細

### GetActiveSession

ユーザーの現在アクティブなセッションを取得します。

**パラメータ**:

```rust
struct GetActiveSessionQuery {
    user_id: UserId,
}
```

**レスポンス**:

```rust
struct ActiveSessionView {
    session_id: Uuid,
    started_at: DateTime<Utc>,
    total_items: u8,
    completed_items: u8,
    current_item: Option<CurrentItemView>,
    session_type: String,
}

struct CurrentItemView {
    item_id: Uuid,
    spelling: String,
    presented_at: DateTime<Utc>,
    answer_revealed: bool,
}
```

**実装のポイント**:

- キャッシュなし（リアルタイム性重視）
- 軽量なレスポンス

### GetSessionById

セッションの詳細情報を取得します。

**パラメータ**:

```rust
struct GetSessionByIdQuery {
    session_id: SessionId,
}
```

**レスポンス**:

```rust
struct SessionDetailView {
    session_id: Uuid,
    user_id: Uuid,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    session_type: String,
    status: String,
    items: Vec<SessionItemView>,
    summary: SessionSummaryView,
}

struct SessionItemView {
    item_id: Uuid,
    spelling: String,
    presented_at: DateTime<Utc>,
    response_time_ms: Option<u32>,
    judgment: Option<String>,
}

struct SessionSummaryView {
    total_items: u8,
    completed_items: u8,
    correct_count: u8,
    average_response_time_ms: u32,
}
```

### GetSessionHistory

ユーザーの学習セッション履歴を取得します。

**パラメータ**:

```rust
struct GetSessionHistoryQuery {
    user_id: UserId,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
    limit: u32,         // デフォルト: 20
    cursor: Option<String>,  // カーソルベースページネーション
}
```

**レスポンス**:

```rust
struct SessionHistoryView {
    sessions: Vec<SessionSummaryView>,
    has_next_page: bool,
    next_cursor: Option<String>,
}

struct SessionSummaryView {
    session_id: Uuid,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    total_items: u8,
    correct_count: u8,
    accuracy_rate: f32,
}
```

### GetUserItemRecord

ユーザーの特定項目に対する学習記録を取得します。

**パラメータ**:

```rust
struct GetUserItemRecordQuery {
    user_id: UserId,
    item_ids: Vec<ItemId>,
}
```

**レスポンス**:

```rust
struct UserItemRecordView {
    records: Vec<ItemRecordView>,
}

struct ItemRecordView {
    item_id: Uuid,
    spelling: String,
    mastery_status: String,
    total_attempts: u32,
    correct_count: u32,
    last_reviewed: DateTime<Utc>,
    next_review: Option<DateTime<Utc>>,
    response_times: ResponseTimeStats,
}

struct ResponseTimeStats {
    average_ms: u32,
    best_ms: u32,
    recent_trend: String,  // "improving", "stable", "declining"
}
```

### GetMasteryStatus

習熟状態別に項目を取得します。

**パラメータ**:

```rust
struct GetMasteryStatusQuery {
    user_id: UserId,
    status_filter: Vec<MasteryStatus>,
    limit: u32,
    offset: u32,
}
```

**レスポンス**:

```rust
struct MasteryStatusView {
    status_groups: Vec<StatusGroupView>,
    total_counts: MasteryCountsView,
}

struct StatusGroupView {
    status: String,
    items: Vec<ItemSummaryView>,
    count: u32,
}

struct MasteryCountsView {
    unknown: u32,
    searched: u32,
    tested: u32,
    test_failed: u32,
    short_term_mastered: u32,
    long_term_mastered: u32,
}
```

### GetLearningProgress

学習進捗の概要を取得します。

**パラメータ**:

```rust
struct GetLearningProgressQuery {
    user_id: UserId,
    period: ProgressPeriod,
}

enum ProgressPeriod {
    Today,
    ThisWeek,
    ThisMonth,
    AllTime,
}
```

**レスポンス**:

```rust
struct LearningProgressView {
    period: String,
    sessions_completed: u32,
    items_learned: u32,
    items_mastered: u32,
    total_study_time_minutes: u32,
    average_accuracy: f32,
    streak_days: u32,
    daily_progress: Vec<DailyProgressView>,
}

struct DailyProgressView {
    date: NaiveDate,
    sessions: u32,
    items_reviewed: u32,
    accuracy: f32,
    study_time_minutes: u32,
}
```

## キャッシング戦略

### Redis キャッシュの使用

```rust
// キャッシュキーの例
fn cache_key_session(session_id: &Uuid) -> String {
    format!("learning:session:{}", session_id)
}

fn cache_key_user_progress(user_id: &Uuid, period: &str) -> String {
    format!("learning:progress:{}:{}", user_id, period)
}
```

### キャッシュ無効化

- セッション更新時: 該当セッションのキャッシュをクリア
- 項目学習時: ユーザーの進捗キャッシュをクリア
- TTL による自動失効

## パフォーマンス最適化

### インデックス戦略

```sql
-- セッション検索用
CREATE INDEX idx_sessions_user_started ON learning_sessions(user_id, started_at DESC);

-- 習熟状態検索用
CREATE INDEX idx_records_user_status ON user_item_records(user_id, mastery_status);

-- 復習期限検索用
CREATE INDEX idx_records_next_review ON user_item_records(next_review) 
WHERE next_review IS NOT NULL;
```

### クエリ最適化

1. **バッチ取得**: 複数項目の記録を一度に取得
2. **部分取得**: 必要なフィールドのみ選択
3. **非正規化**: Read Model で事前計算した値を保存

## GraphQL との統合

```graphql
type Query {
  # セッション関連
  activeSession: ActiveSession
  session(id: ID!): SessionDetail
  sessionHistory(
    dateFrom: DateTime
    dateTo: DateTime
    first: Int = 20
    after: String
  ): SessionHistoryConnection!
  
  # 学習記録関連
  itemRecords(itemIds: [ID!]!): [ItemRecord!]!
  masteryStatus(
    statuses: [MasteryStatus!]
    first: Int = 50
    offset: Int = 0
  ): MasteryStatusGroup!
  
  # 進捗関連
  learningProgress(period: ProgressPeriod!): LearningProgress!
}
```

## エラーハンドリング

```rust
enum QueryError {
    NotFound,
    Unauthorized,
    InvalidParameter(String),
    ServiceUnavailable,
}
```
