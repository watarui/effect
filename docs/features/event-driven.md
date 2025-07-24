# イベントドリブン機能仕様

## 概要

effect では、すべての学習行動をイベントとして記録し、そのデータを活用して個人に最適化された学習体験を提供します。

## イベント設計

### 学習イベント階層

```
DomainEvent
├── WordEvents
│   ├── WordCreated
│   ├── WordUpdated
│   └── WordDeleted
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
    pub meaning: String,
    pub difficulty: u8,
    pub category: String,
    pub tags: Vec<String>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordUpdated {
    pub word_id: Uuid,
    pub changes: HashMap<String, serde_json::Value>,
    pub updated_by: Uuid,
}
```

### 2. セッション関連イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStarted {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub mode: LearningMode,
    pub word_ids: Vec<Uuid>,
    pub config: SessionConfig,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCompleted {
    pub session_id: Uuid,
    pub duration_seconds: u32,
    pub total_questions: u32,
    pub correct_answers: u32,
    pub performance_metrics: PerformanceMetrics,
}
```

### 3. 進捗関連イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdated {
    pub user_id: Uuid,
    pub word_id: Uuid,
    pub old_sm2_params: SM2Parameters,
    pub new_sm2_params: SM2Parameters,
    pub quality_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakUpdated {
    pub user_id: Uuid,
    pub old_streak: u32,
    pub new_streak: u32,
    pub milestone_reached: Option<u32>,
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
                    └── Notification Handler
```

## イベントベース分析機能

### 1. 学習パターン分析

#### 時間帯別分析

```rust
pub struct TimeBasedAnalysis {
    pub best_performance_hours: Vec<u8>,      // 最も成績の良い時間帯
    pub average_session_duration: HashMap<u8, u32>, // 時間帯別平均セッション時間
    pub accuracy_by_hour: HashMap<u8, f32>,   // 時間帯別正答率
}
```

#### 単語タイプ別分析

```rust
pub struct WordTypeAnalysis {
    pub difficulty_performance: HashMap<u8, f32>,  // 難易度別成績
    pub category_mastery: HashMap<String, f32>,    // カテゴリ別習熟度
    pub problematic_patterns: Vec<String>,         // 苦手なパターン
}
```

### 2. 忘却曲線予測

```rust
pub struct ForgettingCurvePredictor {
    pub fn predict_retention(
        &self,
        word_id: Uuid,
        days_ahead: u32,
    ) -> f32 {
        // 過去の学習イベントから個人の忘却パターンを学習
        // 機械学習モデルによる予測
    }

    pub fn suggest_review_timing(
        &self,
        word_id: Uuid,
        target_retention: f32,
    ) -> DateTime<Utc> {
        // 目標定着率を維持するための最適な復習タイミング
    }
}
```

### 3. 学習効率最適化

```rust
pub struct EfficiencyOptimizer {
    pub fn analyze_session_efficiency(
        &self,
        events: Vec<SessionEvent>,
    ) -> EfficiencyReport {
        EfficiencyReport {
            optimal_session_length: self.calculate_optimal_length(&events),
            concentration_curve: self.analyze_concentration(&events),
            break_recommendations: self.suggest_breaks(&events),
        }
    }
}
```

## リアルタイム処理

### 1. ストリーム処理

```rust
pub struct EventStreamProcessor {
    pub async fn process_stream(&self) {
        let mut stream = self.event_store.subscribe().await;

        while let Some(event) = stream.next().await {
            match event {
                DomainEvent::QuestionAnswered(e) => {
                    self.update_real_time_metrics(e).await;
                }
                DomainEvent::SessionCompleted(e) => {
                    self.trigger_post_session_analysis(e).await;
                }
                _ => {}
            }
        }
    }
}
```

### 2. アラート生成

```rust
pub enum LearningAlert {
    StreakAtRisk { days_until_lost: u32 },
    ReviewOverdue { word_count: u32 },
    PerformanceDecline { metric: String, change: f32 },
    MilestoneApproaching { milestone: String, progress: f32 },
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
    COUNT(*) FILTER (WHERE event_data->>'is_correct' = 'true') as correct_answers
FROM events
WHERE metadata->>'user_id' = ?
  AND created_at > NOW() - INTERVAL '7 days'
GROUP BY day;
```

## プライバシーとセキュリティ

### 1. イベントの匿名化

- ユーザーIDのハッシュ化オプション
- 個人情報を含まないイベント設計

### 2. データ保持ポリシー

- 詳細イベント: 6ヶ月
- 集計データ: 無期限
- ユーザー要求による削除対応
