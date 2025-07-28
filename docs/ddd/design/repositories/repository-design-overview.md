# リポジトリ設計の概要

## 概要

このドキュメントでは、Effect プロジェクトにおけるリポジトリパターンの設計原則と共通仕様を定義します。
リポジトリは、集約の永続化と取得を抽象化し、ドメイン層をインフラストラクチャの詳細から分離する重要な役割を担います。

## リポジトリパターンの目的

### 1. 永続化の抽象化

- ドメインモデルをデータベースの詳細から分離
- 集約単位での一貫した永続化を保証
- テスト容易性の向上（インメモリ実装への差し替え）

### 2. 集約境界の保護

- 集約ルートを通じたアクセスの強制
- トランザクション境界の明確化（1 トランザクション 1 集約）
- 不変条件の維持

### 3. クエリの集約化

- ドメイン特有のクエリメソッドの提供
- パフォーマンスを考慮した専用メソッド
- 複雑なクエリのカプセル化

## 基本設計原則

### 1. インターフェースと実装の分離

```rust
// ドメイン層：インターフェース定義
use async_trait::async_trait;

#[async_trait]
pub trait LearningSessionRepository {
    type Error;

    async fn find_by_id(&self, id: &SessionId) -> Result<Option<LearningSession>, Self::Error>;
    async fn save(&self, session: &LearningSession) -> Result<(), Self::Error>;
}

// インフラ層：実装（例）
pub struct PostgresLearningSessionRepository {
    pool: sqlx::PgPool,
}

impl LearningSessionRepository for PostgresLearningSessionRepository {
    type Error = RepositoryError;
    // 実装...
}
```

### 2. 集約単位での操作

- リポジトリは集約ルートに対してのみ定義
- 集約内部のエンティティへの直接アクセスは禁止
- 集約全体の整合性を保証

### 3. 明示的なエラーハンドリング

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Entity not found")]
    NotFound,

    #[error("Optimistic lock conflict: current version {current_version}, expected {expected_version}")]
    OptimisticLockConflict {
        current_version: u32,
        expected_version: u32
    },

    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
```

## 共通インターフェース

### 基本的な CRUD 操作

```rust
use async_trait::async_trait;
use std::fmt::Debug;

/// 基本的なリポジトリトレイト
#[async_trait]
pub trait Repository<T, ID>
where
    ID: Debug + Send + Sync,
    T: Send + Sync,
{
    type Error: std::error::Error + Send + Sync;

    /// IDで集約を取得
    async fn find_by_id(&self, id: &ID) -> Result<Option<T>, Self::Error>;

    /// 集約を保存（新規作成または更新）
    async fn save(&self, aggregate: &T) -> Result<(), Self::Error>;

    /// 集約を削除
    async fn delete(&self, id: &ID) -> Result<(), Self::Error>;

    /// 複数の集約を一括取得（N+1問題の回避）
    async fn find_by_ids(&self, ids: &[ID]) -> Result<Vec<T>, Self::Error>;
}
```

### ページネーション

```rust
/// ページネーションリクエスト
#[derive(Debug, Clone)]
pub struct PageRequest {
    pub page: u32,        // 0-indexed
    pub size: u32,        // 1ページあたりの件数
    pub sort: Option<Sort>,
}

/// ソート指定
#[derive(Debug, Clone)]
pub struct Sort {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// ページネーション結果
#[derive(Debug)]
pub struct Page<T> {
    pub content: Vec<T>,
    pub total_elements: u64,
    pub total_pages: u32,
    pub current_page: u32,
    pub page_size: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

impl PageRequest {
    pub fn new(page: u32, size: u32) -> Self {
        Self { page, size, sort: None }
    }

    pub fn with_sort(mut self, field: &str, direction: SortDirection) -> Self {
        self.sort = Some(Sort {
            field: field.to_string(),
            direction,
        });
        self
    }
}
```

### 楽観的ロック

```rust
/// バージョン管理された集約
pub trait VersionedAggregate {
    fn version(&self) -> u32;
    fn increment_version(&mut self);
}

/// バージョンチェック付きリポジトリ
#[async_trait]
pub trait VersionedRepository<T, ID>: Repository<T, ID>
where
    T: VersionedAggregate + Send + Sync,
    ID: Debug + Send + Sync,
{
    /// バージョンチェック付きで保存
    async fn save_with_version_check(&self, aggregate: &T) -> Result<(), Self::Error>;
}
```

## トランザクション管理

### 原則

1. **1 トランザクション 1 集約**

   - 1 つのトランザクションで更新するのは 1 つの集約のみ
   - 複数集約の更新が必要な場合は、ドメインイベントで結果整合性を実現

2. **Unit of Work パターン**（オプション）

   ```rust
   #[async_trait]
   pub trait UnitOfWork: Send + Sync {
       type Error: std::error::Error + Send + Sync;

       async fn begin(&mut self) -> Result<(), Self::Error>;
       async fn commit(&mut self) -> Result<(), Self::Error>;
       async fn rollback(&mut self) -> Result<(), Self::Error>;
   }
   ```

## イベントストア（Progress Context 用）

```rust
use chrono::{DateTime, Utc};

/// イベントストア用の特別なリポジトリ
#[async_trait]
pub trait EventStore: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    /// イベントを追記
    async fn append_events(
        &self,
        stream_id: &str,
        events: Vec<DomainEvent>,
        expected_version: Option<u64>,
    ) -> Result<(), Self::Error>;

    /// イベントを読み取り
    async fn read_events(
        &self,
        stream_id: &str,
        from_version: u64,
        to_version: Option<u64>,
    ) -> Result<Vec<PersistedEvent>, Self::Error>;

    /// 全イベントをストリーミング（大量データ用）
    async fn stream_all_events(
        &self,
        from_position: u64,
        batch_size: u32,
    ) -> Result<EventStream, Self::Error>;
}

/// 永続化されたイベント
pub struct PersistedEvent {
    pub event_id: EventId,
    pub stream_id: String,
    pub version: u64,
    pub event_type: String,
    pub event_data: DomainEvent,
    pub metadata: EventMetadata,
    pub created_at: DateTime<Utc>,
}
```

## 実装ガイドライン

### 1. 非同期処理

- すべてのリポジトリメソッドは `async` で定義
- `async-trait` クレートを使用してトレイトに非同期メソッドを定義
- タイムアウト処理の考慮

### 2. エラー型

- 各リポジトリは `RepositoryError` を使用、または独自のエラー型を定義
- `thiserror` クレートで人間に読みやすいエラーメッセージ

### 3. テスタビリティ

```rust
// テスト用のインメモリ実装例
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct InMemoryLearningSessionRepository {
    sessions: Arc<Mutex<HashMap<SessionId, LearningSession>>>,
}

impl InMemoryLearningSessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
```

### 4. 依存性注入

```rust
// アプリケーションサービスでの使用例
pub struct LearningApplicationService<SR, UR>
where
    SR: LearningSessionRepository,
    UR: UserItemRecordRepository,
{
    session_repo: Arc<SR>,
    user_item_repo: Arc<UR>,
}
```

## 設計上の考慮事項

### 1. パフォーマンス最適化

- N+1 問題の回避：`find_by_ids` メソッドの提供
- 必要に応じて専用のクエリメソッドを追加
- インデックスを考慮したクエリ設計

### 2. キャッシュ戦略

- リポジトリ実装層でのキャッシュは可能
- ドメイン層はキャッシュの存在を意識しない
- キャッシュの無効化戦略を明確に

### 3. 監査とロギング

- 重要な操作（作成、更新、削除）のログ記録
- 誰が、いつ、何を変更したかの追跡
- デバッグ用の詳細ログ（開発環境）

### 4. マルチテナンシー（将来の拡張）

- テナント ID によるデータ分離
- リポジトリレベルでの透過的な処理

## コンテキスト別リポジトリ

各コンテキストの詳細なリポジトリ設計：

1. [Learning Context](./learning-context-repositories.md)

   - LearningSessionRepository
   - UserItemRecordRepository

2. Vocabulary Context（作成予定）

   - VocabularyEntryRepository
   - VocabularyItemRepository

3. Learning Algorithm Context（作成予定）

   - ItemLearningRecordRepository

4. Progress Context（作成予定）

   - EventStore
   - 各種 ProjectionRepository

5. AI Integration Context（作成予定）

   - AIGenerationTaskRepository
   - ChatSessionRepository

6. User Context（作成予定）
   - UserProfileRepository

## 更新履歴

- 2025-07-28: 初版作成（教科書的な DDD アプローチで再構成）
