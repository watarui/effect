# API Gateway Service

## 概要

API Gateway は、Effect プロジェクトの統一エントリーポイントとして機能する GraphQL サーバーです。
各マイクロサービスと gRPC で通信し、クライアントに対して統一された GraphQL API を提供します。

### 責務

1. **GraphQL API の提供**: 各 Bounded Context の機能を統合した GraphQL エンドポイント
2. **認証・認可**: JWT トークンの検証とユーザーコンテキストの管理
3. **ルーティング**: GraphQL クエリ/ミューテーションを適切なマイクロサービスへ転送
4. **データ集約**: 複数のマイクロサービスからのデータを統合
5. **キャッシング**: Redis を使用したクエリ結果のキャッシュ
6. **バッチ処理**: DataLoader による N+1 問題の解決

### 技術スタック

- **GraphQL Server**: async-graphql 7.0+
- **gRPC Client**: tonic 0.12+
- **Web Framework**: axum 0.7+
- **非同期ランタイム**: tokio 1.40+
- **キャッシュ**: Redis (redis-rs)
- **認証**: JWT (jsonwebtoken)
- **トレーシング**: tracing + opentelemetry

## アーキテクチャ

### ヘキサゴナルアーキテクチャ

```
┌─────────────────────────────────────────────────────────┐
│                      Adapters Layer                      │
│  ┌──────────────┐  ┌───────────┐  ┌─────────────────┐  │
│  │   GraphQL    │  │   Auth    │  │     Cache       │  │
│  │   Handler    │  │  Firebase │  │     Redis       │  │
│  └──────┬───────┘  └─────┬─────┘  └────────┬────────┘  │
└─────────┼─────────────────┼─────────────────┼───────────┘
          │                 │                 │
┌─────────┼─────────────────┼─────────────────┼───────────┐
│         ▼                 ▼                 ▼           │
│  ┌──────────────┐  ┌───────────┐  ┌─────────────────┐  │
│  │   GraphQL    │  │   Auth    │  │     Cache       │  │
│  │     Port     │  │   Port    │  │      Port       │  │
│  └──────┬───────┘  └─────┬─────┘  └────────┬────────┘  │
│         │   Ports Layer   │                 │           │
└─────────┼─────────────────┼─────────────────┼───────────┘
          │                 │                 │
┌─────────▼─────────────────▼─────────────────▼───────────┐
│                                                          │
│                     Domain Layer                         │
│  ┌─────────────────────────────────────────────────┐    │
│  │          GraphQL Schema & Resolvers             │    │
│  │               Authentication Logic              │    │
│  │                Cache Strategy                   │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
└──────────────────────────────────────────────────────────┘
          │                 │                 │
┌─────────┼─────────────────┼─────────────────┼───────────┐
│         ▼                 ▼                 ▼           │
│  ┌──────────────┐  ┌───────────┐  ┌─────────────────┐  │
│  │    gRPC      │  │   gRPC    │  │     gRPC        │  │
│  │  Vocabulary  │  │   User    │  │   Learning      │  │
│  │    Client    │  │  Client   │  │    Client       │  │
│  └──────────────┘  └───────────┘  └─────────────────┘  │
│                  Infrastructure Layer                    │
└──────────────────────────────────────────────────────────┘
```

### レイヤーの責務

#### Domain Layer

- GraphQL スキーマ定義
- ビジネスロジック（データ変換、バリデーション）
- 認証・認可ルール
- キャッシュ戦略

#### Ports Layer

```rust
// 認証ポート
#[async_trait]
pub trait AuthenticationPort: Send + Sync {
    async fn verify_token(&self, token: &str) -> Result<UserContext>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair>;
}

// キャッシュポート
#[async_trait]
pub trait CachePort: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl: Duration) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

// サービスクライアントポート
#[async_trait]
pub trait VocabularyServicePort: Send + Sync {
    async fn get_item(&self, id: &str) -> Result<VocabularyItem>;
    async fn create_item(&self, input: CreateItemInput) -> Result<String>;
    // ...
}
```

#### Adapters Layer

- Firebase Auth アダプター（AuthenticationPort の実装）
- Redis アダプター（CachePort の実装）
- gRPC クライアントアダプター（各 ServicePort の実装）

## GraphQL スキーマ統合

### 統合アプローチ

各 Bounded Context のスキーマを手動で統合し、単一の GraphQL エンドポイントとして提供：

```graphql
# 統合されたルートスキーマ
type Query {
  # User Context
  me: User!
  user(id: UUID!): User
  
  # Vocabulary Context
  vocabularyItem(id: UUID!): VocabularyItem
  searchVocabulary(query: String, filters: SearchFilters): VocabularyItemConnection!
  
  # Learning Context
  activeSession: ActiveSession
  session(id: UUID!): LearningSession
  
  # Progress Context
  dailyStats(userId: ID!, date: Date): DailyStats!
  userSummary(userId: ID!): UserSummary!
}

type Mutation {
  # User Context
  signIn(idToken: String!): AuthResult!
  signOut: Boolean!
  
  # Vocabulary Context
  createVocabularyItem(input: CreateVocabularyItemInput!): UUID!
  updateVocabularyItem(id: UUID!, input: UpdateVocabularyItemInput!): Boolean!
  
  # Learning Context
  startSession(input: StartSessionInput!): UUID!
  submitAnswer(sessionId: UUID!, input: AnswerInput!): AnswerResult!
  
  # Progress Context mutations は基本的にイベント駆動で更新
}
```

### DataLoader による最適化

N+1 問題を防ぐため、DataLoader パターンを実装：

```rust
pub struct VocabularyItemLoader {
    client: Arc<dyn VocabularyServicePort>,
}

#[async_trait::async_trait]
impl Loader<String> for VocabularyItemLoader {
    type Value = VocabularyItem;
    type Error = Arc<Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        // バッチでアイテムを取得
        let items = self.client.get_items_batch(keys).await?;
        Ok(items.into_iter().map(|item| (item.id.clone(), item)).collect())
    }
}
```

## 認証・認可

### 認証フローアーキテクチャ

```
Client                API Gateway              Auth Provider           User Service
  │                        │                         │                      │
  ├──Request + Token──────▶│                         │                      │
  │                        ├──Verify Token──────────▶│                      │
  │                        │◀────User Claims─────────┤                      │
  │                        │                         │                      │
  │                        ├──Get User Context──────────────────────────────▶│
  │                        │◀────────────User Data───────────────────────────┤
  │                        │                         │                      │
  │◀──Response + Data──────┤                         │                      │
```

### 認証プロバイダーの抽象化

```rust
// domain/auth/mod.rs
#[derive(Clone, Debug)]
pub struct UserContext {
    pub user_id: String,
    pub email: String,
    pub roles: Vec<Role>,
    pub token_claims: HashMap<String, Value>,
}

#[derive(Clone, Debug)]
pub enum Role {
    User,
    Admin,
}

// ports/auth.rs
#[async_trait]
pub trait AuthenticationPort: Send + Sync {
    async fn verify_token(&self, token: &str) -> Result<UserContext>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair>;
}

// adapters/auth/firebase.rs
pub struct FirebaseAuthAdapter {
    // Firebase specific implementation
}

#[async_trait]
impl AuthenticationPort for FirebaseAuthAdapter {
    async fn verify_token(&self, token: &str) -> Result<UserContext> {
        // Firebase Auth SDK を使用した検証
    }
}
```

### GraphQL コンテキストでの認証

```rust
pub struct GraphQLContext {
    pub user: Option<UserContext>,
    pub dataloaders: DataLoaders,
    pub services: Services,
}

// 認証ミドルウェア
pub async fn auth_middleware(
    State(auth): State<Arc<dyn AuthenticationPort>>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Response {
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(token) = extract_bearer_token(auth_header) {
            if let Ok(user) = auth.verify_token(token).await {
                req.extensions_mut().insert(user);
            }
        }
    }
    next.run(req).await
}
```

## gRPC クライアント統合

### サービス接続管理

```rust
pub struct ServiceClients {
    pub vocabulary_command: VocabularyCommandClient<Channel>,
    pub vocabulary_query: VocabularyQueryClient<Channel>,
    pub user: UserServiceClient<Channel>,
    pub learning: LearningServiceClient<Channel>,
    pub progress: ProgressServiceClient<Channel>,
}

impl ServiceClients {
    pub async fn new(config: &ServiceConfig) -> Result<Self> {
        let vocabulary_command = VocabularyCommandClient::connect(
            config.vocabulary_command_url.clone()
        ).await?;
        
        // 他のクライアントも同様に初期化
        
        Ok(Self {
            vocabulary_command,
            // ...
        })
    }
}
```

### エラーハンドリング

```rust
// gRPC エラーを GraphQL エラーに変換
impl From<tonic::Status> for GraphQLError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            Code::NotFound => GraphQLError::NotFound {
                message: status.message().to_string(),
            },
            Code::InvalidArgument => GraphQLError::ValidationError {
                message: status.message().to_string(),
            },
            Code::Unauthenticated => GraphQLError::Unauthorized,
            _ => GraphQLError::Internal {
                message: "Internal server error".to_string(),
            },
        }
    }
}
```

## キャッシュ戦略

### Redis を使用したキャッシュ層

```rust
pub struct RedisCacheAdapter {
    client: redis::Client,
}

impl RedisCacheAdapter {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl CachePort for RedisCacheAdapter {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut conn = self.client.get_async_connection().await?;
        Ok(conn.get(key).await?)
    }
    
    async fn set(&self, key: &str, value: &[u8], ttl: Duration) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        conn.set_ex(key, value, ttl.as_secs() as usize).await?;
        Ok(())
    }
}
```

### キャッシュポリシー

| データ種別 | TTL | 無効化タイミング |
|-----------|-----|-----------------|
| User Profile | 5分 | プロフィール更新時 |
| Vocabulary Item | 10分 | アイテム更新時 |
| Search Results | 3分 | - |
| Learning Session | キャッシュなし | - |
| Progress Stats | 1分 | イベント受信時 |

### キャッシュキー設計

```
user:profile:{user_id}
vocabulary:item:{item_id}
vocabulary:search:{query_hash}
progress:daily:{user_id}:{date}
```

## 非機能要件

### ヘルスチェック

```rust
// GET /health - 基本的な生存確認
pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339()
    }))
}

// GET /ready - 依存サービスの確認
pub async fn readiness_check(
    State(deps): State<HealthCheckDeps>,
) -> impl IntoResponse {
    let mut checks = HashMap::new();
    
    // Redis 接続確認
    checks.insert("redis", check_redis(&deps.redis).await);
    
    // gRPC サービス確認
    checks.insert("vocabulary_service", check_grpc_service(&deps.vocabulary).await);
    checks.insert("user_service", check_grpc_service(&deps.user).await);
    
    let all_healthy = checks.values().all(|v| *v);
    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (status_code, Json(json!({
        "ready": all_healthy,
        "checks": checks
    })))
}
```

### CORS 設定

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin([
        "http://localhost:3000".parse::<HeaderValue>().unwrap(),
        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
    ])
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    .allow_credentials(true);
```

### レート制限

```rust
use tower::ServiceBuilder;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10)  // 1秒あたり10リクエスト
        .burst_size(30)   // バースト許容量
        .finish()
        .unwrap(),
);

let governor_limiter = ServiceBuilder::new()
    .layer(GovernorLayer {
        config: Box::leak(governor_conf),
    });
```

### モニタリング（Google Cloud 向け）

```rust
use opentelemetry::sdk::trace::{self, RandomIdGenerator, Sampler};
use opentelemetry_otlp::WithExportConfig;

pub fn init_tracing() -> Result<()> {
    // Google Cloud Trace エクスポーター設定
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("https://cloudtrace.googleapis.com");
    
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default()),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    // tracing subscriber 設定
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
    
    Ok(())
}

// メトリクス
pub fn record_graphql_metrics(operation: &str, duration: Duration, success: bool) {
    // Google Cloud Monitoring 向けのメトリクス記録
    metrics::histogram!("graphql_request_duration", duration.as_secs_f64(), 
        "operation" => operation,
        "success" => success.to_string()
    );
    
    metrics::increment_counter!("graphql_request_total",
        "operation" => operation,
        "success" => success.to_string()
    );
}
```

### 環境変数設定

```env
# サーバー設定
PORT=8080
ENVIRONMENT=development

# Redis
REDIS_URL=redis://localhost:6379

# サービス URL
VOCABULARY_COMMAND_SERVICE_URL=http://localhost:50051
VOCABULARY_QUERY_SERVICE_URL=http://localhost:50052
USER_SERVICE_URL=http://localhost:50053
LEARNING_SERVICE_URL=http://localhost:50054
PROGRESS_SERVICE_URL=http://localhost:50055

# 認証
AUTH_PROVIDER=firebase
FIREBASE_PROJECT_ID=your-project-id

# GraphQL
GRAPHQL_PLAYGROUND_ENABLED=true
GRAPHQL_INTROSPECTION_ENABLED=true

# CORS
CORS_ALLOWED_ORIGINS=http://localhost:3000,http://localhost:5173

# レート制限
RATE_LIMIT_PER_SECOND=10
RATE_LIMIT_BURST=30

# モニタリング
OTEL_EXPORTER_OTLP_ENDPOINT=https://cloudtrace.googleapis.com
OTEL_SERVICE_NAME=api-gateway
```

## 実装フェーズ

### Phase 1: 基盤実装（2-3日）

1. **プロジェクト構造のセットアップ**
   - Cargo.toml の依存関係設定
   - 基本的なディレクトリ構造作成

2. **共通機能の実装**
   - 設定管理（config.rs）
   - ヘルスチェックエンドポイント
   - CORS、レート制限の設定
   - エラーハンドリング

3. **認証基盤**
   - AuthenticationPort trait 定義
   - Firebase Auth アダプター実装
   - 認証ミドルウェア

### Phase 2: Vocabulary Context 統合（2日）

1. **gRPC クライアント**
   - VocabularyCommandClient の設定
   - VocabularyQueryClient の設定（モック実装でも可）

2. **GraphQL スキーマ定義**
   - Vocabulary 関連の型定義
   - Query/Mutation の実装

3. **DataLoader 実装**
   - VocabularyItemLoader
   - バッチ取得の最適化

### Phase 3: User Context 統合（2日）

1. **User Service との統合**
   - 認証フローの完成
   - プロフィール管理

2. **Redis キャッシュ層**
   - ユーザー情報のキャッシュ
   - セッション管理

### Phase 4: その他の Context（3-4日）

1. **Learning Context**
   - セッション管理
   - 学習フロー

2. **Progress Context**
   - 統計情報の取得
   - Read Model からのクエリ

## プロジェクト構造

```
services/api_gateway/
├── Cargo.toml
├── build.rs                    # gRPC コード生成
├── src/
│   ├── main.rs                 # エントリーポイント
│   ├── lib.rs                  # ライブラリルート
│   ├── config.rs               # 設定管理
│   ├── error.rs                # エラー定義
│   ├── types.rs                # 共通型定義
│   │
│   ├── domain.rs               # ドメイン層のルート
│   ├── domain/                 # ドメイン層
│   │   ├── auth.rs             # 認証ドメイン
│   │   ├── auth/               # 認証関連
│   │   │   └── context.rs     # UserContext
│   │   ├── cache.rs            # キャッシュ戦略
│   │   └── cache/              # キャッシュ関連
│   │
│   ├── ports.rs                # ポート層のルート
│   ├── ports/                  # ポート層（インターフェース）
│   │   ├── auth.rs             # AuthenticationPort
│   │   ├── cache.rs            # CachePort
│   │   ├── services.rs         # サービスポートのルート
│   │   └── services/           # 各サービスのポート
│   │       ├── vocabulary.rs
│   │       ├── user.rs
│   │       └── learning.rs
│   │
│   ├── adapters.rs             # アダプター層のルート
│   ├── adapters/               # アダプター層（実装）
│   │   ├── auth.rs             # 認証アダプターのルート
│   │   ├── auth/               # 認証アダプター
│   │   │   └── firebase.rs    # Firebase実装
│   │   ├── cache.rs            # キャッシュアダプターのルート
│   │   ├── cache/              # キャッシュアダプター
│   │   │   └── redis.rs       # Redis実装
│   │   ├── grpc.rs             # gRPCクライアントのルート
│   │   └── grpc/               # gRPCクライアント
│   │       ├── vocabulary.rs
│   │       ├── user.rs
│   │       └── learning.rs
│   │
│   ├── graphql.rs              # GraphQL層のルート
│   ├── graphql/                # GraphQL層
│   │   ├── schema.rs           # スキーマ定義
│   │   ├── context.rs          # GraphQLContext
│   │   ├── resolvers.rs        # リゾルバーのルート
│   │   ├── resolvers/          # リゾルバー実装
│   │   │   ├── vocabulary.rs
│   │   │   ├── user.rs
│   │   │   ├── learning.rs
│   │   │   └── progress.rs
│   │   ├── data_loaders.rs     # DataLoaderのルート
│   │   └── data_loaders/       # DataLoader実装
│   │       └── vocabulary.rs
│   │
│   ├── middleware.rs           # ミドルウェアのルート
│   ├── middleware/             # ミドルウェア
│   │   ├── auth.rs             # 認証ミドルウェア
│   │   ├── cors.rs             # CORS設定
│   │   ├── rate_limit.rs      # レート制限
│   │   └── tracing.rs         # トレーシング
│   │
│   └── proto/                  # 生成されたgRPCコード
│       └── ...
│
├── tests/                      # テスト
│   ├── integration/
│   └── e2e/
│
└── .env.example               # 環境変数サンプル
```

### Rust Edition 2024 のモジュール構造

Rust Edition 2024 では、`mod.rs` の代わりに同名のファイルをモジュールのルートとして使用します：

```rust
// src/domain.rs （従来の src/domain/mod.rs の代わり）
pub mod auth;
pub mod cache;

// src/domain/auth.rs （従来の src/domain/auth/mod.rs の代わり）
pub mod context;

pub use context::UserContext;

// src/graphql.rs
pub mod schema;
pub mod context;
pub mod resolvers;
pub mod data_loaders;

// src/graphql/resolvers.rs
pub mod vocabulary;
pub mod user;
pub mod learning;
pub mod progress;

```

この構造により：

- ファイル名がモジュール名と一致し、構造が明確になる
- `mod.rs` の乱立を避けられる
- IDE でのナビゲーションが改善される

## セキュリティ考慮事項

### GraphQL 固有のセキュリティ

1. **クエリ深度制限**

   ```rust
   .limit_depth(10)  // ネストの深さを10に制限
   ```

2. **クエリ複雑度制限**

   ```rust
   .limit_complexity(100)  // 複雑度スコアを100に制限
   ```

3. **イントロスペクションの制御**
   - 本番環境では無効化
   - 開発環境でのみ有効

### 認証・認可

- JWT トークンの有効期限チェック
- Role ベースのアクセス制御（RBAC）
- GraphQL ディレクティブによる認可

  ```graphql
  type Mutation {
    deleteUser(id: ID!): Boolean! @auth(requires: ADMIN)
  }
  ```

## パフォーマンス最適化

### DataLoader によるバッチ処理

- 同一リクエスト内の重複クエリを自動的にバッチ化
- 1つの GraphQL クエリで複数のエンティティを効率的に取得

### Redis キャッシュ

- 頻繁にアクセスされるデータのキャッシュ
- GraphQL レスポンス全体のキャッシュ（特定のクエリ）

### 接続プール

- gRPC チャネルの再利用
- Redis 接続プール
- 適切なタイムアウト設定

## 運用考慮事項

### Cloud Run へのデプロイ

```dockerfile
FROM rust:1.80 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin api_gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api_gateway /usr/local/bin/
EXPOSE 8080
CMD ["api_gateway"]
```

### スケーリング設定

```yaml
# Cloud Run の推奨設定
spec:
  containers:
    - image: gcr.io/project/api-gateway
      resources:
        limits:
          cpu: "2"
          memory: "2Gi"
      env:
        - name: RUST_LOG
          value: "info"
  scaling:
    minInstances: 1
    maxInstances: 100
    concurrency: 100
```

### モニタリングダッシュボード

Google Cloud Monitoring で監視すべきメトリクス：

1. **リクエストメトリクス**
   - リクエスト数/秒
   - レスポンスタイム（P50, P95, P99）
   - エラー率

2. **リソースメトリクス**
   - CPU 使用率
   - メモリ使用量
   - アクティブな接続数

3. **ビジネスメトリクス**
   - 認証成功/失敗率
   - GraphQL クエリ/ミューテーション別の実行数
   - キャッシュヒット率

## 今後の拡張ポイント

### 短期的な改善

1. **GraphQL サブスクリプション**
   - WebSocket サポート（Cloud Run の制限を考慮）
   - Server-Sent Events での代替実装

2. **高度なキャッシング**
   - CDN 統合
   - Edge キャッシング

### 長期的な検討事項

1. **マルチテナント対応**
   - テナント別のレート制限
   - データ分離

2. **A/B テスト機能**
   - フィーチャーフラグ
   - カナリアデプロイメント

3. **GraphQL フェデレーション**
   - 将来的に各サービスが独自の GraphQL を持つ場合
   - Apollo Federation や GraphQL Mesh の採用

## 参考資料

- [async-graphql Documentation](https://async-graphql.github.io/async-graphql/)
- [tonic gRPC Framework](https://github.com/hyperium/tonic)
- [Google Cloud Run Best Practices](https://cloud.google.com/run/docs/tips)
- [GraphQL Security Best Practices](https://www.apollographql.com/blog/graphql/security/)
