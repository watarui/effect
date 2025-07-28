# Learning Algorithm Context - EventStorming Design Level

## 概要

Learning Algorithm Context は、Effect プロジェクトの学習効果を最大化する中核コンテキストです。科学的に実証された SM-2（SuperMemo 2）アルゴリズムを基盤に、最適な復習タイミングと項目選定を実現します。

### 主要な責務

- **項目選定**: 学習戦略に基づいて最適な項目を選定
- **復習スケジューリング**: SM-2 アルゴリズムによる次回復習日の計算
- **難易度管理**: 各項目の難易度係数（Easiness Factor）の調整
- **学習統計**: 正答率、習熟度、パフォーマンスの追跡

### 設計方針

- 項目ごとに独立した集約（スケーラビリティと並行性を重視）
- 反応時間を考慮した品質評価（0-5スケール）
- 85%ルールに基づく動的な難易度調整
- 科学的根拠に基づいた学習効率の最適化

## 集約の設計

### 1. ItemLearningRecord（項目学習記録）- 集約ルート

ユーザーと項目の組み合わせごとの学習状態を管理します。

```rust
pub struct ItemLearningRecord {
    // 識別子
    record_id: RecordId,  // user_id + item_id の複合キー
    user_id: UserId,
    item_id: ItemId,
    
    // SM-2 アルゴリズム関連
    easiness_factor: f32,        // 難易度係数 (1.3-2.5)
    repetition_count: u32,       // 連続正解回数
    interval_days: u32,          // 現在の復習間隔（日数）
    next_review_date: Date,      // 次回復習予定日
    
    // 統計情報
    total_reviews: u32,          // 総復習回数
    correct_count: u32,          // 正解回数
    streak_count: u32,           // 現在の連続正解数
    average_response_time: Duration,  // 平均反応時間
    last_review_date: Option<Date>,   // 最終復習日
    last_quality: Option<u8>,    // 最後の品質評価 (0-5)
    
    // 状態
    status: ReviewStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub enum ReviewStatus {
    New,                         // 未学習
    Learning {                   // 学習中（短期記憶形成中）
        step: u32,              // 現在のステップ (1-4)
    },
    Review,                      // 通常復習
    Overdue {                   // 期限切れ
        days_overdue: u32,
    },
    Suspended,                   // 一時停止中
}
```

### 2. SelectionCriteria（選定基準）- 値オブジェクト

項目選定時の評価基準を表現します。

```rust
pub struct SelectionCriteria {
    priority_score: f32,         // 優先度スコア (0.0-1.0)
    selection_reason: SelectionReason,
    urgency_factor: f32,         // 緊急度 (期限切れ日数など)
    difficulty_match: f32,       // 現在の実力との適合度
}

pub enum SelectionReason {
    NewItem,
    DueForReview { 
        scheduled_date: Date,
    },
    Overdue { 
        days_overdue: u32,
    },
    WeakItem { 
        accuracy_rate: f32,
    },
    AIRecommended { 
        reason: String,
    },
}
```

### 3. LearningPerformance（学習パフォーマンス）- 値オブジェクト

ユーザーの現在のパフォーマンスを表現します。

```rust
pub struct LearningPerformance {
    recent_accuracy: f32,        // 直近10回の正答率
    average_quality: f32,        // 平均品質評価
    session_count: u32,          // 総セッション数
    consistency_score: f32,      // 学習の継続性スコア
    optimal_difficulty: f32,     // 最適な難易度レベル
}
```

## SM-2 アルゴリズムの実装

### 品質評価の算出

```rust
impl ItemLearningRecord {
    /// 反応時間と正誤から品質評価（0-5）を算出
    pub fn calculate_quality(
        judgment: CorrectnessJudgment, 
        response_time_ms: u32
    ) -> u8 {
        match (judgment, response_time_ms) {
            // 正解の場合
            (UserConfirmedCorrect, t) if t < 2000 => 5,  // 完璧（即答）
            (UserConfirmedCorrect, t) if t < 3000 => 4,  // 良好（素早い）
            (UserConfirmedCorrect, t) if t < 5000 => 3,  // 普通
            (UserConfirmedCorrect, _) => 3,              // 遅いが正解
            
            // 自動確認（タイムアウトなし）
            (AutoConfirmed, _) => 3,                     
            
            // 不正解
            (UserConfirmedIncorrect, _) => 0,            
        }
    }
}
```

### SM-2 アルゴリズムコア

```rust
impl ItemLearningRecord {
    /// SM-2 アルゴリズムに基づいて次回復習日を計算
    pub fn calculate_next_review(&mut self, quality: u8) -> Result<ReviewUpdate> {
        // 品質が3未満の場合、リセット
        if quality < 3 {
            self.repetition_count = 0;
            self.interval_days = 1;
        } else {
            // 連続正解回数を増やす
            self.repetition_count += 1;
            
            // 復習間隔の計算
            self.interval_days = match self.repetition_count {
                1 => 1,
                2 => 6,
                _ => (self.interval_days as f32 * self.easiness_factor).round() as u32,
            };
        }
        
        // 難易度係数の更新
        self.update_easiness_factor(quality);
        
        // 次回復習日の設定
        self.next_review_date = Utc::today() + Duration::days(self.interval_days as i64);
        
        Ok(ReviewUpdate {
            next_review_date: self.next_review_date,
            interval_days: self.interval_days,
            easiness_factor: self.easiness_factor,
        })
    }
    
    /// 難易度係数の更新
    fn update_easiness_factor(&mut self, quality: u8) {
        let q = quality as f32;
        self.easiness_factor = self.easiness_factor + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02));
        
        // 範囲制限 (1.3 - 2.5)
        self.easiness_factor = self.easiness_factor.max(1.3).min(2.5);
    }
}
```

## 項目選定ロジック

### MixedStrategy（混合戦略）

```rust
pub struct MixedStrategy {
    // 基本配分
    overdue_ratio: f32,    // 40% - 期限切れ項目
    due_ratio: f32,        // 30% - 期限内復習項目
    weak_ratio: f32,       // 20% - 苦手項目
    new_ratio: f32,        // 10% - 新規項目
    
    // パフォーマンス閾値
    target_accuracy: f32,   // 0.85 (85%ルール)
    adjustment_rate: f32,   // 0.05 (調整幅)
}

impl MixedStrategy {
    /// パフォーマンスに基づいて配分を動的調整
    pub fn adjust_for_performance(&mut self, performance: &LearningPerformance) {
        let accuracy = performance.recent_accuracy;
        
        if accuracy > 0.90 {
            // 簡単すぎる → 新規項目を増やす
            self.new_ratio = (self.new_ratio + self.adjustment_rate).min(0.3);
            self.due_ratio = (self.due_ratio - self.adjustment_rate).max(0.2);
        } else if accuracy < 0.70 {
            // 難しすぎる → 復習を増やす
            self.new_ratio = (self.new_ratio - self.adjustment_rate).max(0.05);
            self.due_ratio = (self.due_ratio + self.adjustment_rate).min(0.5);
        }
        
        // 合計が1.0になるように正規化
        self.normalize_ratios();
    }
    
    /// 項目を選定
    pub fn select_items(
        &self,
        candidates: Vec<ItemCandidate>,
        count: usize,
    ) -> Vec<SelectedItem> {
        let mut selected = Vec::new();
        
        // カテゴリ別に分類
        let (overdue, due, weak, new) = self.categorize_items(candidates);
        
        // 各カテゴリから配分に従って選定
        let overdue_count = (count as f32 * self.overdue_ratio).round() as usize;
        let due_count = (count as f32 * self.due_ratio).round() as usize;
        let weak_count = (count as f32 * self.weak_ratio).round() as usize;
        let new_count = count - overdue_count - due_count - weak_count;
        
        // 優先度順に選定
        selected.extend(self.select_from_category(overdue, overdue_count));
        selected.extend(self.select_from_category(due, due_count));
        selected.extend(self.select_from_category(weak, weak_count));
        selected.extend(self.select_from_category(new, new_count));
        
        selected
    }
}
```

## コマンドとイベント

### コマンド（青い付箋 🟦）

```rust
pub enum LearningAlgorithmCommand {
    // 項目選定
    SelectItems {
        user_id: UserId,
        strategy: SelectionStrategy,
        count: usize,
    },
    
    // 学習結果の記録
    RecordReview {
        user_id: UserId,
        item_id: ItemId,
        quality: u8,
        response_time_ms: u32,
    },
    
    // スケジュールの更新
    UpdateSchedule {
        user_id: UserId,
        item_id: ItemId,
        quality: u8,
    },
    
    // 統計の更新
    UpdateStatistics {
        user_id: UserId,
        session_results: Vec<ReviewResult>,
    },
    
    // 項目の状態変更
    SuspendItem {
        user_id: UserId,
        item_id: ItemId,
    },
    
    ResumeItem {
        user_id: UserId,
        item_id: ItemId,
    },
}
```

### ドメインイベント（オレンジの付箋 🟠）

```rust
pub enum LearningAlgorithmEvent {
    // 項目選定
    ItemsSelected {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        selected_items: Vec<SelectedItem>,
        strategy: SelectionStrategy,
    },
    
    // 復習記録
    ReviewRecorded {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        item_id: ItemId,
        quality: u8,
        response_time_ms: u32,
    },
    
    // スケジュール更新
    ScheduleUpdated {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        item_id: ItemId,
        next_review_date: Date,
        interval_days: u32,
        easiness_factor: f32,
    },
    
    // 状態変更
    ItemStatusChanged {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        item_id: ItemId,
        old_status: ReviewStatus,
        new_status: ReviewStatus,
    },
    
    // 統計更新
    StatisticsUpdated {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        performance: LearningPerformance,
    },
    
    // 戦略調整
    StrategyAdjusted {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        old_ratios: MixedStrategy,
        new_ratios: MixedStrategy,
        reason: String,
    },
}
```

## ビジネスポリシー（紫の付箋 🟪）

### SM-2 計算ポリシー

```rust
// 品質評価に基づくスケジュール更新
when ReviewRecordedEvent {
    if quality >= 3 {
        // 成功 → 間隔を延ばす
        calculate_next_interval()
        update_easiness_factor()
    } else {
        // 失敗 → リセット
        reset_to_learning_phase()
    }
    emit ScheduleUpdatedEvent
}
```

### 項目選定ポリシー

```rust
// 期限切れ項目の優先処理
when SelectItemsCommand {
    // 1. 期限切れ項目を最優先
    prioritize_overdue_items()
    
    // 2. 戦略に基づいて残りを選定
    apply_selection_strategy()
    
    // 3. 重複や除外項目をフィルタ
    filter_invalid_items()
    
    emit ItemsSelectedEvent
}
```

### パフォーマンス調整ポリシー

```rust
// セッション完了時の戦略調整
when SessionCompletedEvent {
    calculate_session_accuracy()
    
    if should_adjust_strategy(accuracy) {
        adjust_strategy_ratios()
        emit StrategyAdjustedEvent
    }
}

// 85%ルールの適用
fn should_adjust_strategy(accuracy: f32) -> bool {
    accuracy < 0.70 || accuracy > 0.90
}
```

### 状態遷移ポリシー

```rust
// 新規項目の学習開始
when first_review && quality >= 3 {
    change_status(New -> Learning { step: 1 })
}

// 学習フェーズの進行
when in_learning_phase && quality >= 3 {
    if step < 4 {
        increment_learning_step()
    } else {
        graduate_to_review()
    }
}

// 期限切れの検出
when current_date > next_review_date {
    change_status(Review -> Overdue { days: calculate_overdue_days() })
}
```

## リードモデル（緑の付箋 🟩）

### ItemSelectionView（項目選定用ビュー）

```rust
pub struct ItemSelectionView {
    user_id: UserId,
    item_id: ItemId,
    
    // 選定用スコア
    priority_score: f32,
    urgency_score: f32,
    difficulty_score: f32,
    
    // 基本情報
    spelling: String,
    next_review_date: Option<Date>,
    days_overdue: Option<i32>,
    
    // 統計
    accuracy_rate: f32,
    average_quality: f32,
    review_count: u32,
    
    // カテゴリ
    category: SelectionCategory,
}

pub enum SelectionCategory {
    Overdue,
    DueToday,
    Weak,
    New,
    Normal,
}
```

### ReviewScheduleView（復習スケジュール表示用）

```rust
pub struct ReviewScheduleView {
    user_id: UserId,
    date: Date,
    
    // 日別の項目数
    overdue_count: u32,
    due_today_count: u32,
    upcoming_counts: HashMap<Date, u32>,  // 今後7日間
    
    // 項目リスト
    items: Vec<ScheduledItemView>,
}

pub struct ScheduledItemView {
    item_id: ItemId,
    spelling: String,
    scheduled_date: Date,
    interval_days: u32,
    repetition_count: u32,
    status: ReviewStatus,
}
```

### LearningStatisticsView（統計表示用）

```rust
pub struct LearningStatisticsView {
    user_id: UserId,
    period: StatisticsPeriod,
    
    // 全体統計
    total_reviews: u32,
    total_items: u32,
    mastered_items: u32,
    
    // パフォーマンス
    accuracy_rate: f32,
    average_quality: f32,
    average_response_time: Duration,
    
    // 進捗
    daily_reviews: Vec<DailyReviewCount>,
    retention_curve: Vec<RetentionPoint>,
    
    // 難易度分布
    difficulty_distribution: HashMap<DifficultyRange, u32>,
}

pub struct RetentionPoint {
    days_after_learning: u32,
    retention_rate: f32,
}
```

## 実装の詳細

### 項目選定サービス

```rust
impl ItemSelectionService for LearningAlgorithmContext {
    async fn select_items(
        &self,
        user_id: UserId,
        strategy: SelectionStrategy,
        count: usize,
    ) -> Result<Vec<SelectedItem>> {
        // 1. ユーザーの全項目を取得
        let records = self.repository.get_user_records(user_id).await?;
        
        // 2. 選定可能な項目をフィルタ
        let candidates = records.into_iter()
            .filter(|r| r.status != ReviewStatus::Suspended)
            .map(|r| self.to_candidate(r))
            .collect();
        
        // 3. 戦略に基づいて選定
        let selected = match strategy {
            SelectionStrategy::NewItemsFirst => {
                self.select_new_items_first(candidates, count)
            }
            SelectionStrategy::DueForReview { date, include_overdue } => {
                self.select_due_items(candidates, date, include_overdue, count)
            }
            SelectionStrategy::Mixed { .. } => {
                let performance = self.get_user_performance(user_id).await?;
                let mut strategy = MixedStrategy::from(strategy);
                strategy.adjust_for_performance(&performance);
                strategy.select_items(candidates, count)
            }
            // 他の戦略...
        };
        
        Ok(selected)
    }
}
```

### パフォーマンス計算

```rust
impl LearningAlgorithmContext {
    async fn calculate_user_performance(
        &self,
        user_id: UserId,
    ) -> Result<LearningPerformance> {
        // 直近のレビュー結果を取得
        let recent_reviews = self.event_store
            .get_recent_reviews(user_id, 10)
            .await?;
        
        // 正答率の計算
        let correct_count = recent_reviews.iter()
            .filter(|r| r.quality >= 3)
            .count();
        let recent_accuracy = correct_count as f32 / recent_reviews.len() as f32;
        
        // 平均品質の計算
        let average_quality = recent_reviews.iter()
            .map(|r| r.quality as f32)
            .sum::<f32>() / recent_reviews.len() as f32;
        
        // 最適難易度の推定（85%ルールに基づく）
        let optimal_difficulty = self.estimate_optimal_difficulty(recent_accuracy);
        
        Ok(LearningPerformance {
            recent_accuracy,
            average_quality,
            session_count: self.get_session_count(user_id).await?,
            consistency_score: self.calculate_consistency(user_id).await?,
            optimal_difficulty,
        })
    }
}
```

## 他コンテキストとの連携

### Learning Context への提供

```rust
// 項目選定サービス
trait ItemSelectionService {
    async fn select_items(
        &self,
        user_id: UserId,
        strategy: SelectionStrategy,
        count: usize,
    ) -> Result<Vec<SelectedItem>>;
}

// スケジュール照会サービス
trait ScheduleQueryService {
    async fn get_next_review_date(
        &self,
        user_id: UserId,
        item_id: ItemId,
    ) -> Result<Option<Date>>;
    
    async fn get_review_items_for_date(
        &self,
        user_id: UserId,
        date: Date,
    ) -> Result<Vec<ScheduledItem>>;
}
```

### Progress Context へのイベント発行

```rust
// Learning Algorithm → Progress
StatisticsUpdatedEvent {
    user_id,
    total_items,
    mastered_items,
    accuracy_rate,
    // Progress Context が集計に使用
}
```

### Learning Context からのイベント受信

```rust
// Learning → Learning Algorithm
impl EventHandler for LearningAlgorithmContext {
    async fn handle(&self, event: LearningDomainEvent) -> Result<()> {
        match event {
            LearningDomainEvent::CorrectnessJudged { user_id, item_id, judgment, .. } => {
                // 品質を計算してレビューを記録
                let quality = self.calculate_quality(judgment);
                self.record_review(user_id, item_id, quality).await?;
            }
            LearningDomainEvent::SessionCompleted { user_id, .. } => {
                // パフォーマンスを更新
                self.update_user_performance(user_id).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

## エラーハンドリング

```rust
pub enum LearningAlgorithmError {
    RecordNotFound { user_id: UserId, item_id: ItemId },
    InvalidQuality { value: u8 },
    InvalidStrategy { reason: String },
    InsufficientItems { requested: usize, available: usize },
}
```

## パフォーマンス最適化

### インデックス戦略

```rust
// 効率的な項目選定のためのインデックス
CREATE INDEX idx_next_review ON item_learning_records (user_id, next_review_date);
CREATE INDEX idx_status ON item_learning_records (user_id, status);
CREATE INDEX idx_accuracy ON item_learning_records (user_id, correct_count, total_reviews);
```

### キャッシュ戦略

```rust
pub struct LearningAlgorithmCache {
    // ユーザーごとのパフォーマンスキャッシュ
    performance_cache: Cache<UserId, LearningPerformance>,
    
    // 本日の復習項目キャッシュ
    today_items_cache: Cache<UserId, Vec<ScheduledItem>>,
    
    // 統計情報キャッシュ
    statistics_cache: Cache<(UserId, StatisticsPeriod), LearningStatisticsView>,
}
```

## CQRS 適用方針

### 適用状況: ❌ 通常の DDD（CQRS なし）

Learning Algorithm Context では、従来の DDD パターンを採用し、CQRS は適用していません。

### 理由

1. **シンプルな責務**
   - SM-2 アルゴリズムによる計算処理が中心
   - 複雑な表示要件がない
   - 読み取りと書き込みのモデルが本質的に同じ

2. **内部サービス的な性質**
   - 他のコンテキストから呼び出される計算エンジン
   - UI に直接データを提供することが少ない
   - ビジネスロジックの実行が主目的

3. **データ構造の安定性**
   - ItemLearningRecord の構造が SM-2 アルゴリズムに最適化
   - 表示用の変換が最小限
   - 正規化された状態で十分

### アーキテクチャ設計

- **集約**: ItemLearningRecord（学習記録）
- **リポジトリ**: 読み書き両方を同じモデルで処理
- **ドメインサービス**: SM2Calculator、PerformanceAnalyzer など
- **データアクセス**: 集約をそのまま使用（DTO 変換は最小限）

### 他コンテキストとの連携

- Learning Context に対して計算結果を提供
- Progress Context にイベントを発行
- いずれも内部的な連携で、UI 表示は他コンテキストが担当

### アーキテクチャ学習の観点

Learning Algorithm Context を通じて以下を学習：

- CQRS が不要な場合の判断基準
- 通常の DDD パターンで十分なケース
- ドメインサービスを中心とした設計
- 「すべてに CQRS を適用しない」という設計判断

## 更新履歴

- 2025-07-27: 初版作成（SM-2アルゴリズム実装、項目選定戦略の詳細設計）
- 2025-07-28: CQRS 適用方針セクションを追加（通常の DDD パターンで十分な理由を明記）
