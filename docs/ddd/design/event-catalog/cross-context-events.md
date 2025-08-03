# Cross-Context イベントカタログ

## 概要

異なる Bounded Context 間で共有・連携されるイベントのカタログです。これらのイベントは Integration Events として、Context 間の疎結合な連携を実現します。

## Context 間の関係マップ

```
Vocabulary ─────┬─────> Progress (項目情報の提供)
    │           │
    │           └─────> Learning (テスト用項目データ)
    │
    └─────────────────> AI Integration (生成要求)
    
Progress ───────┬─────> Notification (学習リマインダー)
    │           │
    │           └─────> User (統計情報)
    │
    └─────────────────> Learning (学習履歴)

AI Integration ────────> Vocabulary (生成結果)
```

## Integration Event 一覧

### 1. Vocabulary → Progress Context

#### VocabularyItemPublished (統合イベント)

語彙項目が公開され、学習可能になったことを通知。

```rust
pub struct VocabularyItemPublishedIntegrationEvent {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub source_context: String,  // "vocabulary"
    pub correlation_id: Option<String>,
    
    // ペイロード
    pub item_id: ItemId,
    pub entry_id: EntryId,
    pub spelling: String,
    pub part_of_speech: String,
    pub cefr_level: Option<String>,
    pub difficulty_estimate: f32,  // 0.0-1.0
    pub content_quality_score: f32,  // 0.0-1.0
}
```

**使用目的**:

- Progress Context で新規学習項目として登録
- 初期難易度の設定
- 学習推奨リストへの追加

#### VocabularyItemUpdated (統合イベント)

語彙項目の学習関連情報が更新されたことを通知。

```rust
pub struct VocabularyItemUpdatedIntegrationEvent {
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub source_context: String,
    
    pub item_id: ItemId,
    pub updated_fields: Vec<String>,  // 更新されたフィールド名
    pub difficulty_changed: bool,
    pub new_difficulty: Option<f32>,
}
```

### 2. Vocabulary → Learning Context

#### VocabularyItemsForTest (統合イベント)

テスト生成用の語彙項目データを提供。

```rust
pub struct VocabularyItemsForTestIntegrationEvent {
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub source_context: String,
    
    pub request_id: String,  // Learning からのリクエストID
    pub items: Vec<TestVocabularyItem>,
}

pub struct TestVocabularyItem {
    pub item_id: ItemId,
    pub spelling: String,
    pub part_of_speech: String,
    pub definitions: Vec<String>,
    pub examples: Vec<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
}
```

### 3. Vocabulary → AI Integration Context

#### AIGenerationRequested (統合イベント)

AI による語彙コンテンツ生成を要求。

```rust
pub struct AIGenerationRequestedIntegrationEvent {
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub source_context: String,
    
    pub request_id: String,
    pub item_id: ItemId,
    pub spelling: String,
    pub context_info: AIGenerationContext,
    pub priority: String,
    pub callback_topic: String,  // 結果を返すトピック
}

pub struct AIGenerationContext {
    pub part_of_speech: Option<String>,
    pub domain: Option<String>,
    pub target_cefr_level: Option<String>,
    pub existing_content: Option<serde_json::Value>,
    pub generation_type: String,  // "full", "definitions", "examples"
}
```

### 4. AI Integration → Vocabulary Context

#### AIGenerationCompleted (統合イベント)

AI 生成が完了したことを通知。

```rust
pub struct AIGenerationCompletedIntegrationEvent {
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub source_context: String,
    
    pub request_id: String,
    pub item_id: ItemId,
    pub success: bool,
    pub generated_content: Option<GeneratedContent>,
    pub error: Option<String>,
    pub model_used: String,
    pub tokens_used: u32,
}
```

### 5. Progress → Learning Context

#### UserProgressUpdated (統合イベント)

ユーザーの学習進捗が更新されたことを通知。

```rust
pub struct UserProgressUpdatedIntegrationEvent {
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub source_context: String,
    
    pub user_id: UserId,
    pub progress_summary: ProgressSummary,
}

pub struct ProgressSummary {
    pub total_items_learned: u32,
    pub items_due_for_review: u32,
    pub average_recall_rate: f32,
    pub current_streak: u32,
    pub next_review_date: Option<DateTime<Utc>>,
}
```

### 6. Progress → Notification Context

#### ReviewReminder (統合イベント)

復習リマインダーの送信を要求。

```rust
pub struct ReviewReminderIntegrationEvent {
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub source_context: String,
    
    pub user_id: UserId,
    pub items_due: u32,
    pub scheduled_time: DateTime<Utc>,
    pub reminder_type: String,  // "daily", "urgent", "streak_risk"
    pub custom_message: Option<String>,
}
```

### 7. Progress → User Context

#### LearningStatsUpdated (統合イベント)

学習統計が更新されたことを通知。

```rust
pub struct LearningStatsUpdatedIntegrationEvent {
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub source_context: String,
    
    pub user_id: UserId,
    pub period: String,  // "daily", "weekly", "monthly"
    pub stats: LearningStats,
}

pub struct LearningStats {
    pub items_studied: u32,
    pub study_time_minutes: u32,
    pub average_quality: f32,
    pub perfect_recalls: u32,
    pub milestones_achieved: Vec<String>,
}
```

### 8. Learning → Progress Context

#### TestCompleted (統合イベント)

テストが完了したことを通知。

```rust
pub struct TestCompletedIntegrationEvent {
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub source_context: String,
    
    pub test_id: String,
    pub user_id: UserId,
    pub test_type: String,
    pub results: Vec<TestItemResult>,
    pub overall_score: f32,
}

pub struct TestItemResult {
    pub item_id: ItemId,
    pub correct: bool,
    pub response_time_ms: u64,
    pub confidence_level: Option<f32>,
}
```

## イベントの配信保証

### At-least-once delivery

すべての Integration Event は以下を保証：

1. **永続化**: Event Store に保存後に配信
2. **再送**: 配信失敗時の自動リトライ
3. **冪等性**: 重複配信に対する耐性

### イベントの順序

- 同一集約内: 順序保証あり
- 異なる集約間: 順序保証なし（結果整合性）

## エラーハンドリング

### Dead Letter Queue

処理できないイベントは DLQ へ：

```yaml
topics:
  vocabulary-events-dlq:
    retention.ms: 2592000000  # 30日
  progress-events-dlq:
    retention.ms: 2592000000
```

### 補償トランザクション

失敗時の補償イベント：

```rust
pub struct CompensationRequired {
    pub original_event_id: EventId,
    pub failure_reason: String,
    pub compensation_action: String,
}
```

## セキュリティとプライバシー

### イベントの暗号化

個人情報を含むイベントは暗号化：

```rust
pub struct EncryptedIntegrationEvent {
    pub metadata: EventMetadata,
    pub encrypted_payload: Vec<u8>,
    pub encryption_key_id: String,
}
```

### アクセス制御

Context ごとのトピック権限：

| Context | 購読可能 | 発行可能 |
|---------|---------|---------|
| Vocabulary | progress-events | vocabulary-events |
| Progress | vocabulary-events | progress-events |
| Learning | vocabulary-events, progress-events | learning-events |
| AI Integration | vocabulary-events | ai-events |

## 監視とトレーシング

### 分散トレーシング

すべてのイベントに追跡情報を含む：

```rust
pub struct TracingContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
}
```

### メトリクス

監視すべき指標：

- イベント発行レート
- 配信遅延
- エラー率
- DLQ サイズ

## 更新履歴

- 2025-08-03: 初版作成
