# イベントドリブン機能仕様

## 概要

Effect では、すべての状態変更をイベントとして記録し、そのデータを活用して個人に最適化された学習体験を提供します。Event Sourcing パターンにより、完全な監査証跡と柔軟な分析が可能です。

## イベント設計

### イベント階層

```
DomainEvent
├── WordEvents
│   ├── WordCreated
│   ├── WordUpdated
│   ├── WordMeaningAdded
│   ├── ExampleAdded
│   └── WordRelationCreated
├── UserEvents
│   ├── UserRegistered
│   ├── UserProfileUpdated
│   ├── UserSettingsChanged
│   ├── WordFavorited
│   └── WordUnfavorited
├── SessionEvents
│   ├── SessionStarted
│   ├── QuestionPresented
│   ├── QuestionAnswered
│   └── SessionCompleted
├── ProgressEvents
│   ├── ProgressUpdated
│   ├── MilestoneAchieved
│   └── StreakUpdated
└── AnalyticsEvents
    ├── PatternDetected
    └── RecommendationGenerated
```

## 詳細なイベント定義

### 1. 単語関連イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordCreated {
    pub word_id: Uuid,
    pub text: String,
    pub phonetic_ipa: String,
    pub cefr_level: CefrLevel,
    pub difficulty: u8,
    pub categories: Vec<TestCategory>,
    pub tags: Vec<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordUpdated {
    pub word_id: Uuid,
    pub version: u32,
    pub changes: HashMap<String, Value>,
    pub updated_by: Uuid,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordMeaningAdded {
    pub meaning_id: Uuid,
    pub word_id: Uuid,
    pub meaning: String,
    pub part_of_speech: PartOfSpeech,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleAdded {
    pub example_id: Uuid,
    pub meaning_id: Uuid,
    pub sentence: String,
    pub translation: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordRelationCreated {
    pub relation_id: Uuid,
    pub word_id: Uuid,
    pub related_word_id: Uuid,
    pub relation_type: RelationType,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}
```

### 2. ユーザー関連イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistered {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordFavorited {
    pub user_id: Uuid,
    pub word_id: Uuid,
    pub favorited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordUnfavorited {
    pub user_id: Uuid,
    pub word_id: Uuid,
    pub unfavorited_at: DateTime<Utc>,
}
```

### 3. セッション関連イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStarted {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub mode: LearningMode,
    pub word_ids: Vec<Uuid>,
    pub config: SessionConfig,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnswered {
    pub session_id: Uuid,
    pub word_id: Uuid,
    pub question_type: QuestionType,
    pub is_correct: bool,
    pub response_time_ms: u32,
    pub attempt_number: u8,
    pub hint_used: bool,
    pub answered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCompleted {
    pub session_id: Uuid,
    pub duration_seconds: u32,
    pub total_questions: u32,
    pub correct_answers: u32,
    pub performance_metrics: PerformanceMetrics,
    pub completed_at: DateTime<Utc>,
}
```

### 4. 進捗関連イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdated {
    pub user_id: Uuid,
    pub word_id: Uuid,
    pub old_sm2_params: SM2Parameters,
    pub new_sm2_params: SM2Parameters,
    pub mastery_level: f32,
    pub quality_score: u8,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakUpdated {
    pub user_id: Uuid,
    pub old_streak: u32,
    pub new_streak: u32,
    pub milestone_reached: Option<u32>,
    pub updated_at: DateTime<Utc>,
}
```

## イベント処理パイプライン

### 1. イベント収集

```
User Action → Command Handler → Domain Event → Event Store
```

### 2. イベント配信

```
Event Store → Pub/Sub → Event Handlers
                    ├── Projection Handler
                    ├── Analytics Handler
                    ├── Notification Handler
                    └── Audit Logger
```

### 3. イベントハンドラー

```rust
#[async_trait]
pub trait EventHandler {
    async fn handle(&self, event: DomainEvent) -> Result<()>;
}

pub struct ProjectionHandler {
    read_db: ReadModelDb,
}

#[async_trait]
impl EventHandler for ProjectionHandler {
    async fn handle(&self, event: DomainEvent) -> Result<()> {
        match event {
            DomainEvent::WordCreated(e) => {
                self.read_db.insert_word(e).await?;
            }
            DomainEvent::WordUpdated(e) => {
                self.read_db.update_word(e).await?;
            }
            // ... 他のイベント処理
        }
        Ok(())
    }
}
```

## イベントベース分析機能

### 1. 学習パターン分析

```rust
pub struct LearningPatternAnalyzer {
    pub async fn analyze_user_patterns(
        &self,
        user_id: Uuid,
        time_range: TimeRange,
    ) -> PatternAnalysis {
        let events = self.event_store.query_user_events(user_id, time_range).await;
        
        PatternAnalysis {
            best_performance_hours: self.analyze_time_patterns(&events),
            category_performance: self.analyze_category_patterns(&events),
            difficulty_progression: self.analyze_difficulty_patterns(&events),
            common_mistakes: self.analyze_mistake_patterns(&events),
        }
    }
}
```

### 2. リアルタイム処理

```rust
pub struct EventStreamProcessor {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl EventStreamProcessor {
    pub async fn process_stream(&self) {
        let mut stream = self.event_store.subscribe().await;
        
        while let Some(event) = stream.next().await {
            for handler in &self.handlers {
                if let Err(e) = handler.handle(event.clone()).await {
                    log::error!("Handler error: {:?}", e);
                }
            }
        }
    }
}
```

### 3. アラート生成

```rust
pub enum LearningAlert {
    StreakAtRisk { days_until_lost: u32 },
    ReviewOverdue { word_count: u32 },
    PerformanceDecline { metric: String, change: f32 },
    MilestoneApproaching { milestone: String, progress: f32 },
}

pub struct AlertGenerator {
    pub async fn check_alerts(&self, user_id: Uuid) -> Vec<LearningAlert> {
        let mut alerts = vec![];
        
        // ストリーク確認
        if let Some(streak) = self.check_streak_status(user_id).await {
            alerts.push(streak);
        }
        
        // 復習期限確認
        if let Some(overdue) = self.check_overdue_reviews(user_id).await {
            alerts.push(overdue);
        }
        
        alerts
    }
}
```

## イベントストアのクエリ

### 1. 時系列クエリ

```sql
-- 特定期間の学習イベントを取得
SELECT * FROM events
WHERE aggregate_type = 'LearningSession'
  AND created_at BETWEEN ? AND ?
  AND metadata->>'user_id' = ?
ORDER BY created_at ASC;
```

### 2. 集計クエリ

```sql
-- ユーザーの週間統計
SELECT
    DATE_TRUNC('day', created_at) as day,
    COUNT(*) FILTER (WHERE event_type = 'QuestionAnswered') as total_questions,
    COUNT(*) FILTER (WHERE event_data->>'is_correct' = 'true') as correct_answers,
    AVG((event_data->>'response_time_ms')::int) as avg_response_time
FROM events
WHERE metadata->>'user_id' = ?
  AND created_at > NOW() - INTERVAL '7 days'
GROUP BY day
ORDER BY day;
```

### 3. 編集履歴クエリ

```sql
-- 単語の編集履歴
SELECT 
    event_data->>'version' as version,
    event_data->>'updated_by' as editor_id,
    event_data->>'changes' as changes,
    created_at
FROM events
WHERE aggregate_type = 'Word'
  AND aggregate_id = ?
  AND event_type = 'WordUpdated'
ORDER BY created_at DESC;
```

## データ保持とプライバシー

### 1. イベントの保持期間

- 詳細イベント: 1年間
- 集計データ: 無期限
- 編集履歴: 無期限（監査用）

### 2. プライバシー保護

- ユーザー削除時のイベント匿名化
- GDPR 準拠のデータエクスポート
- 個人を特定できる情報の最小化

### 3. パフォーマンス最適化

- イベントの非同期処理
- Read Model の適切なインデックス
- 古いイベントのアーカイブ