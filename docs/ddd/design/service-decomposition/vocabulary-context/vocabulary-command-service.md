# vocabulary-command-service 設計書

## 概要

vocabulary-command-service は、Vocabulary Context の Write Model を管理するサービスです。すべての書き込み操作を処理し、ドメインイベントを生成して Event Store に永続化します。

## 責務

1. **コマンドの受付と検証**
   - CreateItem, UpdateItem, DeleteItem など
   - ビジネスルールの適用
   - 権限チェック

2. **集約の管理**
   - VocabularyEntry 集約
   - VocabularyItem 集約
   - 不変条件の保証

3. **イベントの生成と永続化**
   - ドメインイベントの生成
   - Event Store への保存
   - Event Bus への発行

4. **楽観的ロックと競合解決**
   - バージョン管理
   - 自動マージ可能な変更の検出
   - 競合通知

## アーキテクチャ

### レイヤー構造

```
vocabulary-command-service/
├── api/              # gRPC API 定義
├── application/      # アプリケーションサービス
├── domain/           # ドメインモデル
├── infrastructure/   # インフラストラクチャ
└── main.rs          # エントリーポイント
```

### 詳細設計

#### API Layer

```rust
// api/grpc/vocabulary_command.proto
service VocabularyCommandService {
    rpc CreateItem(CreateItemCommand) returns (CreateItemResponse);
    rpc UpdateItem(UpdateItemCommand) returns (UpdateItemResponse);
    rpc DeleteItem(DeleteItemCommand) returns (DeleteItemResponse);
    rpc RequestAIGeneration(RequestAIGenerationCommand) returns (RequestAIGenerationResponse);
}

message CreateItemCommand {
    string spelling = 1;
    string disambiguation = 2;
    string part_of_speech = 3;
    CreationMethod creation_method = 4;
    string created_by = 5;
}

message UpdateItemCommand {
    string item_id = 1;
    uint32 base_version = 2;
    repeated FieldChange changes = 3;
    string updated_by = 4;
}

message FieldChange {
    string field_path = 1;
    google.protobuf.Value old_value = 2;
    google.protobuf.Value new_value = 3;
}
```

#### Domain Layer

```rust
// domain/aggregates/vocabulary_entry.rs
pub struct VocabularyEntry {
    entry_id: EntryId,
    spelling: String,
    items: Vec<ItemSummary>,
    created_at: DateTime<Utc>,
    events: Vec<DomainEvent>,  // 未コミットのイベント
}

impl VocabularyEntry {
    pub fn create(spelling: String) -> Result<Self, DomainError> {
        let entry_id = EntryId::new();
        let mut entry = Self {
            entry_id,
            spelling: spelling.clone(),
            items: vec![],
            created_at: Utc::now(),
            events: vec![],
        };
        
        entry.raise_event(DomainEvent::EntryCreated {
            entry_id,
            spelling,
            occurred_at: Utc::now(),
        });
        
        Ok(entry)
    }
    
    pub fn add_item(&mut self, item_summary: ItemSummary) -> Result<(), DomainError> {
        // 重複チェック
        if self.items.iter().any(|i| i.disambiguation == item_summary.disambiguation) {
            return Err(DomainError::DuplicateItem);
        }
        
        self.items.push(item_summary);
        Ok(())
    }
}

// domain/aggregates/vocabulary_item.rs
pub struct VocabularyItem {
    item_id: ItemId,
    entry_id: EntryId,
    spelling: String,
    disambiguation: String,
    // ... 他のフィールド
    version: u32,
    status: ItemStatus,
    events: Vec<DomainEvent>,
}

impl VocabularyItem {
    pub fn update_fields(
        &mut self,
        base_version: u32,
        changes: Vec<FieldChange>,
        updated_by: UserId,
    ) -> Result<(), DomainError> {
        // 楽観的ロックチェック
        if base_version != self.version {
            return Err(DomainError::VersionConflict {
                expected: base_version,
                actual: self.version,
            });
        }
        
        // 変更を適用
        for change in changes {
            self.apply_field_change(&change)?;
            
            self.raise_event(DomainEvent::FieldUpdated {
                item_id: self.item_id,
                field_path: change.field_path,
                old_value: change.old_value,
                new_value: change.new_value,
                updated_by,
                version: self.version + 1,
                occurred_at: Utc::now(),
            });
        }
        
        self.version += 1;
        Ok(())
    }
}
```

#### Application Layer

```rust
// application/command_handlers/create_item_handler.rs
pub struct CreateItemHandler {
    event_store: Arc<dyn EventStore>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreateItemHandler {
    pub async fn handle(&self, command: CreateItemCommand) -> Result<ItemId, ApplicationError> {
        // 1. Entry の存在確認または作成
        let entry = match self.event_store.load_aggregate::<VocabularyEntry>(&command.spelling).await? {
            Some(entry) => entry,
            None => VocabularyEntry::create(command.spelling.clone())?,
        };
        
        // 2. Item の作成
        let item = VocabularyItem::create(
            entry.entry_id,
            command.spelling,
            command.disambiguation,
            command.created_by,
        )?;
        
        // 3. Entry に Item を追加
        entry.add_item(ItemSummary {
            item_id: item.item_id,
            disambiguation: command.disambiguation,
            is_primary: entry.items.is_empty(),
        })?;
        
        // 4. イベントを収集
        let mut events = vec![];
        events.extend(entry.take_events());
        events.extend(item.take_events());
        
        // 5. Event Store に保存
        self.event_store.save_events(&events).await?;
        
        // 6. Event Bus に発行
        for event in events {
            self.event_publisher.publish(event).await?;
        }
        
        Ok(item.item_id)
    }
}
```

#### Infrastructure Layer

```rust
// infrastructure/event_store/postgres_event_store.rs
pub struct PostgresEventStore {
    pool: PgPool,
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn save_events(&self, events: &[DomainEvent]) -> Result<(), EventStoreError> {
        let mut tx = self.pool.begin().await?;
        
        for event in events {
            let event_data = serde_json::to_value(event)?;
            
            sqlx::query!(
                r#"
                INSERT INTO vocabulary_events 
                (event_id, aggregate_id, aggregate_type, event_type, event_data, occurred_at, sequence_number)
                VALUES ($1, $2, $3, $4, $5, $6, 
                    (SELECT COALESCE(MAX(sequence_number), 0) + 1 
                     FROM vocabulary_events 
                     WHERE aggregate_id = $2))
                "#,
                event.event_id(),
                event.aggregate_id(),
                event.aggregate_type(),
                event.event_type(),
                event_data,
                event.occurred_at(),
            )
            .execute(&mut tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }
    
    async fn load_events(&self, aggregate_id: &str) -> Result<Vec<DomainEvent>, EventStoreError> {
        let records = sqlx::query!(
            r#"
            SELECT event_data, event_type, occurred_at
            FROM vocabulary_events
            WHERE aggregate_id = $1
            ORDER BY sequence_number
            "#,
            aggregate_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let events = records
            .into_iter()
            .map(|r| Self::deserialize_event(r.event_type, r.event_data))
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(events)
    }
}

// infrastructure/event_publisher/pubsub_publisher.rs
pub struct PubSubEventPublisher {
    publisher: Publisher,
    topic_name: String,
}

#[async_trait]
impl EventPublisher for PubSubEventPublisher {
    async fn publish(&self, event: DomainEvent) -> Result<(), PublisherError> {
        let message_data = serde_json::to_vec(&event)?;
        
        let pubsub_message = PubsubMessage {
            data: message_data,
            attributes: HashMap::from([
                ("event_type".to_string(), event.event_type().to_string()),
                ("aggregate_id".to_string(), event.aggregate_id().to_string()),
                ("occurred_at".to_string(), event.occurred_at().to_rfc3339()),
            ]),
            ..Default::default()
        };
        
        let topic = self.publisher.topic(&self.topic_name);
        topic.publish(pubsub_message).await?;
        
        Ok(())
    }
}
```

## 設定とデプロイメント

### 環境変数

```yaml
# 環境設定
DATABASE_URL: postgres://user:pass@postgres:5432/event_store
PUBSUB_PROJECT_ID: effect-project
PUBSUB_TOPIC: vocabulary-events
SERVICE_PORT: 50051
LOG_LEVEL: info
TRACE_ENDPOINT: http://jaeger:14268/api/traces
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
COPY --from=builder /app/target/release/vocabulary-command-service /usr/local/bin/
CMD ["vocabulary-command-service"]
```

### Kubernetes マニフェスト

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vocabulary-command-service
spec:
  replicas: 2
  selector:
    matchLabels:
      app: vocabulary-command-service
  template:
    metadata:
      labels:
        app: vocabulary-command-service
    spec:
      containers:
      - name: service
        image: vocabulary-command-service:latest
        ports:
        - containerPort: 50051
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: vocabulary-secrets
              key: event-store-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

## エラーハンドリング

### ドメインエラー

```rust
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Item with disambiguation '{0}' already exists")]
    DuplicateItem(String),
    
    #[error("Version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u32, actual: u32 },
    
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },
}
```

### リトライポリシー

```rust
// Exponential backoff for transient failures
async fn with_retry<F, T>(operation: F) -> Result<T, Error>
where
    F: Fn() -> Future<Output = Result<T, Error>>,
{
    let mut attempts = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_transient() && attempts < 3 => {
                let delay = Duration::from_millis(100 * 2_u64.pow(attempts));
                tokio::time::sleep(delay).await;
                attempts += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## 監視とロギング

### 構造化ログ

```rust
use tracing::{info, error, instrument};

#[instrument(skip(self), fields(command_type = "CreateItem"))]
pub async fn handle(&self, command: CreateItemCommand) -> Result<ItemId> {
    info!(
        spelling = %command.spelling,
        disambiguation = %command.disambiguation,
        "Processing CreateItem command"
    );
    
    // 処理...
    
    info!(
        item_id = %item_id,
        "CreateItem command completed successfully"
    );
}
```

### メトリクス

```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref COMMAND_COUNTER: Counter = Counter::new(
        "vocabulary_command_total", 
        "Total number of commands processed"
    ).unwrap();
    
    static ref COMMAND_DURATION: Histogram = Histogram::new(
        "vocabulary_command_duration_seconds",
        "Command processing duration"
    ).unwrap();
}
```

## 更新履歴

- 2025-08-03: 初版作成
