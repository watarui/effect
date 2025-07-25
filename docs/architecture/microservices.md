# マイクロサービスアーキテクチャ

## 概要

Effect は、独立したマイクロサービスの集合体として構築されています。各サービスは単一責任原則に従い、特定のビジネス機能を担当します。

## サービス構成

```
┌─────────────────────────────────────────────────────────────┐
│                         Clients                             │
│              (Web App, Mobile App, CLI)                     │
└──────────────────────────┬──────────────────────────────────┘
                           │
                    ┌──────▼──────┐
                    │ API Gateway │
                    │  (GraphQL)  │
                    └──────┬──────┘
                           │
    ┌──────────────────────┼──────────────────────┐
    │                      │                      │
┌───▼────┐         ┌──────▼──────┐        ┌─────▼─────┐
│  User  │         │   Command   │        │   Query   │
│Service │         │   Service   │        │  Service  │
└───┬────┘         └──────┬──────┘        └─────┬─────┘
    │                     │                      │
    │              ┌──────▼──────┐               │
    │              │ Event Store │               │
    │              │(PostgreSQL) │               │
    │              └──────┬──────┘               │
    │                     │                      │
    │              ┌──────▼──────┐        ┌─────▼─────┐
    │              │   Pub/Sub   │        │Read Model │
    │              │   (Redis)   │        │   (DB)    │
    │              └──────┬──────┘        └───────────┘
    │                     │
    └─────────────────────┼─────────────────────┘
                          │
                   ┌──────▼──────┐
                   │    Saga     │
                   │  Executor   │
                   └─────────────┘
```

## 各サービスの詳細

### 1. API Gateway

**責務:**

- クライアントへの単一エンドポイント提供
- GraphQL スキーマの管理
- 認証トークンの検証
- リクエストのルーティング
- レート制限

**技術スタック:**

- Rust + async-graphql
- JWT 認証
- DataLoader による N+1 問題の解決

```rust
// GraphQL スキーマ定義
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn word(&self, ctx: &Context<'_>, id: Uuid) -> Result<Word> {
        let query_client = ctx.data::<QueryServiceClient>()?;
        query_client.get_word(id).await
    }

    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let user_client = ctx.data::<UserServiceClient>()?;
        let user_id = ctx.data::<UserId>()?;
        user_client.get_user(*user_id).await
    }
}
```

### 2. User Service

**責務:**

- ユーザー認証・認可
- プロフィール管理
- 設定管理
- セッション管理
- 貢献度トラッキング

**主要なエンティティ:**

- User
- UserProfile
- UserSettings
- AuthToken
- ContributionScore

**gRPC インターフェース:**

```protobuf
service UserService {
    rpc GetUser(GetUserRequest) returns (UserResponse);
    rpc GetUsers(GetUsersRequest) returns (UsersResponse);
    rpc ValidateToken(ValidateTokenRequest) returns (ValidationResponse);
    rpc Register(RegisterRequest) returns (AuthResponse);
    rpc Login(LoginRequest) returns (AuthResponse);
}
```

### 3. Command Service

**責務:**

- 書き込み操作の処理
- ドメインロジックの実行
- イベントの生成
- トランザクション管理

**主要な集約:**

- Word Aggregate
- LearningSession Aggregate
- UserProgress Aggregate

**コマンドハンドラー例:**

```rust
pub async fn handle_create_word(
    &self,
    cmd: CreateWordCommand,
) -> Result<Uuid> {
    // ドメインロジック実行
    let word = WordAggregate::create(cmd)?;

    // イベントストアに保存
    self.event_store.save_events(
        &word.id,
        word.pending_events,
    ).await?;

    Ok(word.id)
}
```

### 4. Query Service

**責務:**

- 読み取り専用 API の提供
- Read Model の管理
- クエリの最適化
- キャッシング

**プロジェクション例:**

```rust
#[async_trait]
impl EventHandler for WordProjection {
    async fn handle(&self, event: DomainEvent) -> Result<()> {
        match event {
            DomainEvent::WordCreated(e) => {
                let word_view = WordView {
                    id: e.word_id,
                    text: e.text,
                    meanings: vec![],
                    categories: e.categories,
                    difficulty: e.difficulty,
                    created_by: e.created_by,
                    created_at: e.created_at,
                    version: 1,
                };
                self.repository.insert(word_view).await?;
            }
            DomainEvent::WordUpdated(e) => {
                self.repository.update(e.word_id, e.changes).await?;
            }
            DomainEvent::WordMeaningAdded(e) => {
                self.repository.add_meaning(e.word_id, e.meaning).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

### 5. Saga Executor

**責務:**

- 分散トランザクションの管理
- 補償トランザクション
- ワークフローの実行
- エラーハンドリング

**Saga 例:**

```rust
pub struct UserRegistrationSaga;

#[async_trait]
impl Saga for UserRegistrationSaga {
    async fn handle(&self, event: DomainEvent) -> Result<Vec<SagaCommand>> {
        match event {
            DomainEvent::UserRegistered(e) => {
                Ok(vec![
                    SagaCommand::SendWelcomeEmail {
                        user_id: e.user_id,
                        email: e.email,
                    },
                    SagaCommand::InitializeUserProgress {
                        user_id: e.user_id,
                    },
                    SagaCommand::CreateDefaultSettings {
                        user_id: e.user_id,
                    },
                ])
            }
            _ => Ok(vec![]),
        }
    }
}
```

## 通信パターン

### 同期通信（gRPC）

```protobuf
// 共通メッセージ定義
message CommandRequest {
    string command_type = 1;
    bytes payload = 2;
    string correlation_id = 3;
    string user_id = 4;
}

message CommandResponse {
    bool success = 1;
    string aggregate_id = 2;
    int64 version = 3;
    string error_message = 4;
}
```

### 非同期通信（Pub/Sub）

**トピック設計:**

```yaml
topics:
  # 単語関連イベント
  - name: word-events
    subscriptions:
      - query-service-word-projection
      - analytics-service

  # ユーザー関連イベント
  - name: user-events
    subscriptions:
      - query-service-user-projection
      - notification-service

  # 学習関連イベント
  - name: learning-events
    subscriptions:
      - query-service-progress-projection
      - saga-executor
      - analytics-service
```

## データ管理

### イベントストア（Write Model）

```sql
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB NOT NULL,
    version BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(aggregate_id, version)
);

CREATE INDEX idx_aggregate ON events(aggregate_id, version);
CREATE INDEX idx_event_type ON events(event_type, created_at);
```

### Read Model データベース

```sql
-- 単語ビュー
CREATE TABLE word_views (
    id UUID PRIMARY KEY,
    text VARCHAR(100) NOT NULL,
    phonetic_ipa VARCHAR(200),
    meanings JSONB NOT NULL DEFAULT '[]',
    categories TEXT[] NOT NULL DEFAULT '{}',
    difficulty INTEGER NOT NULL,
    cefr_level VARCHAR(2),
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_favorite BOOLEAN DEFAULT FALSE
);

-- ユーザー進捗ビュー
CREATE TABLE user_progress_views (
    user_id UUID NOT NULL,
    word_id UUID NOT NULL,
    mastery_level FLOAT NOT NULL DEFAULT 0.0,
    sm2_repetition_count INTEGER DEFAULT 0,
    sm2_easiness_factor FLOAT DEFAULT 2.5,
    sm2_interval_days INTEGER DEFAULT 1,
    last_reviewed_at TIMESTAMP WITH TIME ZONE,
    next_review_at TIMESTAMP WITH TIME ZONE,
    total_reviews INTEGER DEFAULT 0,
    correct_count INTEGER DEFAULT 0,
    PRIMARY KEY (user_id, word_id)
);
```

## サービス間の依存関係

```mermaid
graph TD
    A[API Gateway] --> B[User Service]
    A --> C[Command Service]
    A --> D[Query Service]
    C --> E[Event Store]
    E --> F[Pub/Sub]
    F --> D
    F --> G[Saga Executor]
    G --> C
    B --> F
```

## スケーリング戦略

### 水平スケーリング

- **Query Service**: 読み取り負荷に応じて複数インスタンス
- **Command Service**: アグリゲート ID によるパーティショニング
- **User Service**: セッションのスティッキー化
- **Saga Executor**: Saga タイプによる分散

### キャッシング戦略

```rust
// Redis を使用したキャッシング
pub struct CachedQueryService {
    cache: Arc<RedisClient>,
    ttl: Duration,
}

impl CachedQueryService {
    pub async fn get_word(&self, id: Uuid) -> Result<Word> {
        let cache_key = format!("word:{}", id);

        // キャッシュチェック
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(cached);
        }

        // データベースから取得
        let word = self.repository.get_word(id).await?;

        // キャッシュに保存
        self.cache.set(&cache_key, &word, self.ttl).await?;

        Ok(word)
    }
}
```

## 監視とロギング

### メトリクス収集

```rust
#[instrument(skip(self))]
pub async fn handle_command(&self, cmd: Command) -> Result<Response> {
    let start = Instant::now();

    let result = self.process_command(cmd).await;

    // メトリクス記録
    metrics::histogram!("command_duration_seconds", start.elapsed());
    metrics::counter!("commands_total", 1,
        "command_type" => cmd.type_name(),
        "status" => if result.is_ok() { "success" } else { "error" }
    );

    result
}
```

### 分散トレーシング

- OpenTelemetry による統一的なトレーシング
- Jaeger でのトレース可視化
- サービス間のリクエストフロー追跡

## デプロイメント

### 開発環境（Docker Compose）

```yaml
version: "3.8"
services:
  api-gateway:
    build: ./services/api-gateway
    ports:
      - "8080:8080"
    depends_on:
      - user-service
      - command-service
      - query-service
    environment:
      - USER_SERVICE_URL=http://user-service:50051
      - COMMAND_SERVICE_URL=http://command-service:50052
      - QUERY_SERVICE_URL=http://query-service:50053

  user-service:
    build: ./services/user-service
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/users # pragma: allowlist secret
      - REDIS_URL=redis://redis:6379
      - JWT_SECRET=${JWT_SECRET}

  command-service:
    build: ./services/command-service
    environment:
      - EVENT_STORE_URL=postgresql://postgres:password@postgres:5432/events # pragma: allowlist secret
      - PUBSUB_URL=redis://redis:6379

  query-service:
    build: ./services/query-service
    environment:
      - READ_MODEL_URL=postgresql://postgres:password@postgres:5432/readmodel # pragma: allowlist secret
      - PUBSUB_URL=redis://redis:6379

  saga-executor:
    build: ./services/saga-executor
    environment:
      - PUBSUB_URL=redis://redis:6379
      - COMMAND_SERVICE_URL=http://command-service:50052

  postgres:
    image: postgres:15
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      - POSTGRES_PASSWORD=password

  redis:
    image: redis:7
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

### 本番環境（Cloud Run）

#### デプロイ戦略

1. **コンテナイメージのビルドとプッシュ**

```bash
# 各サービスのイメージをビルド
docker build -t gcr.io/effect-project/api-gateway ./services/api-gateway
docker build -t gcr.io/effect-project/user-service ./services/user-service
# ... 他のサービスも同様

# Google Container Registry にプッシュ
docker push gcr.io/effect-project/api-gateway
docker push gcr.io/effect-project/user-service
```

2. **Cloud Run へのデプロイ**

```bash
# API Gateway
gcloud run deploy api-gateway \
  --image gcr.io/effect-project/api-gateway \
  --platform managed \
  --region asia-northeast1 \
  --allow-unauthenticated \
  --set-env-vars="USER_SERVICE_URL=https://user-service-xxx.a.run.app"

# User Service（内部通信のみ）
gcloud run deploy user-service \
  --image gcr.io/effect-project/user-service \
  --platform managed \
  --region asia-northeast1 \
  --no-allow-unauthenticated \
  --set-env-vars="DATABASE_URL=postgresql://..."
```

#### Cloud Run の利点

- **自動スケーリング**: 0 から数千インスタンスまで自動調整
- **従量課金**: リクエストがない時は課金されない
- **マネージド**: インフラ管理が不要
- **HTTPS 自動対応**: SSL 証明書の管理不要

#### 外部サービスの構成

```yaml
# Cloud SQL (PostgreSQL)
- Event Store 用インスタンス
- Read Model 用インスタンス
- 自動バックアップ設定

# Cloud Memorystore (Redis)
- Pub/Sub 用
- キャッシュ用
- VPC 内でのプライベート接続

# Cloud Storage
- 静的ファイル（音声、画像）の保存
```

#### CI/CD パイプライン

```yaml
# .github/workflows/deploy.yml
name: Deploy to Cloud Run
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - id: auth
        uses: google-github-actions/auth@v1
        with:
          credentials_json: ${{ secrets.GCP_SA_KEY }}

      - name: Deploy API Gateway
        run: |
          docker build -t gcr.io/$PROJECT_ID/api-gateway ./services/api-gateway
          docker push gcr.io/$PROJECT_ID/api-gateway
          gcloud run deploy api-gateway --image gcr.io/$PROJECT_ID/api-gateway
```
