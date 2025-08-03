# progress-projection-service 設計書

## 概要

progress-projection-service は、Progress Context のイベントプロジェクションを担当します。
Event Store から発行されるイベントを消費し、様々な Read Model を構築・更新します。
また、他の Context からのイベントも処理し、進捗情報に反映させます。

## 責務

1. **イベント消費と処理**

   - Progress Context のイベント処理
   - Vocabulary Context のイベント処理
   - エラーハンドリングとリトライ

2. **Read Model の更新**

   - UserProgressView の維持
   - ItemProgressView の更新
   - 統計情報の集計

3. **データ整合性の保証**

   - イベント順序の保証
   - 冪等性の確保
   - 欠落イベントの検出

4. **パフォーマンス最適化**
   - バッチ処理
   - 並列処理
   - キャッシュ無効化

## アーキテクチャ

### レイヤー構造

```
progress-projection-service/
├── application/      # イベントハンドラー
├── domain/           # プロジェクションロジック
├── infrastructure/   # Pub/Sub、データベース
└── main.rs          # エントリーポイント
```

### 詳細設計

#### Application Layer

```rust
// application/event_handlers/mod.rs
pub trait EventHandler: Send + Sync {
    type Event;

    async fn handle(&self, event: Self::Event) -> Result<(), ProjectionError>;
    fn event_type(&self) -> &'static str;
}

// application/event_handlers/learning_session_started_handler.rs
pub struct LearningSessionStartedHandler {
    user_progress_projector: Arc<UserProgressProjector>,
    session_repository: Arc<dyn SessionWriteRepository>,
}

#[async_trait]
impl EventHandler for LearningSessionStartedHandler {
    type Event = LearningSessionStarted;

    async fn handle(&self, event: Self::Event) -> Result<(), ProjectionError> {
        // 1. セッション情報を保存
        let session = LearningSessionProjection {
            session_id: event.session_id,
            user_id: event.user_id,
            session_type: event.session_type,
            started_at: event.occurred_at,
            planned_items: event.planned_items,
            actual_items: 0,
            status: SessionStatus::Active,
        };

        self.session_repository.save(&session).await?;

        // 2. ユーザー進捗を更新
        self.user_progress_projector
            .on_session_started(&event.user_id)
            .await?;

        Ok(())
    }

    fn event_type(&self) -> &'static str {
        "LearningSessionStarted"
    }
}

// application/event_handlers/item_recalled_handler.rs
pub struct ItemRecalledHandler {
    item_progress_projector: Arc<ItemProgressProjector>,
    stats_projector: Arc<StatsProjector>,
    cache_invalidator: Arc<CacheInvalidator>,
}

#[async_trait]
impl EventHandler for ItemRecalledHandler {
    type Event = ItemRecalled;

    async fn handle(&self, event: Self::Event) -> Result<(), ProjectionError> {
        // 1. 項目進捗を更新
        self.item_progress_projector
            .on_item_recalled(&event)
            .await?;

        // 2. 統計情報を更新
        self.stats_projector
            .record_recall(&event)
            .await?;

        // 3. 関連キャッシュを無効化
        self.cache_invalidator
            .invalidate_user_progress(&event.user_id)
            .await?;

        self.cache_invalidator
            .invalidate_due_items(&event.user_id)
            .await?;

        Ok(())
    }

    fn event_type(&self) -> &'static str {
        "ItemRecalled"
    }
}

// application/event_handlers/review_scheduled_handler.rs
pub struct ReviewScheduledHandler {
    item_progress_projector: Arc<ItemProgressProjector>,
    review_calendar_projector: Arc<ReviewCalendarProjector>,
}

#[async_trait]
impl EventHandler for ReviewScheduledHandler {
    type Event = ReviewScheduled;

    async fn handle(&self, event: Self::Event) -> Result<(), ProjectionError> {
        // 1. 項目進捗の次回復習日を更新
        self.item_progress_projector
            .update_next_review(&event)
            .await?;

        // 2. 復習カレンダーを更新
        self.review_calendar_projector
            .add_review(&event.user_id, &event.item_id, event.scheduled_for)
            .await?;

        Ok(())
    }

    fn event_type(&self) -> &'static str {
        "ReviewScheduled"
    }
}

// application/event_handlers/vocabulary_item_created_handler.rs
pub struct VocabularyItemCreatedHandler {
    item_mapping_repository: Arc<dyn ItemMappingRepository>,
}

#[async_trait]
impl EventHandler for VocabularyItemCreatedHandler {
    type Event = VocabularyItemCreated;

    async fn handle(&self, event: Self::Event) -> Result<(), ProjectionError> {
        // Vocabulary Item が作成されたら、マッピング情報を保存
        let mapping = ItemMapping {
            item_id: event.item_id,
            entry_id: event.entry_id,
            title: event.title,
            category: event.category,
            tags: event.tags,
        };

        self.item_mapping_repository.save(&mapping).await?;

        Ok(())
    }

    fn event_type(&self) -> &'static str {
        "VocabularyItemCreated"
    }
}
```

#### Domain Layer

```rust
// domain/projectors/user_progress_projector.rs
pub struct UserProgressProjector {
    repository: Arc<dyn UserProgressWriteRepository>,
}

impl UserProgressProjector {
    pub async fn on_session_started(
        &self,
        user_id: &UserId,
    ) -> Result<(), ProjectionError> {
        let mut progress = self.repository
            .find_by_user_id(user_id)
            .await?
            .unwrap_or_else(|| UserProgressProjection::new(user_id.clone()));

        // アクティブセッションをインクリメント
        progress.active_sessions += 1;
        progress.last_activity = Utc::now();

        self.repository.save(&progress).await?;
        Ok(())
    }

    pub async fn on_item_studied(
        &self,
        user_id: &UserId,
        item_id: &ItemId,
        is_new: bool,
    ) -> Result<(), ProjectionError> {
        let mut progress = self.repository
            .find_by_user_id(user_id)
            .await?
            .ok_or(ProjectionError::UserNotFound)?;

        if is_new {
            progress.total_items_learned += 1;
            progress.items_in_review += 1;
        }

        progress.last_activity = Utc::now();
        progress.updated_at = Utc::now();

        self.repository.save(&progress).await?;
        Ok(())
    }

    pub async fn calculate_mastery_percentage(
        &self,
        user_id: &UserId,
    ) -> Result<f32, ProjectionError> {
        let item_progresses = self.repository
            .get_all_item_progress(user_id)
            .await?;

        if item_progresses.is_empty() {
            return Ok(0.0);
        }

        let mastered_count = item_progresses.iter()
            .filter(|p| p.is_mastered())
            .count();

        Ok((mastered_count as f32 / item_progresses.len() as f32) * 100.0)
    }
}

// domain/projectors/item_progress_projector.rs
pub struct ItemProgressProjector {
    repository: Arc<dyn ItemProgressWriteRepository>,
}

impl ItemProgressProjector {
    pub async fn on_item_recalled(
        &self,
        event: &ItemRecalled,
    ) -> Result<(), ProjectionError> {
        let mut progress = self.repository
            .find_by_id(&event.progress_id)
            .await?
            .unwrap_or_else(|| {
                ItemProgressProjection::new(
                    event.progress_id.clone(),
                    event.user_id.clone(),
                    event.item_id.clone(),
                )
            });

        // 統計を更新
        progress.total_reviews += 1;
        if event.recall_quality as u8 >= 3 {
            progress.successful_reviews += 1;
        }

        // 平均応答時間を更新
        progress.update_average_response_time(event.response_time_ms);

        // 安定性と難易度を計算
        progress.stability = self.calculate_stability(&progress);
        progress.difficulty = self.calculate_difficulty(&progress);

        progress.last_reviewed = event.occurred_at;
        progress.updated_at = Utc::now();

        self.repository.save(&progress).await?;
        Ok(())
    }

    pub async fn update_next_review(
        &self,
        event: &ReviewScheduled,
    ) -> Result<(), ProjectionError> {
        let mut progress = self.repository
            .find_by_id(&event.progress_id)
            .await?
            .ok_or(ProjectionError::ItemProgressNotFound)?;

        progress.repetition_number = event.repetition_number;
        progress.easiness_factor = event.easiness_factor;
        progress.interval_days = event.interval_days;
        progress.next_review_date = event.scheduled_for;
        progress.updated_at = Utc::now();

        self.repository.save(&progress).await?;
        Ok(())
    }

    fn calculate_stability(&self, progress: &ItemProgressProjection) -> f32 {
        if progress.total_reviews == 0 {
            return 0.5; // デフォルト
        }

        // 成功率と復習間隔から安定性を計算
        let success_rate = progress.successful_reviews as f32 / progress.total_reviews as f32;
        let interval_factor = (progress.interval_days / 30.0).min(1.0);

        (success_rate * 0.7 + interval_factor * 0.3).min(1.0)
    }

    fn calculate_difficulty(&self, progress: &ItemProgressProjection) -> f32 {
        // Easiness Factor から難易度を計算（逆相関）
        // EF: 1.3 (最小) → difficulty: 1.0 (最難)
        // EF: 2.5 (デフォルト) → difficulty: 0.5
        // EF: 4.0 (最大) → difficulty: 0.0 (最易)

        let normalized_ef = (progress.easiness_factor - 1.3) / (4.0 - 1.3);
        (1.0 - normalized_ef).max(0.0).min(1.0)
    }
}

// domain/projectors/stats_projector.rs
pub struct StatsProjector {
    repository: Arc<dyn StatsWriteRepository>,
}

impl StatsProjector {
    pub async fn record_recall(
        &self,
        event: &ItemRecalled,
    ) -> Result<(), ProjectionError> {
        let date = event.occurred_at.date_naive();
        let hour = event.occurred_at.hour();

        // 日次統計を更新
        let mut daily_stats = self.repository
            .find_daily_stats(&event.user_id, date)
            .await?
            .unwrap_or_else(|| DailyStats::new(event.user_id.clone(), date));

        // 想起品質ごとのカウント
        match event.recall_quality {
            RecallQuality::Blackout => daily_stats.recall_quality_0 += 1,
            RecallQuality::IncorrectDifficult => daily_stats.recall_quality_1 += 1,
            RecallQuality::IncorrectEasy => daily_stats.recall_quality_2 += 1,
            RecallQuality::CorrectDifficult => daily_stats.recall_quality_3 += 1,
            RecallQuality::CorrectEasy => daily_stats.recall_quality_4 += 1,
            RecallQuality::Perfect => daily_stats.recall_quality_5 += 1,
        }

        // 時間分布を更新
        daily_stats.increment_hour_count(hour as u8);

        self.repository.save_daily_stats(&daily_stats).await?;

        Ok(())
    }

    pub async fn complete_session(
        &self,
        event: &LearningSessionCompleted,
    ) -> Result<(), ProjectionError> {
        let date = event.ended_at.date_naive();

        let mut daily_stats = self.repository
            .find_daily_stats(&event.user_id, date)
            .await?
            .ok_or(ProjectionError::StatsNotFound)?;

        daily_stats.study_sessions += 1;
        daily_stats.total_study_time_minutes += event.duration_minutes;
        daily_stats.items_studied += event.items_studied;

        self.repository.save_daily_stats(&daily_stats).await?;

        // ストリーク更新
        self.update_streak(&event.user_id, date).await?;

        Ok(())
    }

    async fn update_streak(
        &self,
        user_id: &UserId,
        study_date: NaiveDate,
    ) -> Result<(), ProjectionError> {
        let mut streak = self.repository
            .find_streak(user_id)
            .await?
            .unwrap_or_else(|| UserStreak::new(user_id.clone()));

        if let Some(last_date) = streak.last_study_date {
            let days_diff = (study_date - last_date).num_days();

            match days_diff {
                0 => {}, // 同日の場合は何もしない
                1 => {
                    // 連続
                    streak.current_streak += 1;
                    if streak.current_streak > streak.longest_streak {
                        streak.longest_streak = streak.current_streak;
                    }
                },
                _ => {
                    // 途切れた
                    streak.current_streak = 1;
                }
            }
        } else {
            // 初回
            streak.current_streak = 1;
            streak.longest_streak = 1;
        }

        streak.last_study_date = Some(study_date);
        self.repository.save_streak(&streak).await?;

        Ok(())
    }
}
```

#### Infrastructure Layer

```rust
// infrastructure/event_consumer/pubsub_event_consumer.rs
pub struct PubSubEventConsumer {
    subscription: Subscription,
    handlers: HashMap<String, Box<dyn EventHandler<Event = DomainEvent>>>,
    error_handler: Arc<ErrorHandler>,
    metrics: Arc<Metrics>,
}

impl PubSubEventConsumer {
    pub async fn start(self) -> Result<(), ConsumerError> {
        let mut stream = self.subscription
            .subscribe()
            .await?;

        while let Some(message) = stream.next().await {
            let message = message?;
            let timer = self.metrics.processing_time.start_timer();

            match self.process_message(&message).await {
                Ok(_) => {
                    message.ack().await?;
                    self.metrics.processed_count.inc();
                }
                Err(e) => {
                    self.metrics.error_count.inc();

                    match self.error_handler.handle(&e, &message).await {
                        ErrorAction::Retry => {
                            message.nack().await?;
                        }
                        ErrorAction::DeadLetter => {
                            // Dead Letter Queue に送信
                            self.send_to_dlq(&message, &e).await?;
                            message.ack().await?;
                        }
                        ErrorAction::Skip => {
                            message.ack().await?;
                        }
                    }
                }
            }

            timer.observe_duration();
        }

        Ok(())
    }

    async fn process_message(&self, message: &PubSubMessage) -> Result<(), ProjectionError> {
        let event_type = message.attributes
            .get("event_type")
            .ok_or(ProjectionError::MissingEventType)?;

        let handler = self.handlers
            .get(event_type)
            .ok_or(ProjectionError::UnknownEventType(event_type.clone()))?;

        let event: DomainEvent = serde_json::from_slice(&message.data)?;

        handler.handle(event).await
    }
}

// infrastructure/repositories/postgres_write_repository.rs
pub struct PostgresItemProgressWriteRepository {
    pool: PgPool,
}

#[async_trait]
impl ItemProgressWriteRepository for PostgresItemProgressWriteRepository {
    async fn save(&self, progress: &ItemProgressProjection) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO item_progress_view (
                progress_id, user_id, item_id, repetition_number,
                easiness_factor, interval_days, next_review_date,
                last_reviewed, total_reviews, successful_reviews,
                average_response_time_ms, stability, difficulty,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (progress_id) DO UPDATE SET
                repetition_number = EXCLUDED.repetition_number,
                easiness_factor = EXCLUDED.easiness_factor,
                interval_days = EXCLUDED.interval_days,
                next_review_date = EXCLUDED.next_review_date,
                last_reviewed = EXCLUDED.last_reviewed,
                total_reviews = EXCLUDED.total_reviews,
                successful_reviews = EXCLUDED.successful_reviews,
                average_response_time_ms = EXCLUDED.average_response_time_ms,
                stability = EXCLUDED.stability,
                difficulty = EXCLUDED.difficulty,
                updated_at = EXCLUDED.updated_at
            "#,
            progress.progress_id.as_str(),
            progress.user_id.as_str(),
            progress.item_id.as_str(),
            progress.repetition_number as i32,
            progress.easiness_factor,
            progress.interval_days,
            progress.next_review_date,
            progress.last_reviewed,
            progress.total_reviews as i32,
            progress.successful_reviews as i32,
            progress.average_response_time_ms as i64,
            progress.stability,
            progress.difficulty,
            progress.created_at,
            progress.updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// infrastructure/cache/cache_invalidator.rs
pub struct RedisCacheInvalidator {
    client: redis::Client,
}

impl CacheInvalidator for RedisCacheInvalidator {
    async fn invalidate_user_progress(&self, user_id: &UserId) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;

        let pattern = format!("user_progress:{}", user_id);
        let keys: Vec<String> = conn.keys(&pattern).await?;

        if !keys.is_empty() {
            conn.del(keys).await?;
        }

        Ok(())
    }

    async fn invalidate_due_items(&self, user_id: &UserId) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;

        let pattern = format!("due_items:{}:*", user_id);
        let keys: Vec<String> = conn.keys(&pattern).await?;

        if !keys.is_empty() {
            conn.del(keys).await?;
        }

        Ok(())
    }
}
```

## エラーハンドリング

### エラー処理戦略

```rust
pub struct ErrorHandler {
    retry_policy: RetryPolicy,
    dlq_publisher: Arc<DeadLetterPublisher>,
}

impl ErrorHandler {
    pub async fn handle(
        &self,
        error: &ProjectionError,
        message: &PubSubMessage,
    ) -> ErrorAction {
        match error {
            // 一時的なエラー → リトライ
            ProjectionError::DatabaseConnection(_) |
            ProjectionError::CacheConnection(_) => {
                if self.should_retry(message) {
                    ErrorAction::Retry
                } else {
                    ErrorAction::DeadLetter
                }
            }

            // データ不整合 → Dead Letter
            ProjectionError::UserNotFound |
            ProjectionError::ItemProgressNotFound => {
                ErrorAction::DeadLetter
            }

            // 不正なイベント → スキップ
            ProjectionError::InvalidEventData(_) |
            ProjectionError::UnknownEventType(_) => {
                ErrorAction::Skip
            }

            _ => ErrorAction::DeadLetter
        }
    }

    fn should_retry(&self, message: &PubSubMessage) -> bool {
        let attempt_count = message.attributes
            .get("delivery_attempt")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);

        attempt_count <= self.retry_policy.max_attempts
    }
}

#[derive(Debug)]
pub enum ErrorAction {
    Retry,
    DeadLetter,
    Skip,
}
```

### Dead Letter Queue

```yaml
# Pub/Sub Dead Letter Topic
topics:
  - name: progress-events-dlq
    messageRetentionDuration: 30d

subscriptions:
  - name: progress-events-dlq-monitoring
    topic: progress-events-dlq
    ackDeadlineSeconds: 300
```

## パフォーマンス最適化

### バッチ処理

```rust
pub struct BatchProcessor {
    batch_size: usize,
    flush_interval: Duration,
    buffer: Arc<Mutex<Vec<ItemProgressProjection>>>,
    repository: Arc<dyn ItemProgressWriteRepository>,
}

impl BatchProcessor {
    pub async fn add(&self, item: ItemProgressProjection) -> Result<(), ProjectionError> {
        let mut buffer = self.buffer.lock().await;
        buffer.push(item);

        if buffer.len() >= self.batch_size {
            let items = std::mem::take(&mut *buffer);
            drop(buffer);
            self.flush(items).await?;
        }

        Ok(())
    }

    async fn flush(&self, items: Vec<ItemProgressProjection>) -> Result<(), ProjectionError> {
        if items.is_empty() {
            return Ok(());
        }

        self.repository.save_batch(&items).await?;
        Ok(())
    }

    pub async fn start_flush_timer(self: Arc<Self>) {
        let mut interval = tokio::time::interval(self.flush_interval);

        loop {
            interval.tick().await;

            let mut buffer = self.buffer.lock().await;
            if !buffer.is_empty() {
                let items = std::mem::take(&mut *buffer);
                drop(buffer);

                if let Err(e) = self.flush(items).await {
                    error!("Batch flush error: {}", e);
                }
            }
        }
    }
}
```

### 並列処理

```rust
pub struct ParallelEventProcessor {
    worker_count: usize,
    event_queue: Arc<AsyncQueue<DomainEvent>>,
    handlers: Arc<HashMap<String, Box<dyn EventHandler>>>,
}

impl ParallelEventProcessor {
    pub async fn start(self) -> Result<(), ProjectionError> {
        let mut workers = vec![];

        for worker_id in 0..self.worker_count {
            let queue = self.event_queue.clone();
            let handlers = self.handlers.clone();

            let worker = tokio::spawn(async move {
                loop {
                    match queue.pop().await {
                        Some(event) => {
                            if let Err(e) = process_event(event, &handlers).await {
                                error!("Worker {} error: {}", worker_id, e);
                            }
                        }
                        None => break,
                    }
                }
            });

            workers.push(worker);
        }

        futures::future::join_all(workers).await;
        Ok(())
    }
}
```

## 設定とデプロイメント

### 環境変数

```yaml
# Pub/Sub
PUBSUB_PROJECT_ID: effect-project
PUBSUB_SUBSCRIPTION: progress-projection
PUBSUB_MAX_MESSAGES: 100
PUBSUB_EMULATOR_HOST: localhost:8085 # 開発環境

# データベース
DATABASE_URL: postgres://user:pass@postgres:5432/progress_read
DATABASE_MAX_CONNECTIONS: 20

# Redis
REDIS_URL: redis://redis:6379
REDIS_POOL_SIZE: 10

# 処理設定
BATCH_SIZE: 100
FLUSH_INTERVAL_SECONDS: 5
WORKER_COUNT: 4

# 監視
METRICS_PORT: 9092
LOG_LEVEL: info
```

### Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/progress-projection-service /usr/local/bin/
EXPOSE 9092
CMD ["progress-projection-service"]
```

### Cloud Run デプロイメント

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: progress-projection-service
spec:
  template:
    metadata:
      annotations:
        run.googleapis.com/cpu-throttling: "false"
        run.googleapis.com/execution-environment: gen2
    spec:
      serviceAccountName: progress-projection
      containers:
        - image: gcr.io/effect-project/progress-projection-service:latest
          ports:
            - containerPort: 9092
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: progress-secrets
                  key: write-database-url
            - name: REDIS_URL
              valueFrom:
                secretKeyRef:
                  name: progress-secrets
                  key: redis-url
          resources:
            limits:
              memory: "1Gi"
              cpu: "2000m"
          startupProbe:
            httpGet:
              path: /health
              port: 9092
            periodSeconds: 10
            failureThreshold: 3
```

## 監視とメトリクス

### 主要メトリクス

```rust
lazy_static! {
    static ref EVENT_PROCESSING_TIME: HistogramVec = register_histogram_vec!(
        "projection_event_processing_seconds",
        "Event processing time by type",
        &["event_type"]
    ).unwrap();

    static ref EVENT_COUNT: IntCounterVec = register_int_counter_vec!(
        "projection_events_total",
        "Total events processed",
        &["event_type", "status"]
    ).unwrap();

    static ref PROJECTION_LAG: GaugeVec = register_gauge_vec!(
        "projection_lag_seconds",
        "Lag between event time and processing time",
        &["event_type"]
    ).unwrap();

    static ref BATCH_SIZE: Histogram = register_histogram!(
        "projection_batch_size",
        "Size of processed batches"
    ).unwrap();
}
```

### ヘルスチェック

```rust
#[derive(Serialize)]
struct HealthStatus {
    status: String,
    subscription_connected: bool,
    database_connected: bool,
    redis_connected: bool,
    lag_seconds: f64,
    pending_events: u64,
}

async fn health_check(deps: &Dependencies) -> HealthStatus {
    let subscription_connected = deps.subscription.is_connected().await;
    let database_connected = check_database(&deps.db_pool).await.is_ok();
    let redis_connected = check_redis(&deps.redis).await.is_ok();
    let lag_seconds = calculate_projection_lag(&deps.metrics).await;
    let pending_events = get_pending_events_count(&deps.subscription).await;

    let status = if subscription_connected && database_connected && redis_connected && lag_seconds < 10.0 {
        "healthy"
    } else if lag_seconds < 60.0 {
        "degraded"
    } else {
        "unhealthy"
    };

    HealthStatus {
        status: status.to_string(),
        subscription_connected,
        database_connected,
        redis_connected,
        lag_seconds,
        pending_events,
    }
}
```

### アラート設定

```yaml
alerts:
  - name: HighProjectionLag
    condition: projection_lag_seconds > 30
    severity: warning

  - name: CriticalProjectionLag
    condition: projection_lag_seconds > 300
    severity: critical

  - name: HighErrorRate
    condition: rate(projection_events_total{status="error"}[5m]) > 0.1
    severity: critical

  - name: PendingEventsBacklog
    condition: projection_pending_events > 10000
    severity: warning
```

## 更新履歴

- 2025-08-03: 初版作成
