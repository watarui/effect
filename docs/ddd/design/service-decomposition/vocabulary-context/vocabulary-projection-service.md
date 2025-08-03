# vocabulary-projection-service 設計書

## 概要

vocabulary-projection-service は、Event Store からイベントを消費し、
Read Model（Query Service 用）と Search Index（Search Service 用）を構築・更新する責務を持ちます。
Event Sourcing アーキテクチャの要となるサービスで、最終的整合性を保証します。

## 責務

1. **イベント消費**

   - Google Pub/Sub からのイベント受信
   - イベント順序の保証
   - 重複排除とべき等性の実現

2. **Read Model 更新**

   - PostgreSQL Read DB への投影
   - 集約ビューの構築
   - 楽観的ロックによる一貫性保証

3. **Search Index 更新**

   - Meilisearch インデックスの更新
   - 非同期バッチ処理
   - インデックス最適化

4. **プロジェクション管理**
   - プロジェクションの状態管理
   - リプレイ機能
   - エラーハンドリングとリトライ

## アーキテクチャ

### レイヤー構造

```
vocabulary-projection-service/
├── application/      # プロジェクションハンドラー
├── domain/           # イベント・投影モデル
├── infrastructure/   # Pub/Sub、DB、検索エンジン接続
└── main.rs          # エントリーポイント
```

### 詳細設計

#### Domain Layer

```rust
// domain/events/vocabulary_events.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VocabularyEvent {
    EntryCreated(EntryCreatedEvent),
    ItemAddedToEntry(ItemAddedToEntryEvent),
    ItemUpdated(ItemUpdatedEvent),
    ItemDeleted(ItemDeletedEvent),
    ExampleAdded(ExampleAddedEvent),
    ItemsGeneratedWithAI(ItemsGeneratedWithAIEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub event_data: VocabularyEvent,
    pub event_version: i64,
    pub occurred_at: DateTime<Utc>,
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: String,
    pub user_id: String,
    pub source_service: String,
    pub idempotency_key: Option<String>,
}

// domain/projections/projection_state.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionState {
    pub projection_name: String,
    pub last_processed_event_id: String,
    pub last_processed_timestamp: DateTime<Utc>,
    pub position: ProjectionPosition,
    pub status: ProjectionStatus,
    pub error_count: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionPosition {
    pub event_store_position: i64,
    pub pubsub_ack_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectionStatus {
    Running,
    Paused,
    Failed,
    Rebuilding,
}
```

#### Application Layer

```rust
// application/projection_manager.rs
pub struct ProjectionManager {
    event_subscriber: Arc<dyn EventSubscriber>,
    projections: Vec<Box<dyn Projection>>,
    state_repository: Arc<dyn ProjectionStateRepository>,
    metrics: Arc<ProjectionMetrics>,
}

impl ProjectionManager {
    pub async fn start(&self) -> Result<(), ProjectionError> {
        info!("Starting projection manager");

        // 各プロジェクションの状態を復元
        for projection in &self.projections {
            let state = self.state_repository
                .get_state(projection.name())
                .await?;

            if let Some(state) = state {
                projection.restore_from_state(state).await?;
            }
        }

        // イベントサブスクリプション開始
        let event_stream = self.event_subscriber
            .subscribe("vocabulary-events")
            .await?;

        self.process_event_stream(event_stream).await
    }

    async fn process_event_stream(
        &self,
        mut stream: impl Stream<Item = Result<EventEnvelope, SubscriptionError>>,
    ) -> Result<(), ProjectionError> {
        while let Some(result) = stream.next().await {
            match result {
                Ok(envelope) => {
                    self.handle_event(envelope).await?;
                }
                Err(e) => {
                    error!("Error receiving event: {:?}", e);
                    self.handle_subscription_error(e).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_event(&self, envelope: EventEnvelope) -> Result<(), ProjectionError> {
        let start = Instant::now();

        // べき等性チェック
        if self.is_duplicate(&envelope).await? {
            debug!("Skipping duplicate event: {}", envelope.event_id);
            return Ok(());
        }

        // 各プロジェクションに配信
        let mut errors = vec![];

        for projection in &self.projections {
            match projection.handle(envelope.clone()).await {
                Ok(_) => {
                    self.update_projection_state(projection.name(), &envelope).await?;
                }
                Err(e) => {
                    error!("Projection {} failed: {:?}", projection.name(), e);
                    errors.push((projection.name(), e));
                }
            }
        }

        // メトリクス記録
        self.metrics.record_event_processed(
            envelope.event_type.as_str(),
            start.elapsed(),
            errors.is_empty(),
        );

        if !errors.is_empty() {
            return Err(ProjectionError::PartialFailure(errors));
        }

        Ok(())
    }
}

// application/projections/vocabulary_read_model_projection.rs
pub struct VocabularyReadModelProjection {
    entry_repository: Arc<dyn EntryWriteRepository>,
    item_repository: Arc<dyn ItemWriteRepository>,
    example_repository: Arc<dyn ExampleWriteRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
}

#[async_trait]
impl Projection for VocabularyReadModelProjection {
    fn name(&self) -> &str {
        "vocabulary_read_model"
    }

    async fn handle(&self, envelope: EventEnvelope) -> Result<(), ProjectionError> {
        match &envelope.event_data {
            VocabularyEvent::EntryCreated(event) => {
                self.handle_entry_created(event, &envelope.metadata).await
            }
            VocabularyEvent::ItemAddedToEntry(event) => {
                self.handle_item_added(event, &envelope.metadata).await
            }
            VocabularyEvent::ItemUpdated(event) => {
                self.handle_item_updated(event, &envelope.metadata).await
            }
            VocabularyEvent::ItemDeleted(event) => {
                self.handle_item_deleted(event, &envelope.metadata).await
            }
            VocabularyEvent::ExampleAdded(event) => {
                self.handle_example_added(event, &envelope.metadata).await
            }
            VocabularyEvent::ItemsGeneratedWithAI(event) => {
                self.handle_items_generated(event, &envelope.metadata).await
            }
        }
    }
}

impl VocabularyReadModelProjection {
    async fn handle_entry_created(
        &self,
        event: &EntryCreatedEvent,
        metadata: &EventMetadata,
    ) -> Result<(), ProjectionError> {
        let entry_view = EntryView {
            entry_id: event.entry_id.clone(),
            word: event.word.clone(),
            reading: event.reading.clone(),
            part_of_speech: event.part_of_speech.clone(),
            item_count: 0,
            created_by: metadata.user_id.clone(),
            created_at: event.created_at,
            updated_at: event.created_at,
            version: 1,
            total_examples: 0,
            difficulty_distribution: HashMap::new(),
            category_distribution: HashMap::new(),
        };

        self.entry_repository.insert(entry_view).await?;

        Ok(())
    }

    async fn handle_item_added(
        &self,
        event: &ItemAddedToEntryEvent,
        metadata: &EventMetadata,
    ) -> Result<(), ProjectionError> {
        // トランザクション内で実行
        self.transaction_manager.execute(|tx| async move {
            // 項目を追加
            let item_view = ItemView {
                item_id: event.item_id.clone(),
                entry_id: event.entry_id.clone(),
                word: event.word.clone(),
                definition: event.definition.clone(),
                difficulty: event.difficulty.clone(),
                categories: event.categories.clone(),
                tags: event.tags.clone(),
                quality_score: 0.0,
                example_count: 0,
                created_by: metadata.user_id.clone(),
                created_at: event.created_at,
                updated_at: event.created_at,
                version: 1,
                contributor_name: String::new(), // 別途取得
                is_ai_generated: event.is_ai_generated,
                review_status: ReviewStatus::Pending,
            };

            self.item_repository.insert_with_tx(tx, item_view).await?;

            // エントリーの統計を更新
            self.entry_repository.increment_item_count_with_tx(
                tx,
                &event.entry_id,
                1,
            ).await?;

            Ok(())
        }).await
    }

    async fn handle_items_generated(
        &self,
        event: &ItemsGeneratedWithAIEvent,
        metadata: &EventMetadata,
    ) -> Result<(), ProjectionError> {
        // バッチ処理
        let chunk_size = 100;

        for chunk in event.items.chunks(chunk_size) {
            self.transaction_manager.execute(|tx| async move {
                for item in chunk {
                    let item_view = ItemView {
                        item_id: item.item_id.clone(),
                        entry_id: event.entry_id.clone(),
                        word: event.word.clone(),
                        definition: item.definition.clone(),
                        difficulty: item.difficulty.clone(),
                        categories: item.categories.clone(),
                        tags: item.tags.clone(),
                        quality_score: 0.0,
                        example_count: item.examples.len() as u32,
                        created_by: metadata.user_id.clone(),
                        created_at: event.created_at,
                        updated_at: event.created_at,
                        version: 1,
                        contributor_name: "AI".to_string(),
                        is_ai_generated: true,
                        review_status: ReviewStatus::Pending,
                    };

                    self.item_repository.insert_with_tx(tx, item_view).await?;

                    // 例文も追加
                    for (idx, example) in item.examples.iter().enumerate() {
                        let example_view = ExampleView {
                            example_id: format!("{}-{}", item.item_id, idx),
                            item_id: item.item_id.clone(),
                            sentence: example.sentence.clone(),
                            translation: example.translation.clone(),
                            source: "AI Generated".to_string(),
                            tags: vec![],
                            created_by: metadata.user_id.clone(),
                            created_at: event.created_at,
                            usage_count: 0,
                            helpfulness_score: 0.0,
                        };

                        self.example_repository.insert_with_tx(tx, example_view).await?;
                    }
                }

                // エントリーの統計を更新
                self.entry_repository.increment_item_count_with_tx(
                    tx,
                    &event.entry_id,
                    chunk.len() as i32,
                ).await?;

                Ok(())
            }).await?;
        }

        Ok(())
    }
}

// application/projections/vocabulary_search_index_projection.rs
pub struct VocabularySearchIndexProjection {
    search_engine: Arc<dyn SearchEngine>,
    batch_processor: Arc<BatchProcessor>,
}

#[async_trait]
impl Projection for VocabularySearchIndexProjection {
    fn name(&self) -> &str {
        "vocabulary_search_index"
    }

    async fn handle(&self, envelope: EventEnvelope) -> Result<(), ProjectionError> {
        match &envelope.event_data {
            VocabularyEvent::ItemAddedToEntry(event) => {
                let doc = self.create_search_document(event).await?;
                self.batch_processor.add(doc).await?;
            }
            VocabularyEvent::ItemUpdated(event) => {
                let doc = self.update_search_document(event).await?;
                self.batch_processor.add(doc).await?;
            }
            VocabularyEvent::ItemDeleted(event) => {
                self.search_engine
                    .delete_document("vocabulary_items", &event.item_id)
                    .await?;
            }
            VocabularyEvent::ItemsGeneratedWithAI(event) => {
                let docs = self.create_search_documents_batch(event).await?;
                self.batch_processor.add_batch(docs).await?;
            }
            _ => {} // 他のイベントは無視
        }

        Ok(())
    }
}

// application/batch_processor.rs
pub struct BatchProcessor {
    search_engine: Arc<dyn SearchEngine>,
    buffer: Arc<Mutex<Vec<VocabularySearchDocument>>>,
    config: BatchConfig,
}

impl BatchProcessor {
    pub async fn start_processing(&self) {
        let mut interval = tokio::time::interval(self.config.flush_interval);

        loop {
            interval.tick().await;
            self.flush().await.unwrap_or_else(|e| {
                error!("Failed to flush batch: {:?}", e);
            });
        }
    }

    pub async fn add(&self, doc: VocabularySearchDocument) -> Result<(), BatchError> {
        let mut buffer = self.buffer.lock().await;
        buffer.push(doc);

        if buffer.len() >= self.config.max_batch_size {
            drop(buffer);
            self.flush().await?;
        }

        Ok(())
    }

    async fn flush(&self) -> Result<(), BatchError> {
        let mut buffer = self.buffer.lock().await;

        if buffer.is_empty() {
            return Ok(());
        }

        let docs = std::mem::take(&mut *buffer);
        drop(buffer);

        let start = Instant::now();

        match self.search_engine
            .index_documents_batch("vocabulary_items", docs)
            .await
        {
            Ok(_) => {
                info!(
                    "Indexed {} documents in {:?}",
                    docs.len(),
                    start.elapsed()
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to index batch: {:?}", e);

                // 失敗したドキュメントをバッファに戻す
                let mut buffer = self.buffer.lock().await;
                buffer.extend(docs);

                Err(BatchError::IndexingFailed(e))
            }
        }
    }
}
```

#### Infrastructure Layer

```rust
// infrastructure/pubsub/google_pubsub_subscriber.rs
use google_cloud_pubsub::{
    client::{Client, ClientConfig},
    subscription::{Subscription, SubscriptionConfig},
};

pub struct GooglePubSubSubscriber {
    client: Client,
    subscription_name: String,
    ack_deadline: Duration,
}

#[async_trait]
impl EventSubscriber for GooglePubSubSubscriber {
    async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<impl Stream<Item = Result<EventEnvelope, SubscriptionError>>, SubscriptionError> {
        let subscription = self.client
            .subscription(&self.subscription_name)
            .await?;

        let stream = subscription
            .receive(SubscriptionConfig {
                max_messages: 100,
                ack_deadline_seconds: self.ack_deadline.as_secs() as i32,
                ..Default::default()
            })
            .await?
            .map(|message| {
                // メッセージをデシリアライズ
                let envelope: EventEnvelope = serde_json::from_slice(&message.data)?;

                // ACK を送信
                message.ack().await?;

                Ok(envelope)
            });

        Ok(stream)
    }
}

// infrastructure/repositories/projection_state_repository.rs
pub struct PostgresProjectionStateRepository {
    pool: PgPool,
}

#[async_trait]
impl ProjectionStateRepository for PostgresProjectionStateRepository {
    async fn get_state(
        &self,
        projection_name: &str,
    ) -> Result<Option<ProjectionState>, RepositoryError> {
        let row = sqlx::query_as!(
            ProjectionStateRow,
            r#"
            SELECT
                projection_name,
                last_processed_event_id,
                last_processed_timestamp,
                event_store_position,
                pubsub_ack_id,
                status as "status: _",
                error_count,
                last_error
            FROM projection_states
            WHERE projection_name = $1
            "#,
            projection_name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(ProjectionState::from))
    }

    async fn update_state(
        &self,
        state: &ProjectionState,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO projection_states (
                projection_name,
                last_processed_event_id,
                last_processed_timestamp,
                event_store_position,
                pubsub_ack_id,
                status,
                error_count,
                last_error,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            ON CONFLICT (projection_name) DO UPDATE SET
                last_processed_event_id = EXCLUDED.last_processed_event_id,
                last_processed_timestamp = EXCLUDED.last_processed_timestamp,
                event_store_position = EXCLUDED.event_store_position,
                pubsub_ack_id = EXCLUDED.pubsub_ack_id,
                status = EXCLUDED.status,
                error_count = EXCLUDED.error_count,
                last_error = EXCLUDED.last_error,
                updated_at = NOW()
            "#,
            state.projection_name,
            state.last_processed_event_id,
            state.last_processed_timestamp,
            state.position.event_store_position,
            state.position.pubsub_ack_id,
            state.status as _,
            state.error_count as i32,
            state.last_error
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

## エラーハンドリングとリトライ

### エラー分類と対処

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("Transient error: {0}")]
    Transient(String),

    #[error("Fatal error: {0}")]
    Fatal(String),

    #[error("Partial failure: {0:?}")]
    PartialFailure(Vec<(&'static str, Box<dyn Error>)>),
}

pub struct RetryPolicy {
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
    exponential_base: f64,
}

impl RetryPolicy {
    pub async fn execute<F, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: Error + IsRetryable,
    {
        let mut attempt = 0;
        let mut delay = self.initial_delay;

        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if !e.is_retryable() => return Err(e),
                Err(e) if attempt >= self.max_attempts => {
                    error!("Max retry attempts exceeded: {:?}", e);
                    return Err(e);
                }
                Err(e) => {
                    warn!("Attempt {} failed: {:?}, retrying in {:?}", attempt, e, delay);
                    tokio::time::sleep(delay).await;

                    attempt += 1;
                    delay = std::cmp::min(
                        self.max_delay,
                        Duration::from_secs_f64(
                            delay.as_secs_f64() * self.exponential_base
                        ),
                    );
                }
            }
        }
    }
}
```

## リプレイ機能

### プロジェクションリビルド

```rust
pub struct ProjectionRebuilder {
    event_store: Arc<dyn EventStore>,
    projection_manager: Arc<ProjectionManager>,
}

impl ProjectionRebuilder {
    pub async fn rebuild_projection(
        &self,
        projection_name: &str,
        from_position: Option<i64>,
    ) -> Result<(), RebuildError> {
        info!("Starting rebuild of projection: {}", projection_name);

        // 1. プロジェクションを一時停止
        self.projection_manager.pause_projection(projection_name).await?;

        // 2. 既存データをクリア（オプション）
        if from_position.is_none() {
            self.clear_projection_data(projection_name).await?;
        }

        // 3. イベントストアから再生
        let start_position = from_position.unwrap_or(0);
        let event_stream = self.event_store
            .read_all_events_forward(start_position)
            .await?;

        // 4. イベントを順次処理
        let mut processed = 0;
        let start_time = Instant::now();

        pin_mut!(event_stream);

        while let Some(event) = event_stream.next().await {
            let envelope = event?;

            self.projection_manager
                .handle_event_for_projection(projection_name, envelope)
                .await?;

            processed += 1;

            if processed % 1000 == 0 {
                info!(
                    "Processed {} events in {:?}",
                    processed,
                    start_time.elapsed()
                );
            }
        }

        // 5. プロジェクションを再開
        self.projection_manager.resume_projection(projection_name).await?;

        info!(
            "Rebuild completed. Processed {} events in {:?}",
            processed,
            start_time.elapsed()
        );

        Ok(())
    }
}
```

## 設定とデプロイメント

### 環境変数

```yaml
# Google Pub/Sub
PUBSUB_PROJECT_ID: effect-project
PUBSUB_SUBSCRIPTION: vocabulary-events-projection-sub
PUBSUB_ACK_DEADLINE: 600s

# データベース
DATABASE_URL: postgres://user:pass@postgres:5432/vocabulary_read
EVENT_STORE_URL: postgres://user:pass@postgres:5432/vocabulary_events

# Meilisearch
MEILISEARCH_URL: http://meilisearch:7700
MEILISEARCH_API_KEY: ${MEILISEARCH_API_KEY}

# バッチ処理
BATCH_SIZE: 100
BATCH_FLUSH_INTERVAL: 5s

# リトライ
MAX_RETRY_ATTEMPTS: 3
INITIAL_RETRY_DELAY: 1s
MAX_RETRY_DELAY: 30s
```

### Cloud Run デプロイメント

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: vocabulary-projection-service
spec:
  template:
    metadata:
      annotations:
        run.googleapis.com/cloudsql-instances: project:region:instance
        run.googleapis.com/vpc-connector: projects/PROJECT/locations/REGION/connectors/CONNECTOR
        # CPU を常に割り当て（バックグラウンド処理のため）
        run.googleapis.com/cpu-throttling: "false"
    spec:
      serviceAccountName: vocabulary-projection
      containers:
        - image: gcr.io/effect-project/vocabulary-projection-service:latest
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: vocabulary-secrets
                  key: read-database-url
            - name: EVENT_STORE_URL
              valueFrom:
                secretKeyRef:
                  name: vocabulary-secrets
                  key: event-store-url
          resources:
            limits:
              memory: "2Gi"
              cpu: "2000m"
          livenessProbe:
            httpGet:
              path: /health
              port: 9090
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: /ready
              port: 9090
            periodSeconds: 10
```

## 監視とメトリクス

### 主要メトリクス

```rust
lazy_static! {
    static ref EVENT_PROCESSING_DURATION: HistogramVec = register_histogram_vec!(
        "projection_event_processing_duration_seconds",
        "Time taken to process an event",
        &["projection", "event_type"]
    ).unwrap();

    static ref EVENT_LAG: GaugeVec = register_gauge_vec!(
        "projection_event_lag_seconds",
        "Lag between event occurrence and processing",
        &["projection"]
    ).unwrap();

    static ref PROJECTION_ERRORS: IntCounterVec = register_int_counter_vec!(
        "projection_errors_total",
        "Total number of projection errors",
        &["projection", "error_type"]
    ).unwrap();

    static ref BATCH_SIZE: Histogram = register_histogram!(
        "search_index_batch_size",
        "Size of batches sent to search index"
    ).unwrap();
}
```

### ヘルスチェック

```rust
#[derive(Serialize)]
struct HealthStatus {
    status: String,
    projections: HashMap<String, ProjectionHealth>,
    lag_seconds: f64,
    error_rate: f64,
}

#[derive(Serialize)]
struct ProjectionHealth {
    status: String,
    last_processed_event: String,
    last_processed_at: DateTime<Utc>,
    error_count: u32,
    position: i64,
}

pub async fn health_check(
    State(app_state): State<Arc<AppState>>,
) -> Json<HealthStatus> {
    let mut projection_health = HashMap::new();

    for projection in &app_state.projections {
        let state = app_state.state_repository
            .get_state(projection.name())
            .await
            .unwrap_or(None);

        if let Some(state) = state {
            projection_health.insert(
                projection.name().to_string(),
                ProjectionHealth {
                    status: format!("{:?}", state.status),
                    last_processed_event: state.last_processed_event_id,
                    last_processed_at: state.last_processed_timestamp,
                    error_count: state.error_count,
                    position: state.position.event_store_position,
                },
            );
        }
    }

    Json(HealthStatus {
        status: "healthy".to_string(),
        projections: projection_health,
        lag_seconds: calculate_lag(&projection_health),
        error_rate: calculate_error_rate(&projection_health),
    })
}
```

## 更新履歴

- 2025-08-03: 初版作成
