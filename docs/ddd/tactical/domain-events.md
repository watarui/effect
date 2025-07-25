# ドメインイベントカタログ

## 概要

このドキュメントでは、Effect で発生するすべてのドメインイベントを定義します。
イベントは過去に起こった事実を表し、不変です。

## イベント設計の原則

1. **過去形で命名**: 「〜された」「〜した」
2. **不変**: 一度作成されたら変更不可
3. **自己完結**: イベント単体で意味が分かる
4. **順序性**: タイムスタンプで順序を保証

## 基底イベント構造

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// すべてのドメインイベントの基底トレイト
pub trait DomainEvent: Serialize + Send + Sync + 'static {
    fn event_id(&self) -> Uuid;
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> String;
    fn aggregate_type(&self) -> &'static str;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn version(&self) -> u32 { 1 }
}

/// イベントのメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub caused_by: Option<UserId>,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
}

impl EventMetadata {
    pub fn new(caused_by: Option<UserId>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            caused_by,
            correlation_id: None,
            causation_id: None,
        }
    }
}
```

## Learning Context のイベント

### セッション関連イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSessionStarted {
    pub metadata: EventMetadata,
    pub session_id: SessionId,
    pub user_id: UserId,
    pub word_ids: Vec<WordId>,
    pub mode: LearningMode,
    pub config: SessionConfig,
}

impl DomainEvent for LearningSessionStarted {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "learning.session.started" }
    fn aggregate_id(&self) -> String { self.session_id.to_string() }
    fn aggregate_type(&self) -> &'static str { "LearningSession" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionGenerated {
    pub metadata: EventMetadata,
    pub session_id: SessionId,
    pub question_id: QuestionId,
    pub word_id: WordId,
    pub question_type: QuestionType,
    pub question_text: String,
    pub options: Vec<String>,
    pub correct_answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnswered {
    pub metadata: EventMetadata,
    pub session_id: SessionId,
    pub question_id: QuestionId,
    pub word_id: WordId,
    pub submitted_answer: String,
    pub is_correct: bool,
    pub response_time_ms: u32,
    pub quality_rating: QualityRating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSessionCompleted {
    pub metadata: EventMetadata,
    pub session_id: SessionId,
    pub duration_seconds: u32,
    pub total_questions: u32,
    pub correct_answers: u32,
    pub accuracy_percentage: f32,
    pub average_response_time_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSessionAbandoned {
    pub metadata: EventMetadata,
    pub session_id: SessionId,
    pub reason: AbandonReason,
    pub questions_answered: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbandonReason {
    Timeout,
    UserAction,
    SystemError,
}
```

### 進捗関連イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgressUpdated {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub word_id: WordId,
    pub old_sm2_params: SM2Parameters,
    pub new_sm2_params: SM2Parameters,
    pub old_mastery_level: MasteryLevel,
    pub new_mastery_level: MasteryLevel,
    pub next_review_date: Date,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordMastered {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub word_id: WordId,
    pub total_reviews: u32,
    pub days_to_master: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewScheduled {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub word_id: WordId,
    pub scheduled_for: Date,
    pub interval_days: u32,
}
```

## Word Management Context のイベント

### 単語関連イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordCreated {
    pub metadata: EventMetadata,
    pub word_id: WordId,
    pub text: String,
    pub initial_meaning: String,
    pub difficulty: Difficulty,
    pub categories: Vec<Category>,
    pub created_by: UserId,
}

impl DomainEvent for WordCreated {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "word.created" }
    fn aggregate_id(&self) -> String { self.word_id.to_string() }
    fn aggregate_type(&self) -> &'static str { "Word" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordUpdated {
    pub metadata: EventMetadata,
    pub word_id: WordId,
    pub version: u32,
    pub changes: HashMap<String, serde_json::Value>,
    pub updated_by: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordDeleted {
    pub metadata: EventMetadata,
    pub word_id: WordId,
    pub deleted_by: UserId,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeaningAdded {
    pub metadata: EventMetadata,
    pub word_id: WordId,
    pub meaning_id: MeaningId,
    pub meaning_text: String,
    pub part_of_speech: PartOfSpeech,
    pub added_by: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleAdded {
    pub metadata: EventMetadata,
    pub word_id: WordId,
    pub example_id: ExampleId,
    pub meaning_id: MeaningId,
    pub sentence: String,
    pub translation: String,
    pub context: Option<Context>,
    pub added_by: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordEnriched {
    pub metadata: EventMetadata,
    pub word_id: WordId,
    pub enrichment_type: EnrichmentType,
    pub enrichment_data: serde_json::Value,
    pub source: EnrichmentSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnrichmentType {
    Pronunciation,
    Image,
    Audio,
    Etymology,
    Collocations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnrichmentSource {
    AI,
    ExternalAPI,
    UserContribution,
}
```

### 単語関係イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordRelationCreated {
    pub metadata: EventMetadata,
    pub relation_id: RelationId,
    pub word_id: WordId,
    pub related_word_id: WordId,
    pub relation_type: RelationType,
    pub created_by: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordRelationRemoved {
    pub metadata: EventMetadata,
    pub relation_id: RelationId,
    pub removed_by: UserId,
    pub reason: Option<String>,
}
```

## User Context のイベント

### ユーザー管理イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistered {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub email: String,
    pub display_name: String,
    pub auth_method: AuthMethod,
    pub registration_source: RegistrationSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistrationSource {
    Web,
    Mobile,
    API,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileUpdated {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub changes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsChanged {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub setting_name: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningGoalSet {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub goal: LearningGoal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedIn {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccountLocked {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub reason: LockReason,
    pub locked_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockReason {
    TooManyFailedAttempts,
    SecurityConcern,
    UserRequest,
    AdminAction,
}
```

### お気に入り関連イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordFavorited {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub word_id: WordId,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordUnfavorited {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub word_id: WordId,
}
```

## Progress Context のイベント

### 統計・達成イベント

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStatisticsCalculated {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub date: Date,
    pub words_learned: u32,
    pub words_reviewed: u32,
    pub accuracy_rate: f32,
    pub study_time_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakUpdated {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub old_streak: u32,
    pub new_streak: u32,
    pub is_longest_streak: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakBroken {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub previous_streak: u32,
    pub last_study_date: Date,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneAchieved {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub milestone_type: MilestoneType,
    pub milestone_value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneType {
    WordsLearned,
    ConsecutiveDays,
    TotalSessions,
    PerfectSessions,
    CategoryMastery { category: Category },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeAwarded {
    pub metadata: EventMetadata,
    pub user_id: UserId,
    pub badge_id: BadgeId,
    pub badge_name: String,
    pub badge_tier: BadgeTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BadgeTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}
```

## イベント処理パターン

### イベントハンドラーの例

```rust
use async_trait::async_trait;

#[async_trait]
pub trait EventHandler {
    type Event: DomainEvent;

    async fn handle(&self, event: &Self::Event) -> Result<(), HandleError>;
}

/// セッション完了時の進捗更新ハンドラー
pub struct UpdateProgressOnSessionCompleted {
    progress_service: Arc<dyn ProgressService>,
}

#[async_trait]
impl EventHandler for UpdateProgressOnSessionCompleted {
    type Event = LearningSessionCompleted;

    async fn handle(&self, event: &Self::Event) -> Result<(), HandleError> {
        self.progress_service
            .update_user_statistics(
                event.session_id,
                event.accuracy_percentage,
                event.duration_seconds,
            )
            .await?;

        Ok(())
    }
}
```

### イベントの発行

```rust
/// アグリゲート内でのイベント発行
impl LearningSession {
    pub fn complete(&mut self) -> Result<Vec<Box<dyn DomainEvent>>, DomainError> {
        // ビジネスロジック...

        let event = LearningSessionCompleted {
            metadata: EventMetadata::new(Some(self.user_id)),
            session_id: self.id,
            duration_seconds: self.calculate_duration(),
            total_questions: self.questions.len() as u32,
            correct_answers: self.count_correct_answers(),
            accuracy_percentage: self.calculate_accuracy(),
            average_response_time_ms: self.calculate_avg_response_time(),
        };

        self.events.push(Box::new(event));
        Ok(self.events.drain(..).collect())
    }
}
```

## イベントのシリアライズ

```rust
/// イベントストアへの保存形式
#[derive(Debug, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub event_type: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub aggregate_version: u64,
    pub event_data: serde_json::Value,
    pub metadata: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
}

impl EventEnvelope {
    pub fn from_domain_event<E: DomainEvent>(
        event: &E,
        aggregate_version: u64,
    ) -> Result<Self, SerializationError> {
        Ok(Self {
            event_id: event.event_id(),
            event_type: event.event_type().to_string(),
            aggregate_id: event.aggregate_id(),
            aggregate_type: event.aggregate_type().to_string(),
            aggregate_version,
            event_data: serde_json::to_value(event)?,
            metadata: serde_json::to_value(event.metadata())?,
            occurred_at: event.occurred_at(),
        })
    }
}
```

## イベントのバージョニング

```rust
/// イベントのアップキャスト（古いバージョンから新しいバージョンへ）
pub trait EventUpcaster {
    fn upcast(&self, event: &mut serde_json::Value) -> Result<(), UpcastError>;
}

pub struct WordCreatedV1ToV2Upcaster;

impl EventUpcaster for WordCreatedV1ToV2Upcaster {
    fn upcast(&self, event: &mut serde_json::Value) -> Result<(), UpcastError> {
        // V1 には categories フィールドがなかった
        if !event.get("categories").is_some() {
            event["categories"] = serde_json::json!([]);
        }
        Ok(())
    }
}
```

## 更新履歴

- 2025-07-25: 初版作成
