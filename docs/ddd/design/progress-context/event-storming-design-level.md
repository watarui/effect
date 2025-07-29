# Progress Context - EventStorming Design Level

## 概要

Progress Context は、Effect プロジェクトにおける「学習活動の鏡」として機能します。複数のコンテキストから発行されるイベントを集約し、学習の全体像を可視化する、純粋な CQRS/イベントソーシングの実践例です。

### 主要な責務

- **イベント集約**: Learning、Learning Algorithm、Vocabulary Context からのイベント収集
- **統計計算**: 日別・週別・月別の学習統計の生成
- **進捗分析**: 領域別（R/W/L/S）、レベル別（CEFR）の習熟度分析
- **可視化データ生成**: GraphQL 経由でフロントエンドに提供するデータの準備

### 設計方針

- **イベントソーシング**: すべての統計はイベントから導出
- **リードモデル中心**: 集約は持たず、プロジェクション（投影）のみ
- **GraphQL 最適化**: 柔軟なクエリに対応できる細かいリードモデル
- **ハイブリッド更新**: リアルタイムとバッチ処理の使い分け

## アーキテクチャ設計

### イベントストア + リードモデル方式

```rust
// Progress Context は集約を持たない
// すべてのデータはイベントから投影される

pub struct ProgressContext {
    event_store: EventStore,
    projections: ProjectionStore,
    cache: ProgressCache,
}

// イベントハンドラー
impl EventHandler for ProgressContext {
    async fn handle(&mut self, event: DomainEvent) -> Result<()> {
        match event {
            // Learning Context から
            DomainEvent::SessionCompleted { .. } => {
                self.update_daily_stats(event).await?;
                self.update_session_stats(event).await?;
            }
            DomainEvent::ItemMasteryUpdated { .. } => {
                self.update_mastery_stats(event).await?;
            }
            
            // Learning Algorithm Context から
            DomainEvent::ReviewRecorded { .. } => {
                self.update_item_stats(event).await?;
            }
            DomainEvent::StatisticsUpdated { .. } => {
                self.update_performance_stats(event).await?;
            }
            
            // Vocabulary Context から
            DomainEvent::ItemCreated { .. } => {
                self.update_vocabulary_stats(event).await?;
            }
            
            _ => {} // 関係ないイベントは無視
        }
        Ok(())
    }
}
```

## プロジェクション（リードモデル）の設計

### 1. DailyStatsProjection（日別統計）

```rust
pub struct DailyStatsProjection {
    // 識別子
    user_id: UserId,
    date: Date,
    
    // 学習活動
    session_count: u32,
    total_review_count: u32,
    correct_count: u32,
    incorrect_count: u32,
    
    // 時間統計
    total_study_time: Duration,
    average_response_time: Duration,
    sessions: Vec<SessionSummary>,
    
    // 項目統計
    new_items_learned: u32,
    items_mastered: u32,
    items_reviewed: u32,
    
    // パフォーマンス
    accuracy_rate: f32,
    
    // メタ情報
    last_updated: DateTime<Utc>,
    version: u64,  // 最後に処理したイベント番号
}

pub struct SessionSummary {
    session_id: SessionId,
    started_at: DateTime<Utc>,
    duration: Duration,
    item_count: u32,
    correct_count: u32,
}
```

### 2. CategoryProgressProjection（カテゴリ別進捗）

```rust
pub struct CategoryProgressProjection {
    // 識別子
    user_id: UserId,
    category: ProgressCategory,
    
    // 項目統計
    total_items: u32,
    mastered_items: u32,
    in_progress_items: u32,
    new_items: u32,
    
    // 学習統計
    total_reviews: u32,
    correct_reviews: u32,
    average_difficulty: f32,
    
    // 習熟度
    mastery_rate: f32,      // mastered / total
    accuracy_rate: f32,     // correct / reviews
    coverage_rate: f32,     // (mastered + in_progress) / total
    
    // 詳細内訳
    breakdown: CategoryBreakdown,
    
    // メタ情報
    last_calculated: DateTime<Utc>,
    version: u64,
}

pub enum ProgressCategory {
    ByDomain(Domain),        // R, W, L, S
    ByCefrLevel(CefrLevel),  // A1-C2
    ByTag(Tag),              // Business, Academic, etc
}

pub struct CategoryBreakdown {
    // サブカテゴリごとの統計
    subcategories: HashMap<String, SubcategoryStats>,
}
```

### 3. UserProgressSummaryProjection（全体サマリー）

```rust
pub struct UserProgressSummaryProjection {
    user_id: UserId,
    
    // 全体統計
    total_study_days: u32,
    total_study_time: Duration,
    total_items_learned: u32,
    total_items_mastered: u32,
    
    // 現在の状態
    current_streak: u32,
    last_study_date: Date,
    
    // パフォーマンストレンド
    weekly_accuracy_trend: Vec<f32>,   // 過去4週間
    monthly_progress_trend: Vec<u32>,   // 過去6ヶ月の習得数
    
    // レベル別サマリー
    level_distribution: HashMap<CefrLevel, LevelStats>,
    
    // IELTS スコア推定
    estimated_ielts_score: IeltsEstimation,
    
    // メタ情報
    created_at: DateTime<Utc>,
    last_updated: DateTime<Utc>,
}

pub struct IeltsEstimation {
    overall: f32,
    reading: f32,
    writing: f32,
    listening: f32,
    speaking: f32,
    confidence: f32,  // 推定の信頼度
    last_calculated: DateTime<Utc>,
}
```

## イベントハンドリング

### イベント受信と処理

```rust
impl ProgressContext {
    /// セッション完了イベントの処理
    async fn handle_session_completed(&mut self, event: SessionCompletedEvent) -> Result<()> {
        // 1. 日別統計を更新
        let daily_stats = self.get_or_create_daily_stats(event.user_id, event.date).await?;
        daily_stats.update_from_session(event.session_summary);
        
        // 2. カテゴリ別統計を更新（非同期）
        self.schedule_category_update(event.user_id, event.items);
        
        // 3. サマリーのストリークを更新
        self.update_user_summary_streak(event.user_id, event.date).await?;
        
        // 4. キャッシュを無効化
        self.cache.invalidate_user(event.user_id);
        
        Ok(())
    }
    
    /// 習熟度更新イベントの処理
    async fn handle_mastery_updated(&mut self, event: ItemMasteryUpdatedEvent) -> Result<()> {
        // 項目のカテゴリを取得
        let item_categories = self.get_item_categories(event.item_id).await?;
        
        // 各カテゴリの統計を更新
        for category in item_categories {
            let projection = self.get_category_projection(event.user_id, category).await?;
            projection.update_mastery(event.old_status, event.new_status);
        }
        
        Ok(())
    }
}
```

### 更新タイミング戦略

```rust
pub enum UpdateStrategy {
    // リアルタイム更新（即座に反映）
    Realtime,
    
    // バッチ更新（定期的に集計）
    Batch { interval: Duration },
    
    // 遅延更新（次回アクセス時に計算）
    Lazy,
}

// プロジェクションごとの更新戦略
impl ProgressContext {
    fn get_update_strategy(projection_type: &str) -> UpdateStrategy {
        match projection_type {
            "DailyStats" => UpdateStrategy::Realtime,  // 今日の統計は即反映
            "CategoryProgress" => UpdateStrategy::Batch { 
                interval: Duration::minutes(5) 
            },
            "UserSummary" => UpdateStrategy::Batch { 
                interval: Duration::hours(1) 
            },
            "IeltsEstimation" => UpdateStrategy::Lazy,  // 要求時に計算
            _ => UpdateStrategy::Realtime,
        }
    }
}
```

## 統計計算ロジック

### 正答率の計算

```rust
impl DailyStatsProjection {
    pub fn calculate_accuracy_rate(&self) -> f32 {
        if self.total_review_count == 0 {
            return 0.0;
        }
        
        self.correct_count as f32 / self.total_review_count as f32
    }
    
    pub fn calculate_weighted_accuracy(&self) -> f32 {
        // セッションごとの重み付き平均
        let total_weight: f32 = self.sessions.iter()
            .map(|s| s.item_count as f32)
            .sum();
            
        if total_weight == 0.0 {
            return 0.0;
        }
        
        self.sessions.iter()
            .map(|s| {
                let accuracy = s.correct_count as f32 / s.item_count as f32;
                accuracy * s.item_count as f32
            })
            .sum::<f32>() / total_weight
    }
}
```

### カテゴリ別習熟度の計算

```rust
impl CategoryProgressProjection {
    pub fn calculate_mastery_metrics(&mut self) {
        // 習熟率
        self.mastery_rate = if self.total_items > 0 {
            self.mastered_items as f32 / self.total_items as f32
        } else {
            0.0
        };
        
        // 正答率
        self.accuracy_rate = if self.total_reviews > 0 {
            self.correct_reviews as f32 / self.total_reviews as f32
        } else {
            0.0
        };
        
        // カバー率（学習中 + 習得済み）
        self.coverage_rate = if self.total_items > 0 {
            (self.mastered_items + self.in_progress_items) as f32 / self.total_items as f32
        } else {
            0.0
        };
    }
    
    pub fn calculate_progress_score(&self) -> f32 {
        // 総合的な進捗スコア（0-100）
        let mastery_weight = 0.5;
        let accuracy_weight = 0.3;
        let coverage_weight = 0.2;
        
        (self.mastery_rate * mastery_weight +
         self.accuracy_rate * accuracy_weight +
         self.coverage_rate * coverage_weight) * 100.0
    }
}
```

### IELTS スコア推定

```rust
impl IeltsEstimationCalculator {
    pub fn estimate_score(
        &self,
        user_progress: &UserProgressSummaryProjection,
        category_progress: &[CategoryProgressProjection],
    ) -> IeltsEstimation {
        // 領域別のスコアを計算
        let reading_score = self.estimate_domain_score(
            &category_progress.iter()
                .find(|c| matches!(c.category, ProgressCategory::ByDomain(Domain::Reading)))
                .unwrap()
        );
        
        let writing_score = self.estimate_domain_score(
            &category_progress.iter()
                .find(|c| matches!(c.category, ProgressCategory::ByDomain(Domain::Writing)))
                .unwrap()
        );
        
        // 他の領域も同様...
        
        // 総合スコアは4領域の平均（0.5刻み）
        let overall = ((reading_score + writing_score + listening_score + speaking_score) / 4.0 * 2.0).round() / 2.0;
        
        // 信頼度は学習項目数とレビュー数に基づく
        let confidence = self.calculate_confidence(user_progress);
        
        IeltsEstimation {
            overall,
            reading: reading_score,
            writing: writing_score,
            listening: listening_score,
            speaking: speaking_score,
            confidence,
            last_calculated: Utc::now(),
        }
    }
    
    fn estimate_domain_score(&self, progress: &CategoryProgressProjection) -> f32 {
        // 基準：
        // - CEFR A1-A2: IELTS 3.0-4.0
        // - CEFR B1-B2: IELTS 4.5-6.5
        // - CEFR C1-C2: IELTS 7.0-9.0
        
        // 習熟度とカバー率から推定
        let base_score = 3.0;
        let max_increment = 6.0;
        
        let mastery_factor = progress.mastery_rate;
        let coverage_factor = progress.coverage_rate;
        let accuracy_factor = progress.accuracy_rate;
        
        let weighted_factor = mastery_factor * 0.4 + coverage_factor * 0.4 + accuracy_factor * 0.2;
        
        // 0.5刻みに丸める
        ((base_score + max_increment * weighted_factor) * 2.0).round() / 2.0
    }
}
```

## GraphQL 対応

### クエリサービス

```rust
pub struct ProgressQueryService {
    projection_store: ProjectionStore,
    cache: ProgressCache,
}

impl ProgressQueryService {
    /// 日別統計の取得
    pub async fn get_daily_stats(
        &self,
        user_id: UserId,
        date: Date,
    ) -> Result<DailyStatsView> {
        // キャッシュチェック
        if let Some(cached) = self.cache.get_daily_stats(user_id, date) {
            return Ok(cached);
        }
        
        // プロジェクションから取得
        let projection = self.projection_store
            .get::<DailyStatsProjection>(user_id, date)
            .await?;
            
        let view = self.to_daily_stats_view(projection);
        
        // キャッシュに保存
        self.cache.set_daily_stats(user_id, date, view.clone());
        
        Ok(view)
    }
    
    /// 期間指定での統計取得
    pub async fn get_period_stats(
        &self,
        user_id: UserId,
        from: Date,
        to: Date,
    ) -> Result<Vec<DailyStatsView>> {
        let mut stats = Vec::new();
        let mut current = from;
        
        while current <= to {
            stats.push(self.get_daily_stats(user_id, current).await?);
            current = current + Duration::days(1);
        }
        
        Ok(stats)
    }
    
    /// カテゴリ別進捗の取得
    pub async fn get_category_progress(
        &self,
        user_id: UserId,
        category: ProgressCategory,
    ) -> Result<CategoryProgressView> {
        let projection = self.projection_store
            .get::<CategoryProgressProjection>(user_id, category)
            .await?;
            
        Ok(self.to_category_progress_view(projection))
    }
}
```

### GraphQL スキーマ対応

```graphql
type Query {
  # 日別統計
  dailyStats(userId: ID!, date: Date!): DailyStats
  periodStats(userId: ID!, from: Date!, to: Date!): [DailyStats!]!
  
  # カテゴリ別進捗
  categoryProgress(userId: ID!, category: CategoryType!, value: String!): CategoryProgress
  allDomainProgress(userId: ID!): [CategoryProgress!]!
  allLevelProgress(userId: ID!): [CategoryProgress!]!
  
  # 全体サマリー
  userProgressSummary(userId: ID!): UserProgressSummary
  
  # ストリーク
  learningStreak(userId: ID!): LearningStreak
  
  # IELTS推定
  ieltsEstimation(userId: ID!): IeltsEstimation
}

type DailyStats {
  date: Date!
  sessionCount: Int!
  totalReviewCount: Int!
  correctCount: Int!
  accuracyRate: Float!
  studyTime: Int!  # 秒数
  newItemsLearned: Int!
  itemsMastered: Int!
}

type CategoryProgress {
  category: String!
  totalItems: Int!
  masteredItems: Int!
  masteryRate: Float!
  accuracyRate: Float!
  progressScore: Float!
}
```

## ビジネスポリシー（紫の付箋 🟪）

### 統計更新ポリシー

```rust
// 日別統計は即座に更新
when SessionCompletedEvent || ItemReviewedEvent {
    update DailyStatsProjection immediately
}

// カテゴリ統計は5分ごとにバッチ更新
when timer.every(5.minutes) {
    for pending_updates in category_update_queue {
        update CategoryProgressProjection
    }
}

// IELTSスコアは要求時に計算（遅延評価）
when GetIeltsEstimationQuery {
    if last_calculated > 24.hours.ago {
        return cached_estimation
    } else {
        recalculate_estimation()
    }
}
```

### データ保持ポリシー

```rust
// 日別統計は1年間保持
when daily_stats.date < 365.days.ago {
    archive to cold_storage
}

// セッション詳細は30日間
when session.completed_at < 30.days.ago {
    remove session_details
    keep aggregated_stats only
}
```

### ストリーク判定ポリシー

```rust
// 学習日の判定
when daily_stats.session_count > 0 {
    mark_as_study_day(date)
}

// ストリークの更新
when new_study_day {
    if previous_day_studied {
        increment_streak()
    } else if gap == 1.day {
        maintain_streak()  // 1日の猶予
    } else {
        reset_streak()
    }
}
```

## リードモデル（ビュー）

### DailyStatsView（フロントエンド用）

```rust
pub struct DailyStatsView {
    date: String,  // "2024-01-20"
    
    // 基本統計
    session_count: u32,
    review_count: u32,
    accuracy_rate: f32,
    
    // 時間表示
    study_time_minutes: u32,
    average_response_seconds: f32,
    
    // 進捗
    new_items: u32,
    mastered_items: u32,
    
    // セッション詳細（オプション）
    sessions: Option<Vec<SessionView>>,
}
```

### ProgressChartData（グラフ表示用）

```rust
pub struct ProgressChartData {
    // 時系列データ
    dates: Vec<String>,
    accuracy_rates: Vec<f32>,
    review_counts: Vec<u32>,
    
    // カテゴリ別データ（レーダーチャート用）
    categories: Vec<String>,
    mastery_rates: Vec<f32>,
    
    // トレンド
    trend_direction: TrendDirection,
    trend_percentage: f32,
}
```

## パフォーマンス最適化

### キャッシュ戦略

```rust
pub struct ProgressCache {
    // 今日の統計は頻繁にアクセスされるので専用キャッシュ
    today_stats: Cache<UserId, DailyStatsView>,
    
    // 最近7日間の統計
    recent_stats: LruCache<(UserId, Date), DailyStatsView>,
    
    // カテゴリ進捗（5分間有効）
    category_progress: TtlCache<(UserId, ProgressCategory), CategoryProgressView>,
    
    // ユーザーサマリー（1時間有効）
    user_summaries: TtlCache<UserId, UserProgressSummaryView>,
}

impl ProgressCache {
    pub fn invalidate_user(&mut self, user_id: UserId) {
        self.today_stats.remove(&user_id);
        // 関連するキャッシュをクリア
    }
}
```

### インデックス設計

```sql
-- 日別統計の効率的な取得
CREATE INDEX idx_daily_stats ON daily_stats_projections (user_id, date DESC);

-- カテゴリ別統計
CREATE INDEX idx_category_progress ON category_progress_projections (user_id, category);

-- イベントストアのインデックス
CREATE INDEX idx_events_by_user ON events (user_id, occurred_at DESC);
CREATE INDEX idx_events_by_type ON events (event_type, occurred_at DESC);
```

### バッチ処理の最適化

```rust
impl BatchProcessor {
    /// カテゴリ統計の効率的な更新
    pub async fn process_category_updates(&mut self) -> Result<()> {
        // 更新が必要なユーザーをグループ化
        let updates_by_user = self.pending_updates
            .drain()
            .group_by(|u| u.user_id);
        
        // 並列処理
        let futures: Vec<_> = updates_by_user
            .into_iter()
            .map(|(user_id, updates)| {
                self.update_user_categories(user_id, updates)
            })
            .collect();
            
        futures::future::join_all(futures).await?;
        
        Ok(())
    }
}
```

## エラーハンドリング

```rust
pub enum ProgressError {
    ProjectionNotFound { user_id: UserId, date: Date },
    EventProcessingFailed { event_id: EventId, reason: String },
    CacheError { reason: String },
    CalculationError { metric: String, reason: String },
}

impl ProgressContext {
    /// イベント処理の失敗に対する復旧
    async fn handle_event_with_retry(&mut self, event: DomainEvent) -> Result<()> {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 3;
        
        loop {
            match self.handle_event(&event).await {
                Ok(_) => return Ok(()),
                Err(e) if attempts < MAX_ATTEMPTS => {
                    attempts += 1;
                    log::warn!("Event processing failed, attempt {}: {:?}", attempts, e);
                    tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                }
                Err(e) => {
                    // Dead letter queue に送る
                    self.dead_letter_queue.push(event, e).await?;
                    return Err(ProgressError::EventProcessingFailed { 
                        event_id: event.event_id(),
                        reason: e.to_string(),
                    });
                }
            }
        }
    }
}
```

## CQRS 適用方針

### 適用状況: ✅ 純粋な CQRS + イベントソーシング

Progress Context は、プロジェクト内で最も純粋な CQRS/イベントソーシングの実装例です。

### 理由

1. **Write Model が存在しない**
   - 他のコンテキストからのイベントを受信するのみ
   - 自身では状態変更を行わない
   - 純粋な「読み取り専用」コンテキスト

2. **複雑な集計要件**
   - 複数のコンテキストからのデータを統合
   - 時系列での集計（日別、週別、月別）
   - 多様な切り口での分析（カテゴリ別、レベル別、スキル別）

3. **イベントソーシングの利点を最大活用**
   - 過去の任意時点の状態を再現可能
   - 新しい集計軸の追加が容易
   - 完全な監査証跡

### Write Model（Command 側）

- **なし** - Progress Context は集約を持たない
- 他のコンテキストがイベントソース
- イベントストアがすべての真実の源

### Read Model（Query 側）

- **DailyStatsProjection**: 日別統計
- **CategoryProgressProjection**: カテゴリ別進捗
- **UserProgressSummaryProjection**: 全体サマリー
- **LearningStreakProjection**: 連続学習記録
- **責務**: GraphQL クエリに最適化されたデータ提供

### イベント処理戦略

- **リアルタイム更新**: SessionCompleted など重要イベント
- **バッチ更新**: 大量の統計再計算
- **遅延評価**: アクセス時に計算するレポート

### アーキテクチャ学習の観点

Progress Context を通じて以下を学習：

- 純粋な CQRS の実装パターン
- イベントソーシングによる状態管理
- 複数のプロジェクションの設計と管理
- GraphQL との統合における CQRS の利点
- Write なしで Read のみのコンテキスト設計

### 特記事項

このコンテキストは「学習活動の鏡」として機能し、システム全体の活動を反映します。
CQRS/ES の教科書的な実装例として、アーキテクチャ学習の中核となる部分です。

## 更新履歴

- 2025-07-27: 初版作成（CQRS/イベントソーシング実装、GraphQL対応設計）
- 2025-07-28: CQRS 適用方針セクションを追加（純粋な CQRS/ES の教科書的実装例として明記）
