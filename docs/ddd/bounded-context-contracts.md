# 境界づけられたコンテキスト間の契約

## 概要

このドキュメントでは、各境界づけられたコンテキスト間の API 仕様、イベントスキーマ、データ形式を定義します。

## 1. User Context → Learning Context

### 認証情報の共有（Shared Kernel）

```rust
// 共有される構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<Role>,
}

// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub email: String,
    pub roles: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}
```

### API エンドポイント

```graphql
# Learning Context が User Context から取得する情報
type Query {
  # 現在のユーザー情報を取得
  currentUser: User!
  
  # ユーザーの学習設定を取得
  userSettings(userId: ID!): UserSettings!
}

type User {
  id: ID!
  email: String!
  displayName: String!
  timezone: String!
  preferredLanguage: String!
}

type UserSettings {
  dailyGoal: Int!
  reminderEnabled: Boolean!
  reminderTime: String
  preferredCategories: [Category!]!
}
```

## 2. Word Management → Learning Context

### 単語データの公開 API（Published Language）

```rust
// Word Management が公開するトレイト
#[async_trait]
pub trait WordQueryService: Send + Sync {
    /// 学習用の単語データを取得
    async fn get_words_for_learning(
        &self,
        criteria: LearningCriteria,
    ) -> Result<Vec<WordData>, ServiceError>;
    
    /// 単語の詳細情報を取得
    async fn get_word_details(
        &self,
        word_ids: Vec<WordId>,
    ) -> Result<Vec<WordDetail>, ServiceError>;
}

// 標準化されたデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordData {
    pub id: WordId,
    pub text: String,
    pub difficulty: u8,
    pub categories: Vec<Category>,
    pub cefr_level: CefrLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordDetail {
    pub id: WordId,
    pub text: String,
    pub phonetic_ipa: String,
    pub meanings: Vec<MeaningData>,
    pub examples: Vec<ExampleData>,
    pub audio_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningCriteria {
    pub user_id: UserId,
    pub categories: Vec<Category>,
    pub difficulty_range: (u8, u8),
    pub limit: u32,
    pub exclude_learned: bool,
}
```

### gRPC サービス定義

```protobuf
syntax = "proto3";

package word_management;

service WordService {
  rpc GetWordsForLearning(LearningCriteria) returns (WordList);
  rpc GetWordDetails(WordIdList) returns (WordDetailList);
}

message LearningCriteria {
  string user_id = 1;
  repeated string categories = 2;
  uint32 min_difficulty = 3;
  uint32 max_difficulty = 4;
  uint32 limit = 5;
  bool exclude_learned = 6;
}

message WordData {
  string id = 1;
  string text = 2;
  uint32 difficulty = 3;
  repeated string categories = 4;
  string cefr_level = 5;
}

message WordList {
  repeated WordData words = 1;
}
```

## 3. Learning Context → Progress Context

### ドメインイベント（Domain Events）

```rust
// イベントの基本構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: EventId,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

// Learning Context が発行するイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum LearningEvent {
    SessionStarted {
        session_id: SessionId,
        user_id: UserId,
        word_count: u32,
        mode: LearningMode,
    },
    
    QuestionAnswered {
        session_id: SessionId,
        user_id: UserId,
        word_id: WordId,
        is_correct: bool,
        response_time_ms: u32,
        quality_rating: u8,
    },
    
    SessionCompleted {
        session_id: SessionId,
        user_id: UserId,
        duration_seconds: u32,
        correct_count: u32,
        total_count: u32,
        words_studied: Vec<WordProgress>,
    },
    
    WordMastered {
        user_id: UserId,
        word_id: WordId,
        mastery_level: f32,
        total_reviews: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordProgress {
    pub word_id: WordId,
    pub was_correct: bool,
    pub quality_rating: u8,
    pub new_interval_days: u32,
}
```

### Pub/Sub トピック

```yaml
topics:
  - name: learning-events
    publishers:
      - learning-context
    subscribers:
      - progress-context
      - analytics-service
    
  - name: user-events
    publishers:
      - user-context
    subscribers:
      - learning-context
      - progress-context
```

## 4. Progress Context → All Contexts

### 統計情報 API

```graphql
type Query {
  # ユーザーの学習統計を取得
  learningStatistics(userId: ID!, period: Period!): Statistics!
  
  # カテゴリ別の進捗
  categoryProgress(userId: ID!): [CategoryProgress!]!
  
  # リーダーボード
  leaderboard(period: Period!, limit: Int = 10): [LeaderboardEntry!]!
}

type Statistics {
  totalWordsLearned: Int!
  totalSessions: Int!
  totalMinutes: Int!
  averageAccuracy: Float!
  currentStreak: Int!
  longestStreak: Int!
}

type CategoryProgress {
  category: Category!
  wordsLearned: Int!
  wordsTotal: Int!
  averageAccuracy: Float!
  lastStudied: DateTime
}

enum Period {
  DAILY
  WEEKLY
  MONTHLY
  ALL_TIME
}
```

## 5. 共通データ型定義

### 列挙型

```rust
// カテゴリ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    General,
    Business,
    Academic,
    IELTS,
    TOEIC,
    TOEFL,
    Daily,
    Technical,
}

// CEFR レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CefrLevel {
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
}

// 学習モード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningMode {
    MultipleChoice,
    Typing,
    Listening,
    Speaking,
}
```

### ID 型の相互変換

```rust
// すべての ID 型は UUID ベース
impl From<UserId> for String {
    fn from(id: UserId) -> Self {
        id.0.to_string()
    }
}

impl TryFrom<String> for UserId {
    type Error = uuid::Error;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(UserId(Uuid::parse_str(&value)?))
    }
}

// JSON シリアライゼーション
impl Serialize for UserId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
```

## 6. エラー処理の契約

### 標準エラー形式

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub trace_id: String,
}

// GraphQL エラー拡張
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphQLErrorExtension {
    pub code: String,
    pub trace_id: String,
    pub details: Option<serde_json::Value>,
}

// エラーコード一覧
pub mod error_codes {
    pub const WORD_NOT_FOUND: &str = "WORD_NOT_FOUND";
    pub const USER_NOT_FOUND: &str = "USER_NOT_FOUND";
    pub const SESSION_EXPIRED: &str = "SESSION_EXPIRED";
    pub const INVALID_ANSWER: &str = "INVALID_ANSWER";
    pub const QUOTA_EXCEEDED: &str = "QUOTA_EXCEEDED";
}
```

## 7. 非同期通信の保証

### メッセージ配信保証

```rust
// At-least-once 配信を保証
pub struct MessageEnvelope {
    pub id: MessageId,
    pub correlation_id: Option<CorrelationId>,
    pub payload: Vec<u8>,
    pub content_type: String,
    pub timestamp: DateTime<Utc>,
    pub retry_count: u32,
}

// 冪等性キー
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn from_event(event: &DomainEvent) -> Self {
        let key = format!("{}:{}", event.aggregate_id(), event.version());
        Self(key)
    }
}
```

### サーキットブレーカー設定

```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 3,
        }
    }
}
```

## 8. バージョニング戦略

### API バージョニング

```rust
// GraphQL スキーマディレクティブ
#[derive(Debug)]
pub struct Deprecated {
    pub reason: String,
    pub removed_in: Option<String>,
}

// REST API バージョニング
pub const API_VERSION_HEADER: &str = "X-API-Version";
pub const CURRENT_VERSION: &str = "v1";
pub const SUPPORTED_VERSIONS: &[&str] = &["v1"];
```

### イベントバージョニング

```rust
// イベントのアップキャスト
pub trait EventUpcaster {
    fn upcast(&self, event: &[u8], version: u32) -> Result<DomainEvent, UpcastError>;
}

pub struct EventUpcasterChain {
    upcasters: Vec<Box<dyn EventUpcaster>>,
}

impl EventUpcasterChain {
    pub fn upcast(&self, event: &[u8], version: u32) -> Result<DomainEvent, UpcastError> {
        let current_version = CURRENT_EVENT_VERSION;
        let mut result = event.to_vec();
        
        for v in version..current_version {
            if let Some(upcaster) = self.upcasters.get(v as usize) {
                result = upcaster.upcast(&result, v)?;
            }
        }
        
        serde_json::from_slice(&result).map_err(Into::into)
    }
}
```

## 9. モニタリングと可観測性

### トレーシング

```rust
// OpenTelemetry スパン属性
pub mod span_attributes {
    pub const USER_ID: &str = "effect.user_id";
    pub const SESSION_ID: &str = "effect.session_id";
    pub const WORD_ID: &str = "effect.word_id";
    pub const CONTEXT: &str = "effect.context";
    pub const OPERATION: &str = "effect.operation";
}

// トレースコンテキストの伝播
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub baggage: HashMap<String, String>,
}
```

### メトリクス

```rust
// Prometheus メトリクス定義
pub mod metrics {
    use prometheus::{Counter, Histogram, IntGauge};
    
    lazy_static! {
        pub static ref API_REQUESTS_TOTAL: Counter = Counter::new(
            "effect_api_requests_total",
            "Total number of API requests"
        ).unwrap();
        
        pub static ref API_REQUEST_DURATION: Histogram = Histogram::new(
            "effect_api_request_duration_seconds",
            "API request duration in seconds"
        ).unwrap();
        
        pub static ref ACTIVE_SESSIONS: IntGauge = IntGauge::new(
            "effect_active_sessions",
            "Number of active learning sessions"
        ).unwrap();
    }
}
```

## 更新履歴

- 2025-07-25: 初版作成
