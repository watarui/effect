# progress-command-service 設計書

## 概要

progress-command-service は、Progress Context の Write Model を管理し、学習進捗の記録と SM-2 アルゴリズムによる復習スケジュール計算を担当します。
すべての学習イベントを生成し、Event Store に永続化します。

## 責務

1. **学習セッション管理**

   - セッションの開始・終了
   - セッション内の学習記録
   - セッション統計の集計

2. **学習進捗の記録**

   - 項目の学習（ItemStudied）
   - 想起結果の記録（ItemRecalled）
   - 学習時間の追跡

3. **SM-2 アルゴリズムの実行**

   - 次回復習日の計算
   - Easiness Factor の更新
   - 復習間隔の最適化

4. **イベント生成と永続化**
   - ドメインイベントの生成
   - Event Store への保存
   - Event Bus への発行

## アーキテクチャ

### レイヤー構造

```
progress-command-service/
├── api/              # gRPC API 定義
├── application/      # コマンドハンドラー
├── domain/           # ドメインモデル、SM-2実装
├── infrastructure/   # Event Store、Pub/Sub連携
└── main.rs          # エントリーポイント
```

### 詳細設計

#### API Layer

```rust
// api/grpc/progress_command.proto
service ProgressCommandService {
    // セッション管理
    rpc StartLearningSession(StartSessionCommand) returns (StartSessionResponse);
    rpc CompleteLearningSession(CompleteSessionCommand) returns (CompleteSessionResponse);

    // 学習記録
    rpc RecordItemStudy(RecordStudyCommand) returns (RecordStudyResponse);
    rpc RecordItemRecall(RecordRecallCommand) returns (RecordRecallResponse);

    // 統計更新
    rpc UpdateStreak(UpdateStreakCommand) returns (UpdateStreakResponse);
    rpc RecordMilestone(RecordMilestoneCommand) returns (RecordMilestoneResponse);
}

message StartSessionCommand {
    string user_id = 1;
    SessionType session_type = 2;
    uint32 planned_items = 3;
    optional uint32 time_limit_minutes = 4;
    StudyContext context = 5;
}

message RecordRecallCommand {
    string progress_id = 1;
    string user_id = 2;
    string item_id = 3;
    string session_id = 4;
    RecallQuality quality = 5;  // 0-5
    uint64 response_time_ms = 6;
    bool hint_used = 7;
}

enum RecallQuality {
    BLACKOUT = 0;           // 完全に忘れた
    INCORRECT_DIFFICULT = 1; // 不正解（難しい）
    INCORRECT_EASY = 2;     // 不正解（簡単に思い出せそう）
    CORRECT_DIFFICULT = 3;  // 正解（難しい）
    CORRECT_EASY = 4;       // 正解（簡単）
    PERFECT = 5;            // 完璧
}
```

#### Domain Layer

```rust
// domain/aggregates/learning_session.rs
pub struct LearningSession {
    session_id: SessionId,
    user_id: UserId,
    session_type: SessionType,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    items_studied: Vec<ItemId>,
    items_recalled: HashMap<ItemId, RecallResult>,
    context: StudyContext,
    events: Vec<DomainEvent>,
}

impl LearningSession {
    pub fn start(
        user_id: UserId,
        session_type: SessionType,
        context: StudyContext,
    ) -> Result<Self, DomainError> {
        let session_id = SessionId::new();
        let mut session = Self {
            session_id,
            user_id,
            session_type,
            started_at: Utc::now(),
            ended_at: None,
            items_studied: vec![],
            items_recalled: HashMap::new(),
            context,
            events: vec![],
        };

        session.raise_event(DomainEvent::LearningSessionStarted {
            session_id,
            user_id,
            session_type,
            planned_items: 0, // 後で更新
            occurred_at: session.started_at,
        });

        Ok(session)
    }

    pub fn record_study(&mut self, item_id: ItemId) -> Result<(), DomainError> {
        if self.ended_at.is_some() {
            return Err(DomainError::SessionAlreadyEnded);
        }

        self.items_studied.push(item_id);

        self.raise_event(DomainEvent::ItemStudied {
            session_id: self.session_id,
            user_id: self.user_id,
            item_id,
            study_type: self.determine_study_type(&item_id),
            presentation_order: self.items_studied.len() as u32,
            occurred_at: Utc::now(),
        });

        Ok(())
    }
}

// domain/aggregates/progress.rs
pub struct Progress {
    progress_id: ProgressId,
    user_id: UserId,
    item_id: ItemId,
    repetition_number: u32,
    easiness_factor: f32,
    interval_days: f32,
    next_review_date: Option<DateTime<Utc>>,
    last_reviewed: Option<DateTime<Utc>>,
    total_reviews: u32,
    successful_reviews: u32,
    events: Vec<DomainEvent>,
}

impl Progress {
    pub fn record_recall(
        &mut self,
        quality: RecallQuality,
        response_time_ms: u64,
    ) -> Result<(), DomainError> {
        // SM-2 アルゴリズムの実行
        let sm2_result = self.calculate_sm2(quality);

        // 進捗の更新
        self.repetition_number = sm2_result.repetition_number;
        self.easiness_factor = sm2_result.easiness_factor;
        self.interval_days = sm2_result.interval_days;
        self.next_review_date = Some(sm2_result.next_review_date);
        self.last_reviewed = Some(Utc::now());
        self.total_reviews += 1;

        if quality as u8 >= 3 {
            self.successful_reviews += 1;
        }

        // イベント生成
        self.raise_event(DomainEvent::ItemRecalled {
            progress_id: self.progress_id,
            user_id: self.user_id,
            item_id: self.item_id,
            recall_quality: quality,
            response_time_ms,
            occurred_at: Utc::now(),
        });

        self.raise_event(DomainEvent::ReviewScheduled {
            progress_id: self.progress_id,
            user_id: self.user_id,
            item_id: self.item_id,
            scheduled_for: sm2_result.next_review_date,
            interval_days: sm2_result.interval_days,
            easiness_factor: sm2_result.easiness_factor,
            repetition_number: sm2_result.repetition_number,
            algorithm_version: "SM-2".to_string(),
            occurred_at: Utc::now(),
        });

        Ok(())
    }
}

// domain/services/sm2_calculator.rs
pub struct SM2Calculator {
    initial_interval: f32,
    easy_bonus: f32,
    min_easiness_factor: f32,
}

impl SM2Calculator {
    pub fn new() -> Self {
        Self {
            initial_interval: 1.0,
            easy_bonus: 1.3,
            min_easiness_factor: 1.3,
        }
    }

    pub fn calculate(
        &self,
        quality: RecallQuality,
        current_repetition: u32,
        current_interval: f32,
        current_ef: f32,
    ) -> SM2Result {
        let q = quality as u8;

        // Easiness Factor の計算
        let new_ef = if q >= 3 {
            current_ef + (0.1 - (5 - q) as f32 * (0.08 + (5 - q) as f32 * 0.02))
        } else {
            current_ef - 0.8
        }
        .max(self.min_easiness_factor);

        // 間隔と繰り返し回数の計算
        let (new_interval, new_repetition) = match q {
            0..=2 => {
                // 失敗：リセット
                (self.initial_interval, 0)
            }
            _ => {
                // 成功：次の間隔を計算
                match current_repetition {
                    0 => (1.0, 1),
                    1 => (6.0, 2),
                    _ => (current_interval * new_ef, current_repetition + 1),
                }
            }
        };

        // Perfect (5) の場合はボーナス
        let final_interval = if q == 5 {
            new_interval * self.easy_bonus
        } else {
            new_interval
        };

        SM2Result {
            interval_days: final_interval,
            easiness_factor: new_ef,
            repetition_number: new_repetition,
            next_review_date: Utc::now() + Duration::days(final_interval.round() as i64),
        }
    }
}
```

#### Application Layer

```rust
// application/command_handlers/record_recall_handler.rs
pub struct RecordRecallHandler {
    event_store: Arc<dyn EventStore>,
    event_publisher: Arc<dyn EventPublisher>,
    progress_repository: Arc<dyn ProgressRepository>,
}

impl RecordRecallHandler {
    pub async fn handle(
        &self,
        command: RecordRecallCommand,
    ) -> Result<RecordRecallResponse, ApplicationError> {
        // 1. Progress の取得または作成
        let mut progress = match self.progress_repository
            .find_by_user_and_item(&command.user_id, &command.item_id)
            .await?
        {
            Some(p) => p,
            None => Progress::new(command.user_id, command.item_id),
        };

        // 2. 想起結果の記録
        progress.record_recall(command.quality, command.response_time_ms)?;

        // 3. イベントの収集
        let events = progress.take_events();

        // 4. Event Store に保存
        self.event_store.append_events(&events).await?;

        // 5. Event Bus に発行
        for event in &events {
            self.event_publisher.publish(event.clone()).await?;
        }

        // 6. レスポンスの構築
        Ok(RecordRecallResponse {
            next_review_date: progress.next_review_date().map(|d| d.to_rfc3339()),
            interval_days: progress.interval_days(),
            easiness_factor: progress.easiness_factor(),
            repetition_number: progress.repetition_number(),
        })
    }
}

// application/command_handlers/complete_session_handler.rs
pub struct CompleteSessionHandler {
    event_store: Arc<dyn EventStore>,
    event_publisher: Arc<dyn EventPublisher>,
    session_repository: Arc<dyn SessionRepository>,
    streak_service: Arc<StreakService>,
}

impl CompleteSessionHandler {
    pub async fn handle(
        &self,
        command: CompleteSessionCommand,
    ) -> Result<CompleteSessionResponse, ApplicationError> {
        // 1. セッションの完了
        let mut session = self.session_repository
            .find_by_id(&command.session_id)
            .await?
            .ok_or(ApplicationError::SessionNotFound)?;

        session.complete(command.completion_reason)?;

        // 2. ストリークの更新チェック
        if let Some(streak_event) = self.streak_service
            .check_and_update_streak(&session.user_id())
            .await?
        {
            session.add_event(streak_event);
        }

        // 3. マイルストーンのチェック
        let milestones = self.check_milestones(&session).await?;
        for milestone in milestones {
            session.add_event(DomainEvent::MilestoneAchieved {
                user_id: session.user_id(),
                milestone_type: milestone.milestone_type,
                milestone_value: milestone.value,
                occurred_at: Utc::now(),
            });
        }

        // 4. イベントの永続化と発行
        let events = session.take_events();
        self.event_store.append_events(&events).await?;

        for event in &events {
            self.event_publisher.publish(event.clone()).await?;
        }

        Ok(CompleteSessionResponse {
            session_stats: self.calculate_session_stats(&session),
            streak_updated: events.iter().any(|e| matches!(e, DomainEvent::StreakUpdated { .. })),
            milestones_achieved: milestones.len() as u32,
        })
    }
}
```

#### Infrastructure Layer

```rust
// infrastructure/repositories/postgres_progress_repository.rs
pub struct PostgresProgressRepository {
    pool: PgPool,
}

#[async_trait]
impl ProgressRepository for PostgresProgressRepository {
    async fn find_by_user_and_item(
        &self,
        user_id: &UserId,
        item_id: &ItemId,
    ) -> Result<Option<Progress>, RepositoryError> {
        // Event Store からイベントを読み込んで Progress を再構築
        let events = sqlx::query!(
            r#"
            SELECT event_data, event_type, occurred_at
            FROM events
            WHERE aggregate_type = 'Progress'
            AND aggregate_id = $1
            ORDER BY event_version
            "#,
            format!("{}:{}", user_id, item_id)
        )
        .fetch_all(&self.pool)
        .await?;

        if events.is_empty() {
            return Ok(None);
        }

        // イベントから Progress を再構築
        let progress = Progress::from_events(events)?;
        Ok(Some(progress))
    }
}
```

## 設定とデプロイメント

### 環境変数

```yaml
# データベース
DATABASE_URL: postgres://user:pass@postgres:5432/progress_event_store
DATABASE_MAX_CONNECTIONS: 10

# Pub/Sub
PUBSUB_PROJECT_ID: effect-project
PUBSUB_TOPIC: progress-events
PUBSUB_EMULATOR_HOST: localhost:8085 # 開発環境

# サービス設定
SERVICE_PORT: 50061
LOG_LEVEL: info
TRACE_ENDPOINT: https://cloudtrace.googleapis.com
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
COPY --from=builder /app/target/release/progress-command-service /usr/local/bin/
EXPOSE 50061
CMD ["progress-command-service"]
```

### Cloud Run デプロイメント

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: progress-command-service
spec:
  template:
    metadata:
      annotations:
        run.googleapis.com/cloudsql-instances: project:region:instance
    spec:
      serviceAccountName: progress-service
      containers:
        - image: gcr.io/effect-project/progress-command-service:latest
          ports:
            - containerPort: 50061
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: progress-secrets
                  key: database-url
          resources:
            limits:
              memory: "512Mi"
              cpu: "1000m"
```

## エラーハンドリング

### ドメインエラー

```rust
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Session already ended")]
    SessionAlreadyEnded,

    #[error("Invalid recall quality: {0}")]
    InvalidRecallQuality(u8),

    #[error("Progress not initialized")]
    ProgressNotInitialized,

    #[error("Invalid state transition")]
    InvalidStateTransition,
}
```

### 補償トランザクション

```rust
// セッション異常終了時の補償
pub async fn compensate_incomplete_session(
    session_id: SessionId,
) -> Result<(), CompensationError> {
    // 1. 未完了セッションのイベントを取得
    let incomplete_events = event_store.get_incomplete_session_events(session_id).await?;

    // 2. 補償イベントを生成
    let compensation_event = DomainEvent::SessionAbnormallyTerminated {
        session_id,
        reason: "System failure",
        occurred_at: Utc::now(),
    };

    // 3. 補償イベントを発行
    event_store.append_event(compensation_event).await?;

    Ok(())
}
```

## 監視とメトリクス

### 主要メトリクス

```rust
lazy_static! {
    static ref SM2_CALCULATION_TIME: Histogram = register_histogram!(
        "progress_sm2_calculation_seconds",
        "Time to calculate SM2 algorithm"
    ).unwrap();

    static ref RECALL_QUALITY_DISTRIBUTION: IntCounterVec = register_int_counter_vec!(
        "progress_recall_quality_total",
        "Distribution of recall quality scores",
        &["quality"]
    ).unwrap();

    static ref SESSION_DURATION: Histogram = register_histogram!(
        "progress_session_duration_seconds",
        "Learning session duration"
    ).unwrap();
}
```

### ヘルスチェック

```rust
async fn health_check() -> HealthStatus {
    let db_healthy = check_database_connection().await;
    let pubsub_healthy = check_pubsub_connection().await;

    if db_healthy && pubsub_healthy {
        HealthStatus::Healthy
    } else {
        HealthStatus::Unhealthy
    }
}
```

## 更新履歴

- 2025-08-03: 初版作成
