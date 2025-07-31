# Effect サービス構造

## 概要

各マイクロサービスは、ヘキサゴナルアーキテクチャ（ポート＆アダプターパターン）に基づいて実装されています。このドキュメントでは、サービスの内部構造と実装パターンを詳しく説明します。

## ディレクトリ構造

```
services/{service-name}/
├── Cargo.toml              # パッケージ定義
├── Dockerfile              # コンテナイメージ定義
├── proto/                  # Protocol Buffers 定義（gRPC）
│   └── service.proto
├── src/
│   ├── domain/             # ドメイン層（ビジネスロジックの中核）
│   │   ├── mod.rs
│   │   ├── aggregates/     # 集約ルート
│   │   ├── entities/       # エンティティ
│   │   ├── value_objects/  # 値オブジェクト
│   │   ├── events/         # ドメインイベント
│   │   ├── commands/       # コマンド
│   │   ├── services/       # ドメインサービス
│   │   └── errors.rs       # ドメインエラー
│   ├── application/        # アプリケーション層（ユースケース）
│   │   ├── mod.rs
│   │   ├── command_handlers/  # コマンドハンドラー
│   │   ├── query_handlers/    # クエリハンドラー
│   │   ├── services/          # アプリケーションサービス
│   │   └── dto/              # DTOs
│   ├── ports/              # ポート定義（インターフェース）
│   │   ├── mod.rs
│   │   ├── inbound.rs      # インバウンドポート
│   │   └── outbound.rs     # アウトバウンドポート
│   ├── adapters/           # アダプター実装
│   │   ├── mod.rs
│   │   ├── inbound/        # プライマリアダプター
│   │   │   └── grpc.rs     # gRPC サービス実装
│   │   └── outbound/       # セカンダリアダプター
│   │       ├── postgres.rs # PostgreSQL リポジトリ
│   │       ├── redis.rs    # Redis 実装
│   │       └── event_bus.rs # イベントバス実装
│   ├── config.rs           # サービス設定
│   ├── lib.rs              # ライブラリルート
│   ├── main.rs             # エントリーポイント
│   └── server.rs           # サーバー起動ロジック
└── tests/
    ├── integration/        # 統合テスト
    └── fixtures/           # テストフィクスチャ
```

## レイヤーの責務

### 1. Domain 層

ビジネスロジックの中核。外部依存を持たない純粋なドメインモデル。

```rust
// domain/aggregates/learning_session.rs
use crate::domain::{
    entities::SessionItem,
    value_objects::{SessionId, UserId, SessionStatus},
    events::LearningSessionEvent,
    commands::CreateSessionCommand,
    errors::DomainError,
};

pub struct LearningSession {
    id: SessionId,
    user_id: UserId,
    status: SessionStatus,
    items: Vec<SessionItem>,
    version: u64,
}

impl LearningSession {
    pub fn create(command: CreateSessionCommand) -> Result<(Self, Vec<LearningSessionEvent>), DomainError> {
        // ビジネスルールの検証
        if command.item_count == 0 || command.item_count > 100 {
            return Err(DomainError::InvalidItemCount);
        }

        let session = Self {
            id: SessionId::new(),
            user_id: command.user_id,
            status: SessionStatus::NotStarted,
            items: Vec::new(),
            version: 0,
        };

        let event = LearningSessionEvent::Created {
            session_id: session.id.clone(),
            user_id: session.user_id.clone(),
            item_count: command.item_count,
        };

        Ok((session, vec![event]))
    }

    pub fn start(&mut self) -> Result<Vec<LearningSessionEvent>, DomainError> {
        // 状態遷移のビジネスルール
        if self.status != SessionStatus::NotStarted {
            return Err(DomainError::InvalidStateTransition);
        }

        self.status = SessionStatus::InProgress;
        
        Ok(vec![LearningSessionEvent::Started {
            session_id: self.id.clone(),
        }])
    }
}
```

### 2. Application 層

ユースケースの実装。ドメインモデルの操作を調整。

```rust
// application/command_handlers/create_session_handler.rs
use async_trait::async_trait;
use crate::{
    domain::{aggregates::LearningSession, commands::CreateSessionCommand},
    ports::outbound::{LearningSessionRepository, EventBus},
    application::dto::CreateSessionResponse,
};

pub struct CreateSessionHandler<R, E> 
where
    R: LearningSessionRepository,
    E: EventBus,
{
    repository: R,
    event_bus: E,
}

impl<R, E> CreateSessionHandler<R, E>
where
    R: LearningSessionRepository,
    E: EventBus,
{
    pub fn new(repository: R, event_bus: E) -> Self {
        Self { repository, event_bus }
    }

    pub async fn handle(&self, command: CreateSessionCommand) -> Result<CreateSessionResponse, ApplicationError> {
        // ドメインロジックの実行
        let (session, events) = LearningSession::create(command)
            .map_err(ApplicationError::from)?;

        // 永続化
        self.repository.save(&session).await
            .map_err(ApplicationError::from)?;

        // イベント発行
        for event in events {
            self.event_bus.publish(event).await
                .map_err(ApplicationError::from)?;
        }

        Ok(CreateSessionResponse {
            session_id: session.id().to_string(),
        })
    }
}
```

### 3. Ports 層

インターフェース定義。依存関係の逆転を実現。

```rust
// ports/outbound.rs
use async_trait::async_trait;
use crate::domain::{
    aggregates::LearningSession,
    value_objects::SessionId,
    events::LearningSessionEvent,
};

#[async_trait]
pub trait LearningSessionRepository: Send + Sync {
    async fn save(&self, session: &LearningSession) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<LearningSession>, RepositoryError>;
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<LearningSession>, RepositoryError>;
}

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: LearningSessionEvent) -> Result<(), EventBusError>;
}
```

### 4. Adapters 層

外部システムとの統合実装。

```rust
// adapters/outbound/postgres.rs
use async_trait::async_trait;
use sqlx::PgPool;
use crate::{
    domain::aggregates::LearningSession,
    ports::outbound::{LearningSessionRepository, RepositoryError},
};

pub struct PostgresLearningSessionRepository {
    pool: PgPool,
}

#[async_trait]
impl LearningSessionRepository for PostgresLearningSessionRepository {
    async fn save(&self, session: &LearningSession) -> Result<(), RepositoryError> {
        let query = r#"
            INSERT INTO learning_sessions (id, user_id, status, items, version)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE SET
                status = $3,
                items = $4,
                version = $5
            WHERE learning_sessions.version = $5 - 1
        "#;

        let result = sqlx::query(query)
            .bind(session.id())
            .bind(session.user_id())
            .bind(session.status())
            .bind(serde_json::to_value(&session.items())?)
            .bind(session.version())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::OptimisticLockError);
        }

        Ok(())
    }
}
```

## gRPC サービス実装

```rust
// adapters/inbound/grpc.rs
use tonic::{Request, Response, Status};
use crate::{
    proto::{
        learning_service_server::LearningService,
        CreateSessionRequest, CreateSessionResponse,
    },
    application::command_handlers::CreateSessionHandler,
};

pub struct LearningServiceImpl<H> {
    create_session_handler: H,
}

#[tonic::async_trait]
impl<H> LearningService for LearningServiceImpl<H>
where
    H: CreateSessionHandler + Send + Sync + 'static,
{
    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionResponse>, Status> {
        let command = request.into_inner().try_into()
            .map_err(|e| Status::invalid_argument(format!("{}", e)))?;

        let response = self.create_session_handler.handle(command).await
            .map_err(|e| Status::internal(format!("{}", e)))?;

        Ok(Response::new(response.into()))
    }
}
```

## 依存性注入

```rust
// main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 設定読み込み
    let config = Config::from_env()?;
    
    // インフラストラクチャの構築
    let db_pool = PgPool::connect(&config.database_url).await?;
    let redis_client = redis::Client::open(config.redis_url)?;
    
    // アダプターの構築
    let repository = PostgresLearningSessionRepository::new(db_pool);
    let event_bus = RedisEventBus::new(redis_client);
    
    // ハンドラーの構築
    let create_session_handler = CreateSessionHandler::new(
        repository.clone(),
        event_bus.clone(),
    );
    
    // gRPC サービスの構築
    let service = LearningServiceImpl::new(create_session_handler);
    
    // サーバー起動
    Server::builder()
        .add_service(LearningServiceServer::new(service))
        .serve(config.grpc_addr())
        .await?;
    
    Ok(())
}
```

## エラーハンドリング

```rust
// domain/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid item count: must be between 1 and 100")]
    InvalidItemCount,
    
    #[error("Invalid state transition")]
    InvalidStateTransition,
    
    #[error("Session not found")]
    SessionNotFound,
}

// application/errors.rs
#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
    
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    
    #[error("Event bus error: {0}")]
    EventBusError(#[from] EventBusError),
}
```

## テスト戦略

### 単体テスト（Domain 層）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_session_with_valid_command() {
        // Given
        let command = CreateSessionCommand {
            user_id: UserId::new(),
            item_count: 50,
        };

        // When
        let result = LearningSession::create(command);

        // Then
        assert!(result.is_ok());
        let (session, events) = result.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(session.status(), SessionStatus::NotStarted);
    }
}
```

### 統合テスト（Adapters 層）

```rust
#[tokio::test]
async fn should_save_and_retrieve_session() {
    // テスト用データベースのセットアップ
    let pool = test_helpers::setup_test_db().await;
    let repository = PostgresLearningSessionRepository::new(pool);

    // Given
    let session = test_helpers::create_test_session();

    // When
    repository.save(&session).await.unwrap();
    let retrieved = repository.find_by_id(session.id()).await.unwrap();

    // Then
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id(), session.id());
}
```

## 設定管理

```rust
// config.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub grpc_port: u16,
    pub service_name: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::new();
        
        cfg.merge(config::Environment::new())?;
        
        cfg.try_into()
    }
    
    pub fn grpc_addr(&self) -> std::net::SocketAddr {
        ([0, 0, 0, 0], self.grpc_port).into()
    }
}
```

## ロギングとトレーシング

```rust
// server.rs
use tracing::{info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(service_name: &str) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(
            opentelemetry_jaeger::new_pipeline()
                .with_service_name(service_name)
                .install_simple()
                .unwrap(),
        ))
        .init();
}

#[instrument(skip(handler))]
pub async fn handle_request<H>(handler: H, request: Request) -> Response {
    info!("Processing request");
    // リクエスト処理
}
```

## メトリクス収集

```rust
use prometheus::{register_counter, Counter};

lazy_static! {
    static ref SESSION_CREATED_COUNTER: Counter = register_counter!(
        "learning_sessions_created_total",
        "Total number of learning sessions created"
    ).unwrap();
}

// ハンドラー内で使用
SESSION_CREATED_COUNTER.inc();
```

## まとめ

この構造により：

1. **テスタビリティ**: 各層が独立してテスト可能
2. **保守性**: 関心事の分離により変更が容易
3. **拡張性**: 新しいアダプターの追加が簡単
4. **ドメイン中心**: ビジネスロジックが技術的詳細から独立

各サービスはこの基本構造を維持しながら、それぞれの Bounded Context の要件に応じて実装されます。
