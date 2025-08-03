# progress-query-service 設計書

## 概要

progress-query-service は、Progress Context の Read Model を管理し、学習進捗、統計情報、復習スケジュールなどの読み取り専用 API を提供します。高速な応答のため Redis キャッシュを活用し、様々な観点からの進捗分析を可能にします。

## 責務

1. **進捗情報の提供**
   - ユーザー全体の学習進捗
   - 項目別の詳細進捗
   - 復習予定の管理

2. **統計情報の表示**
   - 日次/週次/月次統計
   - 学習パターン分析
   - 難易度分布

3. **ランキング機能**
   - 連続学習日数ランキング
   - 学習項目数ランキング
   - 週間/月間ランキング

4. **キャッシュ管理**
   - 頻繁にアクセスされるデータのキャッシュ
   - キャッシュの無効化戦略
   - キャッシュウォーミング

## アーキテクチャ

### レイヤー構造

```
progress-query-service/
├── api/              # gRPC API 定義
├── application/      # クエリハンドラー
├── domain/           # 読み取りモデル
├── infrastructure/   # データアクセス、キャッシュ
└── main.rs          # エントリーポイント
```

### 詳細設計

#### API Layer

```rust
// api/grpc/progress_query.proto
service ProgressQueryService {
    // 進捗情報
    rpc GetUserProgress(GetUserProgressQuery) returns (UserProgressResponse);
    rpc GetItemProgress(GetItemProgressQuery) returns (ItemProgressResponse);
    rpc GetMultipleItemProgress(GetMultipleItemProgressQuery) returns (MultipleItemProgressResponse);
    
    // 復習管理
    rpc GetDueItems(GetDueItemsQuery) returns (DueItemsResponse);
    rpc GetUpcomingReviews(GetUpcomingReviewsQuery) returns (UpcomingReviewsResponse);
    rpc GetReviewCalendar(GetReviewCalendarQuery) returns (ReviewCalendarResponse);
    
    // 統計情報
    rpc GetLearningStats(GetLearningStatsQuery) returns (LearningStatsResponse);
    rpc GetSessionHistory(GetSessionHistoryQuery) returns (SessionHistoryResponse);
    rpc GetStreakInfo(GetStreakInfoQuery) returns (StreakInfoResponse);
    
    // 分析
    rpc GetDifficultyDistribution(GetDifficultyDistributionQuery) returns (DifficultyDistributionResponse);
    rpc GetLearningCurve(GetLearningCurveQuery) returns (LearningCurveResponse);
    rpc GetTimeAnalysis(GetTimeAnalysisQuery) returns (TimeAnalysisResponse);
    
    // ランキング
    rpc GetStreakRanking(GetRankingQuery) returns (RankingResponse);
    rpc GetItemCountRanking(GetRankingQuery) returns (RankingResponse);
}

message GetUserProgressQuery {
    string user_id = 1;
}

message UserProgressResponse {
    UserProgress progress = 1;
    LearningMilestone next_milestone = 2;
    repeated Achievement recent_achievements = 3;
}

message UserProgress {
    string user_id = 1;
    uint32 total_items_learned = 2;
    uint32 items_in_review = 3;
    uint32 items_due_today = 4;
    uint32 items_overdue = 5;
    float average_recall_rate = 6;
    uint32 current_streak = 7;
    uint32 longest_streak = 8;
    uint64 total_study_time_minutes = 9;
    string last_study_date = 10;
    string next_review_date = 11;
    float mastery_percentage = 12;
}

message GetDueItemsQuery {
    string user_id = 1;
    DueItemsFilter filter = 2;
    uint32 limit = 3;
    uint32 offset = 4;
}

message DueItemsFilter {
    repeated string categories = 1;
    repeated string tags = 2;
    DifficultyRange difficulty_range = 3;
    bool include_overdue = 4;
}

message DueItemsResponse {
    repeated DueItem items = 1;
    uint32 total_count = 2;
    DueItemsSummary summary = 3;
}

message DueItem {
    string progress_id = 1;
    string item_id = 2;
    string scheduled_date = 3;
    uint32 overdue_days = 4;
    float easiness_factor = 5;
    uint32 repetition_number = 6;
    ItemSummary item_summary = 7;
}

message GetLearningStatsQuery {
    string user_id = 1;
    StatsPeriod period = 2;
    string start_date = 3;
    string end_date = 4;
}

enum StatsPeriod {
    DAILY = 0;
    WEEKLY = 1;
    MONTHLY = 2;
    YEARLY = 3;
    CUSTOM = 4;
}

message LearningStatsResponse {
    LearningStats stats = 1;
    repeated DailyStats daily_breakdown = 2;
    LearningTrends trends = 3;
}

message LearningStats {
    string user_id = 1;
    StatsPeriod period = 2;
    uint32 items_studied = 3;
    uint32 items_mastered = 4;
    uint32 new_items = 5;
    uint32 review_items = 6;
    uint32 study_sessions = 7;
    uint64 total_study_time_minutes = 8;
    float average_session_length = 9;
    RecallDistribution recall_distribution = 10;
    TimeDistribution time_distribution = 11;
}

message RecallDistribution {
    uint32 perfect = 1;        // Quality 5
    uint32 correct_easy = 2;   // Quality 4
    uint32 correct_hard = 3;   // Quality 3
    uint32 incorrect_easy = 4; // Quality 2
    uint32 incorrect_hard = 5; // Quality 1
    uint32 blackout = 6;       // Quality 0
}
```

#### Domain Layer

```rust
// domain/read_models/user_progress.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgress {
    pub user_id: UserId,
    pub total_items_learned: u32,
    pub items_in_review: u32,
    pub items_due_today: u32,
    pub items_overdue: u32,
    pub average_recall_rate: f32,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub total_study_time_minutes: u64,
    pub last_study_date: Option<NaiveDate>,
    pub next_review_date: Option<DateTime<Utc>>,
    pub mastery_percentage: f32,
    pub updated_at: DateTime<Utc>,
}

impl UserProgress {
    pub fn calculate_mastery_percentage(&self) -> f32 {
        if self.total_items_learned == 0 {
            return 0.0;
        }
        
        // 「マスター」の定義: 連続3回以上正解し、間隔が30日以上
        let mastered_threshold = 30;
        // この計算は ItemProgress の集計から取得
        0.0 // プレースホルダー
    }
    
    pub fn get_achievement_progress(&self) -> Vec<AchievementProgress> {
        vec![
            AchievementProgress {
                achievement_type: AchievementType::StreakMilestone,
                current_value: self.current_streak,
                target_value: self.next_streak_milestone(),
                percentage: self.streak_milestone_percentage(),
            },
            AchievementProgress {
                achievement_type: AchievementType::ItemCountMilestone,
                current_value: self.total_items_learned,
                target_value: self.next_item_milestone(),
                percentage: self.item_milestone_percentage(),
            },
        ]
    }
}

// domain/read_models/item_progress.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemProgress {
    pub progress_id: ProgressId,
    pub user_id: UserId,
    pub item_id: ItemId,
    pub repetition_number: u32,
    pub easiness_factor: f32,
    pub interval_days: f32,
    pub next_review_date: DateTime<Utc>,
    pub last_reviewed: DateTime<Utc>,
    pub total_reviews: u32,
    pub successful_reviews: u32,
    pub average_response_time_ms: u64,
    pub stability: f32,  // 0.0-1.0: 記憶の安定性
    pub difficulty: f32, // 0.0-1.0: 項目の難易度
}

impl ItemProgress {
    pub fn is_due(&self) -> bool {
        self.next_review_date <= Utc::now()
    }
    
    pub fn overdue_days(&self) -> u32 {
        if !self.is_due() {
            return 0;
        }
        
        let overdue_duration = Utc::now() - self.next_review_date;
        overdue_duration.num_days().max(0) as u32
    }
    
    pub fn recall_rate(&self) -> f32 {
        if self.total_reviews == 0 {
            return 0.0;
        }
        
        self.successful_reviews as f32 / self.total_reviews as f32
    }
    
    pub fn is_mastered(&self) -> bool {
        self.repetition_number >= 3 && self.interval_days >= 30.0
    }
}

// domain/read_models/learning_stats.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStats {
    pub user_id: UserId,
    pub period: StatsPeriod,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub items_studied: u32,
    pub items_mastered: u32,
    pub new_items: u32,
    pub review_items: u32,
    pub study_sessions: u32,
    pub total_study_time_minutes: u64,
    pub average_session_length: f32,
    pub recall_distribution: RecallDistribution,
    pub time_distribution: TimeDistribution,
    pub best_performance_time: Option<u8>, // 0-23時
    pub most_productive_day: Option<Weekday>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDistribution {
    pub by_hour: HashMap<u8, u32>,      // 0-23時
    pub by_weekday: HashMap<Weekday, u32>,
    pub morning_percentage: f32,   // 6-12時
    pub afternoon_percentage: f32, // 12-18時
    pub evening_percentage: f32,   // 18-24時
    pub night_percentage: f32,     // 0-6時
}

// domain/read_models/ranking.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingEntry {
    pub rank: u32,
    pub user_id: UserId,
    pub username: String,
    pub value: u32,
    pub change_from_previous: i32, // 前回からの順位変動
    pub is_current_user: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ranking {
    pub ranking_type: RankingType,
    pub period: RankingPeriod,
    pub entries: Vec<RankingEntry>,
    pub current_user_rank: Option<u32>,
    pub total_participants: u32,
    pub updated_at: DateTime<Utc>,
}
```

#### Application Layer

```rust
// application/query_handlers/get_user_progress_handler.rs
pub struct GetUserProgressHandler {
    repository: Arc<dyn ProgressReadRepository>,
    cache: Arc<dyn CacheService>,
}

impl GetUserProgressHandler {
    const CACHE_KEY_PREFIX: &'static str = "user_progress:";
    const CACHE_TTL: Duration = Duration::from_secs(300); // 5分
    
    pub async fn handle(
        &self,
        query: GetUserProgressQuery,
    ) -> Result<UserProgressResponse, QueryError> {
        // 1. キャッシュチェック
        let cache_key = format!("{}{}", Self::CACHE_KEY_PREFIX, query.user_id);
        if let Some(cached) = self.cache.get::<UserProgress>(&cache_key).await? {
            return Ok(self.build_response(cached));
        }
        
        // 2. データベースから取得
        let progress = self.repository
            .get_user_progress(&query.user_id)
            .await?
            .ok_or(QueryError::NotFound)?;
        
        // 3. 追加情報の取得
        let next_milestone = self.calculate_next_milestone(&progress).await?;
        let recent_achievements = self.get_recent_achievements(&query.user_id).await?;
        
        // 4. キャッシュに保存
        self.cache.set(&cache_key, &progress, Self::CACHE_TTL).await?;
        
        // 5. レスポンス構築
        Ok(UserProgressResponse {
            progress,
            next_milestone,
            recent_achievements,
        })
    }
    
    async fn calculate_next_milestone(
        &self,
        progress: &UserProgress,
    ) -> Result<Option<LearningMilestone>, QueryError> {
        let milestones = vec![
            (10, "初心者"),
            (50, "学習者"),
            (100, "常連"),
            (500, "エキスパート"),
            (1000, "マスター"),
        ];
        
        for (target, title) in milestones {
            if progress.total_items_learned < target {
                return Ok(Some(LearningMilestone {
                    milestone_type: MilestoneType::ItemCount,
                    target_value: target,
                    current_value: progress.total_items_learned,
                    title: title.to_string(),
                    reward_description: format!("{}バッジを獲得", title),
                }));
            }
        }
        
        Ok(None)
    }
}

// application/query_handlers/get_due_items_handler.rs
pub struct GetDueItemsHandler {
    repository: Arc<dyn ProgressReadRepository>,
    vocabulary_client: Arc<dyn VocabularyQueryClient>,
    cache: Arc<dyn CacheService>,
}

impl GetDueItemsHandler {
    pub async fn handle(
        &self,
        query: GetDueItemsQuery,
    ) -> Result<DueItemsResponse, QueryError> {
        // 1. 期限切れアイテムの取得
        let due_items = self.repository
            .get_due_items(&query.user_id, Utc::now())
            .await?;
        
        // 2. フィルタリング
        let filtered_items = self.apply_filters(due_items, &query.filter)?;
        
        // 3. ページネーション
        let total_count = filtered_items.len() as u32;
        let paginated = filtered_items
            .into_iter()
            .skip(query.offset as usize)
            .take(query.limit as usize)
            .collect::<Vec<_>>();
        
        // 4. 語彙情報の取得（バッチ）
        let item_ids: Vec<_> = paginated.iter()
            .map(|p| p.item_id.clone())
            .collect();
        
        let item_summaries = self.vocabulary_client
            .get_item_summaries(&item_ids)
            .await?;
        
        // 5. レスポンス構築
        let due_items = paginated.into_iter()
            .zip(item_summaries)
            .map(|(progress, summary)| DueItem {
                progress_id: progress.progress_id,
                item_id: progress.item_id,
                scheduled_date: progress.next_review_date,
                overdue_days: progress.overdue_days(),
                easiness_factor: progress.easiness_factor,
                repetition_number: progress.repetition_number,
                item_summary: summary,
            })
            .collect();
        
        // 6. サマリー計算
        let summary = self.calculate_summary(&due_items);
        
        Ok(DueItemsResponse {
            items: due_items,
            total_count,
            summary,
        })
    }
}

// application/query_handlers/get_learning_stats_handler.rs
pub struct GetLearningStatsHandler {
    repository: Arc<dyn StatsReadRepository>,
    cache: Arc<dyn CacheService>,
}

impl GetLearningStatsHandler {
    const CACHE_TTL: Duration = Duration::from_secs(3600); // 1時間
    
    pub async fn handle(
        &self,
        query: GetLearningStatsQuery,
    ) -> Result<LearningStatsResponse, QueryError> {
        let (start_date, end_date) = self.calculate_date_range(&query)?;
        
        // 1. 統計情報の取得
        let stats = self.repository
            .get_learning_stats(&query.user_id, start_date, end_date)
            .await?;
        
        // 2. 日次内訳の取得
        let daily_breakdown = self.repository
            .get_daily_stats(&query.user_id, start_date, end_date)
            .await?;
        
        // 3. トレンド分析
        let trends = self.analyze_trends(&stats, &daily_breakdown)?;
        
        Ok(LearningStatsResponse {
            stats,
            daily_breakdown,
            trends,
        })
    }
    
    fn analyze_trends(
        &self,
        stats: &LearningStats,
        daily: &[DailyStats],
    ) -> Result<LearningTrends, QueryError> {
        // 前期間との比較
        let mid_point = daily.len() / 2;
        let first_half: Vec<_> = daily[..mid_point].to_vec();
        let second_half: Vec<_> = daily[mid_point..].to_vec();
        
        let first_half_avg = first_half.iter()
            .map(|d| d.items_studied)
            .sum::<u32>() as f32 / first_half.len() as f32;
            
        let second_half_avg = second_half.iter()
            .map(|d| d.items_studied)
            .sum::<u32>() as f32 / second_half.len() as f32;
        
        let growth_rate = if first_half_avg > 0.0 {
            ((second_half_avg - first_half_avg) / first_half_avg) * 100.0
        } else {
            0.0
        };
        
        Ok(LearningTrends {
            growth_rate,
            consistency_score: self.calculate_consistency(daily),
            peak_performance_day: self.find_peak_day(daily),
            improvement_areas: self.identify_improvement_areas(stats),
        })
    }
}
```

#### Infrastructure Layer

```rust
// infrastructure/repositories/postgres_progress_read_repository.rs
pub struct PostgresProgressReadRepository {
    pool: PgPool,
}

#[async_trait]
impl ProgressReadRepository for PostgresProgressReadRepository {
    async fn get_user_progress(
        &self,
        user_id: &UserId,
    ) -> Result<Option<UserProgress>, RepositoryError> {
        let progress = sqlx::query_as!(
            UserProgressRow,
            r#"
            SELECT 
                user_id,
                total_items_learned,
                items_in_review,
                items_due_today,
                items_overdue,
                average_recall_rate,
                current_streak,
                longest_streak,
                total_study_time_minutes,
                last_study_date,
                next_review_date,
                mastery_percentage,
                updated_at
            FROM user_progress_view
            WHERE user_id = $1
            "#,
            user_id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(progress.map(UserProgress::from))
    }
    
    async fn get_due_items(
        &self,
        user_id: &UserId,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<ItemProgress>, RepositoryError> {
        let items = sqlx::query_as!(
            ItemProgressRow,
            r#"
            SELECT 
                progress_id,
                user_id,
                item_id,
                repetition_number,
                easiness_factor,
                interval_days,
                next_review_date,
                last_reviewed,
                total_reviews,
                successful_reviews,
                average_response_time_ms,
                stability,
                difficulty
            FROM item_progress_view
            WHERE user_id = $1 
            AND next_review_date <= $2
            ORDER BY next_review_date ASC
            "#,
            user_id.as_str(),
            as_of
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(items.into_iter().map(ItemProgress::from).collect())
    }
}

// infrastructure/cache/redis_cache_service.rs
pub struct RedisCacheService {
    client: redis::Client,
    serializer: Arc<dyn Serializer>,
}

#[async_trait]
impl CacheService for RedisCacheService {
    async fn get<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let data: Option<Vec<u8>> = conn.get(key).await?;
        
        match data {
            Some(bytes) => {
                let value = self.serializer.deserialize(&bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
    
    async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let bytes = self.serializer.serialize(value)?;
        
        conn.set_ex(key, bytes, ttl.as_secs() as usize).await?;
        
        Ok(())
    }
    
    async fn invalidate_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let keys: Vec<String> = conn.keys(pattern).await?;
        
        if !keys.is_empty() {
            conn.del(keys).await?;
        }
        
        Ok(())
    }
}

// infrastructure/clients/vocabulary_query_client.rs
pub struct VocabularyQueryGrpcClient {
    client: VocabularyQueryServiceClient<Channel>,
}

#[async_trait]
impl VocabularyQueryClient for VocabularyQueryGrpcClient {
    async fn get_item_summaries(
        &self,
        item_ids: &[ItemId],
    ) -> Result<Vec<ItemSummary>, ClientError> {
        let request = GetItemSummariesRequest {
            item_ids: item_ids.iter().map(|id| id.to_string()).collect(),
        };
        
        let response = self.client
            .get_item_summaries(request)
            .await?
            .into_inner();
        
        Ok(response.summaries.into_iter()
            .map(ItemSummary::from)
            .collect())
    }
}
```

## データベース設計

### Read Model テーブル

```sql
-- ユーザー進捗ビュー（マテリアライズドビュー）
CREATE MATERIALIZED VIEW user_progress_view AS
WITH progress_summary AS (
    SELECT 
        user_id,
        COUNT(DISTINCT item_id) as total_items_learned,
        COUNT(DISTINCT CASE WHEN next_review_date > NOW() THEN item_id END) as items_in_review,
        COUNT(DISTINCT CASE WHEN DATE(next_review_date) = CURRENT_DATE THEN item_id END) as items_due_today,
        COUNT(DISTINCT CASE WHEN next_review_date < NOW() THEN item_id END) as items_overdue,
        AVG(CASE WHEN total_reviews > 0 THEN successful_reviews::float / total_reviews ELSE 0 END) as average_recall_rate
    FROM item_progress_view
    GROUP BY user_id
),
streak_info AS (
    SELECT 
        user_id,
        current_streak,
        longest_streak,
        last_study_date
    FROM user_streak_view
),
study_time AS (
    SELECT 
        user_id,
        SUM(duration_minutes) as total_study_time_minutes
    FROM learning_session_view
    GROUP BY user_id
),
next_review AS (
    SELECT 
        user_id,
        MIN(next_review_date) as next_review_date
    FROM item_progress_view
    WHERE next_review_date > NOW()
    GROUP BY user_id
)
SELECT 
    p.user_id,
    p.total_items_learned,
    p.items_in_review,
    p.items_due_today,
    p.items_overdue,
    p.average_recall_rate,
    COALESCE(s.current_streak, 0) as current_streak,
    COALESCE(s.longest_streak, 0) as longest_streak,
    COALESCE(t.total_study_time_minutes, 0) as total_study_time_minutes,
    s.last_study_date,
    n.next_review_date,
    CASE 
        WHEN p.total_items_learned = 0 THEN 0
        ELSE (SELECT COUNT(*)::float FROM item_progress_view ipv 
              WHERE ipv.user_id = p.user_id 
              AND ipv.repetition_number >= 3 
              AND ipv.interval_days >= 30) / p.total_items_learned * 100
    END as mastery_percentage,
    NOW() as updated_at
FROM progress_summary p
LEFT JOIN streak_info s ON p.user_id = s.user_id
LEFT JOIN study_time t ON p.user_id = t.user_id
LEFT JOIN next_review n ON p.user_id = n.user_id;

-- インデックス
CREATE UNIQUE INDEX idx_user_progress_view_user_id ON user_progress_view(user_id);

-- 項目進捗ビュー
CREATE TABLE item_progress_view (
    progress_id UUID PRIMARY KEY,
    user_id VARCHAR(50) NOT NULL,
    item_id VARCHAR(50) NOT NULL,
    repetition_number INTEGER NOT NULL,
    easiness_factor REAL NOT NULL,
    interval_days REAL NOT NULL,
    next_review_date TIMESTAMPTZ NOT NULL,
    last_reviewed TIMESTAMPTZ NOT NULL,
    total_reviews INTEGER NOT NULL,
    successful_reviews INTEGER NOT NULL,
    average_response_time_ms BIGINT NOT NULL,
    stability REAL NOT NULL, -- 0.0-1.0
    difficulty REAL NOT NULL, -- 0.0-1.0
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- インデックス
CREATE INDEX idx_item_progress_user_item ON item_progress_view(user_id, item_id);
CREATE INDEX idx_item_progress_user_next_review ON item_progress_view(user_id, next_review_date);
CREATE INDEX idx_item_progress_next_review ON item_progress_view(next_review_date);

-- 統計テーブル
CREATE TABLE learning_stats_daily (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(50) NOT NULL,
    date DATE NOT NULL,
    items_studied INTEGER NOT NULL,
    items_mastered INTEGER NOT NULL,
    new_items INTEGER NOT NULL,
    review_items INTEGER NOT NULL,
    study_sessions INTEGER NOT NULL,
    total_study_time_minutes INTEGER NOT NULL,
    recall_quality_0 INTEGER NOT NULL DEFAULT 0,
    recall_quality_1 INTEGER NOT NULL DEFAULT 0,
    recall_quality_2 INTEGER NOT NULL DEFAULT 0,
    recall_quality_3 INTEGER NOT NULL DEFAULT 0,
    recall_quality_4 INTEGER NOT NULL DEFAULT 0,
    recall_quality_5 INTEGER NOT NULL DEFAULT 0,
    hour_distribution JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, date)
);

-- インデックス
CREATE INDEX idx_learning_stats_daily_user_date ON learning_stats_daily(user_id, date DESC);
```

### キャッシュ戦略

```yaml
cache_policies:
  user_progress:
    ttl: 300  # 5分
    invalidation:
      - on_event: ItemStudied
      - on_event: ItemRecalled
      - on_event: SessionCompleted
  
  due_items:
    ttl: 60   # 1分（リアルタイム性重視）
    invalidation:
      - on_event: ItemRecalled
      - on_event: ReviewScheduled
  
  learning_stats:
    ttl: 3600 # 1時間
    invalidation:
      - on_event: SessionCompleted
      - scheduled: "0 * * * *"  # 毎時0分
  
  ranking:
    ttl: 600  # 10分
    invalidation:
      - scheduled: "*/10 * * * *"  # 10分ごと
```

## パフォーマンス最適化

### クエリ最適化

1. **N+1 問題の回避**
   - 語彙情報のバッチ取得
   - DataLoader パターンの適用

2. **適切なインデックス**
   - user_id + next_review_date の複合インデックス
   - 統計クエリ用の日付インデックス

3. **マテリアライズドビュー**
   - user_progress_view: 5分ごとに更新
   - ranking_view: 10分ごとに更新

### キャッシュウォーミング

```rust
pub struct CacheWarmingService {
    handlers: Vec<Arc<dyn QueryHandler>>,
    cache: Arc<dyn CacheService>,
}

impl CacheWarmingService {
    pub async fn warm_user_cache(&self, user_id: &UserId) -> Result<()> {
        // ユーザー進捗のプリロード
        let progress_query = GetUserProgressQuery {
            user_id: user_id.to_string(),
        };
        self.handlers.user_progress.handle(progress_query).await?;
        
        // 期限切れアイテムのプリロード
        let due_items_query = GetDueItemsQuery {
            user_id: user_id.to_string(),
            filter: Default::default(),
            limit: 20,
            offset: 0,
        };
        self.handlers.due_items.handle(due_items_query).await?;
        
        Ok(())
    }
}
```

## 設定とデプロイメント

### 環境変数

```yaml
# データベース
DATABASE_URL: postgres://user:pass@postgres:5432/progress_read
DATABASE_MAX_CONNECTIONS: 25
DATABASE_MIN_CONNECTIONS: 5

# Redis
REDIS_URL: redis://redis:6379
REDIS_MAX_CONNECTIONS: 10

# gRPC
SERVICE_PORT: 50062
GRPC_MAX_MESSAGE_SIZE: 10485760  # 10MB

# 外部サービス
VOCABULARY_SERVICE_URL: http://vocabulary-query-service:50052

# 監視
TRACE_ENDPOINT: https://cloudtrace.googleapis.com
METRICS_PORT: 9091
```

### Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY proto ./proto
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/progress-query-service /usr/local/bin/
EXPOSE 50062 9091
CMD ["progress-query-service"]
```

### Cloud Run デプロイメント

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: progress-query-service
spec:
  template:
    metadata:
      annotations:
        run.googleapis.com/cloudsql-instances: project:region:instance
        run.googleapis.com/vpc-connector: projects/PROJECT/locations/REGION/connectors/CONNECTOR
    spec:
      serviceAccountName: progress-service
      containers:
      - image: gcr.io/effect-project/progress-query-service:latest
        ports:
        - containerPort: 50062
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: progress-secrets
              key: read-database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: progress-secrets
              key: redis-url
        resources:
          limits:
            memory: "1Gi"
            cpu: "2000m"
        livenessProbe:
          grpc:
            port: 50062
          initialDelaySeconds: 10
        readinessProbe:
          grpc:
            port: 50062
          initialDelaySeconds: 5
```

## エラーハンドリング

### クエリエラー

```rust
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Resource not found")]
    NotFound,
    
    #[error("Invalid query parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Cache error: {0}")]
    CacheError(#[from] CacheError),
    
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

// リトライ戦略
pub struct RetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
        }
    }
}
```

## 監視とメトリクス

### 主要メトリクス

```rust
lazy_static! {
    static ref QUERY_DURATION: HistogramVec = register_histogram_vec!(
        "progress_query_duration_seconds",
        "Query execution time",
        &["query_type", "status"]
    ).unwrap();
    
    static ref CACHE_HIT_RATE: GaugeVec = register_gauge_vec!(
        "progress_cache_hit_rate",
        "Cache hit rate by query type",
        &["query_type"]
    ).unwrap();
    
    static ref QUERY_COUNT: IntCounterVec = register_int_counter_vec!(
        "progress_query_total",
        "Total number of queries",
        &["query_type", "status"]
    ).unwrap();
}
```

### アラート設定

```yaml
alerts:
  - name: HighQueryLatency
    condition: progress_query_duration_seconds_p95 > 0.1
    severity: warning
    
  - name: LowCacheHitRate
    condition: progress_cache_hit_rate < 0.8
    severity: warning
    
  - name: HighErrorRate
    condition: rate(progress_query_total{status="error"}[5m]) > 0.05
    severity: critical
```

## 更新履歴

- 2025-08-03: 初版作成
