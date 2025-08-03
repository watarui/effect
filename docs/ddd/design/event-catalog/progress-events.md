# Progress Context イベントカタログ

## 概要

Progress Context で発生するすべてのドメインイベントのカタログです。
学習進捗、SM-2 アルゴリズムによるスケジューリング、学習統計に関するイベントを管理します。

## イベント一覧

| イベント名               | 説明                       | 発生タイミング                      |
| ------------------------ | -------------------------- | ----------------------------------- |
| LearningSessionStarted   | 学習セッションが開始された | ユーザーが学習を開始した時          |
| ItemStudied              | 項目が学習された           | 項目の学習（表示）が記録された時    |
| ItemRecalled             | 項目の想起結果が記録された | ユーザーが回答し評価された時        |
| ReviewScheduled          | 復習がスケジュールされた   | SM-2 により次回復習日が計算された時 |
| StreakUpdated            | 連続学習日数が更新された   | 日次の学習が完了した時              |
| MilestoneAchieved        | マイルストーンが達成された | 特定の条件を満たした時              |
| LearningSessionCompleted | 学習セッションが完了した   | セッションが正常終了した時          |
| DifficultyAdjusted       | 項目の難易度が調整された   | 正答率に基づく自動調整時            |
| StudyTimeRecorded        | 学習時間が記録された       | 項目ごとの学習時間集計時            |

## イベント詳細

### 1. LearningSessionStarted

学習セッションの開始を表すイベント。

```rust
pub struct LearningSessionStarted {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: SessionId,
    pub aggregate_version: u32,

    // イベントペイロード
    pub session_id: SessionId,
    pub user_id: UserId,
    pub session_type: SessionType,
    pub planned_items: u32,
    pub time_limit_minutes: Option<u32>,
    pub context: StudyContext,
}

pub enum SessionType {
    NewItems,
    Review,
    Mixed,
    Test,
}

pub struct StudyContext {
    pub device_type: String,
    pub location: Option<String>,
    pub continuation_from: Option<SessionId>,
}
```

**発生条件**:

- ユーザーが学習画面を開いた時
- 新規セッションが作成された時

**例**:

```json
{
  "event_id": "evt_session_001",
  "occurred_at": "2025-08-03T09:00:00Z",
  "aggregate_id": "ses_20250803_usr123_001",
  "aggregate_version": 1,
  "session_id": "ses_20250803_usr123_001",
  "user_id": "usr_123",
  "session_type": "Review",
  "planned_items": 20,
  "time_limit_minutes": 30,
  "context": {
    "device_type": "mobile",
    "location": "commute",
    "continuation_from": null
  }
}
```

### 2. ItemStudied

項目が学習（表示）されたことを表すイベント。

```rust
pub struct ItemStudied {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: ProgressId,  // user_id + item_id の複合キー
    pub aggregate_version: u32,

    // イベントペイロード
    pub progress_id: ProgressId,
    pub user_id: UserId,
    pub item_id: ItemId,
    pub session_id: SessionId,
    pub study_type: StudyType,
    pub presentation_order: u32,
    pub time_spent_ms: u64,
}

pub enum StudyType {
    FirstTime,         // 初回学習
    ScheduledReview,   // スケジュール復習
    ExtraReview,       // 追加復習
    TestMode,          // テストモード
}
```

**発生条件**:

- 学習画面で項目が表示された時
- フラッシュカードが提示された時

### 3. ItemRecalled

項目の想起（回答）結果を表すイベント。

```rust
pub struct ItemRecalled {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: ProgressId,
    pub aggregate_version: u32,

    // イベントペイロード
    pub progress_id: ProgressId,
    pub user_id: UserId,
    pub item_id: ItemId,
    pub session_id: SessionId,
    pub recall_quality: RecallQuality,
    pub response_time_ms: u64,
    pub hint_used: bool,
    pub attempt_number: u32,
}

pub enum RecallQuality {
    Perfect = 5,        // 完璧な想起
    CorrectEasy = 4,    // 正解（簡単）
    CorrectDifficult = 3, // 正解（難しい）
    IncorrectEasy = 2,  // 不正解（簡単に思い出せそう）
    IncorrectDifficult = 1, // 不正解（難しい）
    Blackout = 0,       // 完全に忘れた
}
```

**発生条件**:

- ユーザーが回答を提出した時
- 自己評価が完了した時

### 4. ReviewScheduled

SM-2 アルゴリズムにより次回復習がスケジュールされたイベント。

```rust
pub struct ReviewScheduled {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: ProgressId,
    pub aggregate_version: u32,

    // イベントペイロード
    pub progress_id: ProgressId,
    pub user_id: UserId,
    pub item_id: ItemId,
    pub scheduled_for: DateTime<Utc>,
    pub interval_days: f32,
    pub easiness_factor: f32,
    pub repetition_number: u32,
    pub algorithm_version: String,
}
```

**発生条件**:

- ItemRecalled イベントの後
- SM-2 計算が完了した時

**SM-2 アルゴリズムの実装**:

```rust
// 次回復習間隔の計算
fn calculate_interval(
    quality: RecallQuality,
    repetition: u32,
    previous_interval: f32,
    easiness_factor: f32,
) -> (f32, f32) {
    let q = quality as u8;

    // Easiness Factor の更新
    let new_ef = (easiness_factor + (0.1 - (5 - q) as f32 * (0.08 + (5 - q) as f32 * 0.02))).max(1.3);

    // 間隔の計算
    let new_interval = match repetition {
        0 => 1.0,
        1 => 6.0,
        _ => previous_interval * new_ef,
    };

    (new_interval, new_ef)
}
```

### 5. StreakUpdated

連続学習日数が更新されたイベント。

```rust
pub struct StreakUpdated {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: UserId,
    pub aggregate_version: u32,

    // イベントペイロード
    pub user_id: UserId,
    pub new_streak: u32,
    pub previous_streak: u32,
    pub streak_type: StreakType,
    pub bonus_points: Option<u32>,
}

pub enum StreakType {
    Daily,           // 日次連続
    Weekly,          // 週次連続
    Monthly,         // 月次連続
    LongestEver,     // 過去最長更新
}
```

### 6. MilestoneAchieved

学習マイルストーンが達成されたイベント。

```rust
pub struct MilestoneAchieved {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: UserId,
    pub aggregate_version: u32,

    // イベントペイロード
    pub user_id: UserId,
    pub milestone_type: MilestoneType,
    pub milestone_value: u32,
    pub reward: Option<Reward>,
}

pub enum MilestoneType {
    TotalItemsLearned,      // 学習項目数
    PerfectRecalls,         // 完璧な想起数
    StudyHours,             // 総学習時間
    ConsecutiveDays,        // 連続学習日数
    MonthlyGoalReached,     // 月間目標達成
}

pub struct Reward {
    pub reward_type: String,
    pub reward_value: serde_json::Value,
}
```

### 7. LearningSessionCompleted

学習セッションが完了したイベント。

```rust
pub struct LearningSessionCompleted {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: SessionId,
    pub aggregate_version: u32,

    // イベントペイロード
    pub session_id: SessionId,
    pub user_id: UserId,
    pub duration_seconds: u64,
    pub items_studied: u32,
    pub items_recalled: u32,
    pub average_quality: f32,
    pub completion_reason: CompletionReason,
}

pub enum CompletionReason {
    AllItemsCompleted,
    TimeLimit,
    UserEnded,
    SystemInterruption,
}
```

### 8. DifficultyAdjusted

項目の難易度が自動調整されたイベント。

```rust
pub struct DifficultyAdjusted {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: ItemId,
    pub aggregate_version: u32,

    // イベントペイロード
    pub item_id: ItemId,
    pub old_difficulty: f32,
    pub new_difficulty: f32,
    pub adjustment_reason: AdjustmentReason,
    pub sample_size: u32,
    pub success_rate: f32,
}

pub enum AdjustmentReason {
    LowSuccessRate,      // 正答率が低い
    HighSuccessRate,     // 正答率が高い
    UserFeedback,        // ユーザーフィードバック
    AdminOverride,       // 管理者による変更
}
```

## イベントの順序と関係

典型的な学習フロー：

```
LearningSessionStarted
    ↓
ItemStudied (item_1)
    ↓
ItemRecalled (item_1) → ReviewScheduled (item_1)
    ↓
ItemStudied (item_2)
    ↓
ItemRecalled (item_2) → ReviewScheduled (item_2)
    ↓
    ...
    ↓
LearningSessionCompleted
    ↓
StreakUpdated (if applicable)
    ↓
MilestoneAchieved (if applicable)
```

## Projection への影響

各イベントが更新する Read Model：

| イベント               | UserProgressView | ItemDifficultyView | LearningStatsView | StreakView |
| ---------------------- | ---------------- | ------------------ | ----------------- | ---------- |
| LearningSessionStarted | ✓                | -                  | ✓                 | -          |
| ItemStudied            | ✓                | -                  | ✓                 | -          |
| ItemRecalled           | ✓                | ✓                  | ✓                 | -          |
| ReviewScheduled        | ✓                | -                  | -                 | -          |
| StreakUpdated          | ✓                | -                  | -                 | ✓          |
| MilestoneAchieved      | ✓                | -                  | ✓                 | -          |
| DifficultyAdjusted     | -                | ✓                  | -                 | -          |

## 分析用途

Progress Context のイベントは以下の分析に使用されます：

1. **学習効率分析**

   - 項目ごとの平均学習時間
   - 難易度と成功率の相関
   - 最適な復習間隔の調整

2. **ユーザー行動分析**

   - 学習時間帯の傾向
   - セッション長の分布
   - 離脱ポイントの特定

3. **アルゴリズム改善**
   - SM-2 パラメータの最適化
   - 個人化された難易度調整
   - 復習スケジュールの改良

## 更新履歴

- 2025-08-03: 初版作成
