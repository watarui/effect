# Learning Algorithm Context - リポジトリインターフェース

## 概要

Learning Algorithm Context には 1 つの主要な集約が存在します：

- `ItemLearningRecord`：SM-2 アルゴリズムに基づく学習記録の管理

このコンテキストは学習アルゴリズムの計算に特化しており、UI 表示用の `UserItemRecord` (Learning Context) とは明確に分離されています。

## ItemLearningRecordRepository

学習アルゴリズム用の記録を管理するリポジトリです。

### インターフェース定義

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// 学習アルゴリズム記録のリポジトリ
#[async_trait]
pub trait ItemLearningRecordRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    // ===== 基本的な CRUD 操作 =====
    
    /// ユーザーと項目の組み合わせで記録を取得
    async fn find_by_user_and_item(
        &self,
        user_id: &UserId,
        item_id: &ItemId,
    ) -> Result<Option<ItemLearningRecord>, Self::Error>;
    
    /// 記録を保存（新規作成または更新）
    async fn save(&self, record: &ItemLearningRecord) -> Result<(), Self::Error>;
    
    /// 複数の記録を一括保存（バッチ処理用）
    async fn save_batch(&self, records: &[ItemLearningRecord]) -> Result<(), Self::Error>;
    
    /// 記録を削除（通常は使用しない）
    async fn delete(
        &self,
        user_id: &UserId,
        item_id: &ItemId,
    ) -> Result<(), Self::Error>;
    
    // ===== アルゴリズム用クエリ =====
    
    /// 次回復習日が到来している項目を取得
    async fn find_due_for_review(
        &self,
        user_id: &UserId,
        as_of: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<ItemLearningRecord>, Self::Error>;
    
    /// 新規項目（未学習）を取得
    async fn find_new_items(
        &self,
        user_id: &UserId,
        item_ids: &[ItemId],
        limit: u32,
    ) -> Result<Vec<ItemId>, Self::Error>;
    
    // ===== 最適化用クエリ =====
    
    /// 学習最適化のための項目選定
    /// (新規:復習 = 1:4 の比率、困難度分散など)
    async fn find_optimal_items(
        &self,
        user_id: &UserId,
        session_config: &SessionConfig,
    ) -> Result<OptimalItemSelection, Self::Error>;
    
    /// ユーザーの学習パフォーマンスを取得
    async fn get_user_performance(
        &self,
        user_id: &UserId,
    ) -> Result<UserPerformance, Self::Error>;
    
    // ===== 統計・分析用 =====
    
    /// Easiness Factor の分布を取得
    async fn get_easiness_distribution(
        &self,
        user_id: &UserId,
    ) -> Result<HashMap<EasinessRange, u64>, Self::Error>;
    
    // ===== バルク操作 =====
    
    /// ユーザーの全記録を削除（アカウント削除時）
    async fn delete_all_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<u64, Self::Error>;
}
```

### 使用例

```rust
// アプリケーションサービスでの使用例
pub struct ProcessLearningResultUseCase<R: ItemLearningRecordRepository> {
    repository: Arc<R>,
    event_bus: Arc<dyn EventBus>,
}

impl<R: ItemLearningRecordRepository> ProcessLearningResultUseCase<R> {
    pub async fn execute(
        &self,
        user_id: UserId,
        item_id: ItemId,
        result: ReviewResult,
    ) -> Result<NextReviewInfo> {
        // 既存の記録を取得または新規作成
        let mut record = match self.repository
            .find_by_user_and_item(&user_id, &item_id)
            .await?
        {
            Some(record) => record,
            None => ItemLearningRecord::new(user_id.clone(), item_id.clone()),
        };
        
        // SM-2 アルゴリズムを適用
        let previous_interval = record.current_interval();
        record.apply_review_result(result, Utc::now())?;
        
        // 次回復習情報
        let next_review_info = NextReviewInfo {
            next_review_date: record.next_review_date().clone(),
            interval_days: record.current_interval(),
            easiness_factor: record.easiness_factor(),
        };
        
        // 保存
        self.repository.save(&record).await?;
        
        // イベント発行
        self.event_bus.publish(ReviewProcessed {
            user_id,
            item_id,
            previous_interval,
            new_interval: record.current_interval(),
            easiness_factor: record.easiness_factor(),
        }).await?;
        
        Ok(next_review_info)
    }
}
```

### 補助的な型定義

```rust
/// セッション設定
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub total_items: u32,
    pub new_item_ratio: f32,  // 0.2 = 20% 新規項目
    pub difficulty_distribution: DifficultyDistribution,
    pub available_time_minutes: Option<u32>,
}

/// 最適な項目選定結果
#[derive(Debug)]
pub struct OptimalItemSelection {
    pub new_items: Vec<ItemId>,
    pub review_items: Vec<ItemLearningRecord>,
    pub estimated_time_minutes: u32,
    pub difficulty_balance: DifficultyBalance,
}

/// ユーザーパフォーマンス
#[derive(Debug)]
pub struct UserPerformance {
    pub average_accuracy: f64,
    pub average_response_time_ms: u64,
    pub retention_rate: f64,
    pub daily_review_completion_rate: f64,
}

/// 復習結果
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub response_quality: ResponseQuality,
    pub response_time_ms: u64,
    pub hints_used: u32,
}

/// 応答品質 (SM-2 アルゴリズム用)
#[derive(Debug, Clone, Copy)]
pub enum ResponseQuality {
    Perfect = 5,        // 完璧な回答
    CorrectEasy = 4,    // 正解（簡単だった）
    CorrectDifficult = 3, // 正解（難しかった）
    IncorrectEasy = 2,  // 不正解（もう少しで正解）
    IncorrectDifficult = 1, // 不正解（難しかった）
    Blackout = 0,       // 完全に忘れた
}

/// Easiness Factor の範囲
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EasinessRange {
    VeryDifficult, // 1.3 - 1.7
    Difficult,     // 1.7 - 2.1
    Medium,        // 2.1 - 2.5
    Easy,          // 2.5 - 2.9
    VeryEasy,      // 2.9+
}

/// 間隔の範囲
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntervalRange {
    New,           // 0 days
    ShortTerm,     // 1-6 days
    MediumTerm,    // 7-30 days
    LongTerm,      // 31-180 days
    VeryLongTerm,  // 180+ days
}
```

## 実装上の考慮事項

### 1. パフォーマンス最適化

```rust
// インデックスの推奨
// ItemLearningRecord
// - (user_id, item_id) - プライマリキー相当
// - (user_id, next_review_date) - 復習予定項目の検索
// - (user_id, easiness_factor) - 困難度別検索
// - (user_id, current_interval) - 間隔別検索
// - (user_id, last_reviewed_at) - 最近の復習順
```

### 2. トランザクション境界

```rust
// 学習結果のバッチ処理例
pub async fn process_session_results(
    repo: &dyn ItemLearningRecordRepository,
    session_results: Vec<SessionResult>,
) -> Result<()> {
    // 各結果を個別に処理（各々が独立したトランザクション）
    for result in session_results {
        if let Err(e) = process_single_result(repo, result).await {
            // エラーをログに記録して続行
            log::error!("Failed to process result: {:?}", e);
            // リトライキューに追加するなどの処理
        }
    }
    Ok(())
}

async fn process_single_result(
    repo: &dyn ItemLearningRecordRepository,
    result: SessionResult,
) -> Result<()> {
    // 1 つの ItemLearningRecord の更新が 1 トランザクション
    let mut record = repo.find_by_user_and_item(&result.user_id, &result.item_id)
        .await?
        .unwrap_or_else(|| ItemLearningRecord::new(result.user_id, result.item_id));
    
    record.apply_review_result(result.review_result, result.timestamp)?;
    repo.save(&record).await?;
    
    Ok(())
}
```

### 3. エラーハンドリング

```rust
/// Learning Algorithm Context 固有のリポジトリエラー
#[derive(Debug, thiserror::Error)]
pub enum AlgorithmRepositoryError {
    #[error("Record not found for user {user_id} and item {item_id}")]
    RecordNotFound { user_id: UserId, item_id: ItemId },
    
    #[error("Invalid algorithm parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Optimization failed: {0}")]
    OptimizationError(String),
    
    #[error("Database error: {0}")]
    Database(String),
}
```

### 4. アルゴリズムのバージョン管理

```rust
// SM-2 アルゴリズムの実装例
impl ItemLearningRecord {
    pub fn apply_review_result(
        &mut self,
        result: ReviewResult,
        reviewed_at: DateTime<Utc>,
    ) -> Result<()> {
        let quality = result.response_quality as u8;
        
        // SM-2 アルゴリズム
        if quality >= 3 {
            // 正解の場合
            if self.repetition_count == 0 {
                self.current_interval = 1;
            } else if self.repetition_count == 1 {
                self.current_interval = 6;
            } else {
                self.current_interval = 
                    (self.current_interval as f32 * self.easiness_factor) as u32;
            }
            self.repetition_count += 1;
        } else {
            // 不正解の場合
            self.repetition_count = 0;
            self.current_interval = 1;
        }
        
        // Easiness Factor の更新
        self.easiness_factor = (self.easiness_factor + 
            (0.1 - (5.0 - quality as f32) * (0.08 + (5.0 - quality as f32) * 0.02)))
            .max(1.3);
        
        // 次回復習日の計算
        self.next_review_date = reviewed_at + chrono::Duration::days(self.current_interval as i64);
        self.last_reviewed_at = Some(reviewed_at);
        self.review_count += 1;
        
        // アルゴリズムバージョンを記録
        self.algorithm_version = "SM-2-v1".to_string();
        
        Ok(())
    }
}
```

## Learning Context との連携

```rust
// イベントハンドラーでの連携
pub struct AlgorithmEventHandler<R: ItemLearningRecordRepository> {
    repository: Arc<R>,
}

#[async_trait]
impl<R: ItemLearningRecordRepository> EventHandler for AlgorithmEventHandler<R> {
    async fn handle(&self, event: DomainEvent) -> Result<()> {
        match event {
            DomainEvent::CorrectnessJudged { 
                user_id, 
                item_id, 
                is_correct, 
                response_time_ms,
                .. 
            } => {
                // 応答品質を判定
                let quality = determine_response_quality(is_correct, response_time_ms);
                
                // 学習記録を更新
                let result = ReviewResult {
                    response_quality: quality,
                    response_time_ms,
                    hints_used: 0,
                };
                
                self.process_review(user_id, item_id, result).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

fn determine_response_quality(is_correct: bool, response_time_ms: u64) -> ResponseQuality {
    match (is_correct, response_time_ms) {
        (true, t) if t < 3000 => ResponseQuality::Perfect,
        (true, t) if t < 10000 => ResponseQuality::CorrectEasy,
        (true, _) => ResponseQuality::CorrectDifficult,
        (false, t) if t < 5000 => ResponseQuality::IncorrectEasy,
        (false, _) => ResponseQuality::IncorrectDifficult,
    }
}
```

## 更新履歴

- 2025-07-28: 初版作成（SM-2 アルゴリズムを中心とした設計）
- 2025-07-29: MVP 向けに簡潔化（細かすぎる検索・統計機能を削除、streak_days を削除）
