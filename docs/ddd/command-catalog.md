# コマンドカタログ

## 概要

このドキュメントでは、Effect システムで使用されるすべてのコマンドを一覧化し、各コマンドの詳細仕様を定義します。

## コマンド命名規則

- 動詞で始まる（例: `CreateWord`, `StartSession`）
- 明確な意図を表現
- アグリゲートに対する操作を示す

## コマンド基本構造

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

// すべてのコマンドが実装するトレイト
pub trait Command: Send + Sync {
    type Result;
    type Error;
    
    fn validate(&self) -> Result<(), ValidationError>;
    fn aggregate_id(&self) -> Option<String>;
}

// コマンドのメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    pub command_id: CommandId,
    pub correlation_id: Option<CorrelationId>,
    pub user_id: UserId,
    pub issued_at: DateTime<Utc>,
    pub trace_id: Option<String>,
}
```

## 1. Word Management Commands

### CreateWord

**概要**: 新しい単語を登録する

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateWord {
    #[validate(length(min = 1, max = 100))]
    pub text: String,
    
    #[validate(length(min = 1, max = 500))]
    pub initial_meaning: String,
    
    pub part_of_speech: PartOfSpeech,
    pub categories: Vec<Category>,
    
    #[validate(range(min = 1, max = 10))]
    pub difficulty: u8,
    
    pub phonetic_ipa: Option<String>,
    pub tags: Vec<Tag>,
}

impl Command for CreateWord {
    type Result = WordId;
    type Error = WordError;
    
    fn validate(&self) -> Result<(), ValidationError> {
        Validate::validate(self)?;
        
        // カスタムバリデーション
        if self.categories.is_empty() {
            return Err(ValidationError::field("categories", "At least one category is required"));
        }
        
        Ok(())
    }
    
    fn aggregate_id(&self) -> Option<String> {
        None // 新規作成なのでまだIDはない
    }
}
```

**バリデーション**:

- テキストは1-100文字
- 意味は1-500文字
- カテゴリは最低1つ必要
- 難易度は1-10

**成功時の結果**: `WordId`

**失敗ケース**:

- `ValidationError`: 入力が不正
- `DuplicateWordError`: 同じテキストの単語が既に存在

### AddMeaningToWord

**概要**: 既存の単語に意味を追加する

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddMeaningToWord {
    pub word_id: WordId,
    
    #[validate(length(min = 1, max = 500))]
    pub meaning_text: String,
    
    pub part_of_speech: PartOfSpeech,
    pub usage_note: Option<String>,
}

impl Command for AddMeaningToWord {
    type Result = MeaningId;
    type Error = WordError;
    
    fn aggregate_id(&self) -> Option<String> {
        Some(self.word_id.to_string())
    }
}
```

**バリデーション**:

- 意味のテキストは1-500文字
- 単語が存在すること
- 意味の数が上限（10個）未満

**成功時の結果**: `MeaningId`

**失敗ケース**:

- `WordNotFound`: 指定された単語が存在しない
- `TooManyMeanings`: 意味の数が上限に達している
- `DuplicateMeaning`: 同じ意味が既に存在

### AddExampleToWord

**概要**: 単語の特定の意味に例文を追加する

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddExampleToWord {
    pub word_id: WordId,
    pub meaning_id: MeaningId,
    
    #[validate(length(min = 5, max = 300))]
    pub sentence: String,
    
    #[validate(length(min = 1, max = 500))]
    pub translation: String,
    
    pub context: Option<Context>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Context {
    Formal,
    Informal,
    Business,
    Academic,
    Literary,
}
```

**バリデーション**:

- 例文は5-300文字
- 翻訳は1-500文字
- 指定された意味が存在すること

**成功時の結果**: `ExampleId`

### UpdateWord

**概要**: 単語の基本情報を更新する

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateWord {
    pub word_id: WordId,
    pub version: u32, // 楽観的ロック用
    
    pub changes: WordUpdateChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WordUpdateChanges {
    pub phonetic_ipa: Option<String>,
    pub phonetic_spelling: Option<String>,
    
    #[validate(range(min = 1, max = 10))]
    pub difficulty: Option<u8>,
    
    pub categories: Option<Vec<Category>>,
    pub tags: Option<Vec<Tag>>,
    pub image_url: Option<Url>,
}
```

**バリデーション**:

- 変更がある場合のみ実行
- バージョンが一致すること（楽観的ロック）

**成功時の結果**: `()`

**失敗ケース**:

- `OptimisticLockError`: バージョンが一致しない
- `NoChangesProvided`: 変更内容が空

### GenerateAudioForWord

**概要**: 単語の音声を生成する

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateAudioForWord {
    pub word_id: WordId,
    pub voice_type: VoiceType,
    pub speed: AudioSpeed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceType {
    AmericanMale,
    AmericanFemale,
    BritishMale,
    BritishFemale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioSpeed {
    Slow,
    Normal,
    Fast,
}
```

**成功時の結果**: `AudioUrl`

### FavoriteWord / UnfavoriteWord

**概要**: 単語をお気に入りに追加/削除する

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteWord {
    pub word_id: WordId,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnfavoriteWord {
    pub word_id: WordId,
    pub user_id: UserId,
}
```

## 2. Learning Commands

### StartLearningSession

**概要**: 学習セッションを開始する

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StartLearningSession {
    pub user_id: UserId,
    pub mode: LearningMode,
    pub config: SessionConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SessionConfiguration {
    #[validate(range(min = 5, max = 100))]
    pub word_count: u32,
    
    pub categories: Vec<Category>,
    pub difficulty_range: (u8, u8),
    pub time_limit: Option<Duration>,
    pub include_favorites_only: bool,
    pub exclude_recently_studied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningMode {
    MultipleChoice { options: u8 },
    Typing,
    Listening,
    Speaking,
    Mixed,
}
```

**バリデーション**:

- 単語数は5-100
- 難易度範囲は1-10
- 時間制限は5分以上

**成功時の結果**: `SessionId`

**失敗ケース**:

- `InsufficientWords`: 条件に合う単語が不足
- `ActiveSessionExists`: 既にアクティブなセッションがある

### SubmitAnswer

**概要**: 問題に対する回答を送信する

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitAnswer {
    pub session_id: SessionId,
    pub question_id: QuestionId,
    pub answer: Answer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Answer {
    MultipleChoice { selected_option: u8 },
    Text { typed_text: String },
    Audio { audio_url: Url },
}
```

**バリデーション**:

- セッションがアクティブであること
- 問題が未回答であること
- 回答形式が問題タイプと一致すること

**成功時の結果**: `AnswerResult`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerResult {
    pub is_correct: bool,
    pub correct_answer: String,
    pub explanation: Option<String>,
    pub quality_rating: u8, // 0-5 for SM-2
}
```

### CompleteSession

**概要**: 学習セッションを完了する

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteSession {
    pub session_id: SessionId,
    pub completion_reason: CompletionReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionReason {
    AllQuestionsAnswered,
    UserRequested,
    TimeLimit,
}
```

**成功時の結果**: `SessionSummary`

### AbandonSession

**概要**: 学習セッションを中断する

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbandonSession {
    pub session_id: SessionId,
    pub reason: Option<String>,
}
```

## 3. User Commands

### RegisterUser

**概要**: 新規ユーザーを登録する

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterUser {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 3, max = 50))]
    pub display_name: String,
    
    pub auth_method: AuthenticationMethod,
    pub timezone: String,
    pub preferred_language: Language,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    EmailPassword {
        #[validate(length(min = 8))]
        password: String,
    },
    OAuth {
        provider: OAuthProvider,
        token: String,
    },
}
```

**バリデーション**:

- メールアドレスの形式
- 表示名は3-50文字
- パスワードは8文字以上
- タイムゾーンが有効

**成功時の結果**: `UserId`

**失敗ケース**:

- `EmailAlreadyExists`: メールアドレスが使用済み
- `InvalidTimezone`: 無効なタイムゾーン

### UpdateUserProfile

**概要**: ユーザープロフィールを更新する

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserProfile {
    pub user_id: UserId,
    pub updates: ProfileUpdates,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProfileUpdates {
    #[validate(length(min = 3, max = 50))]
    pub display_name: Option<String>,
    
    #[validate(length(max = 500))]
    pub bio: Option<String>,
    
    pub avatar_url: Option<Url>,
    pub learning_goals: Option<Vec<LearningGoal>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningGoal {
    pub goal_type: GoalType,
    pub target_value: u32,
    pub deadline: Option<Date>,
}
```

### UpdateUserSettings

**概要**: ユーザー設定を変更する

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserSettings {
    pub user_id: UserId,
    pub settings: SettingsUpdates,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SettingsUpdates {
    #[validate(range(min = 1, max = 100))]
    pub daily_goal: Option<u32>,
    
    pub reminder_enabled: Option<bool>,
    pub reminder_time: Option<Time>,
    pub notification_settings: Option<NotificationPreferences>,
    pub preferred_categories: Option<Vec<Category>>,
    pub sound_enabled: Option<bool>,
    pub dark_mode: Option<bool>,
}
```

### DeleteUser

**概要**: ユーザーアカウントを削除する

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUser {
    pub user_id: UserId,
    pub confirmation_phrase: String,
    pub deletion_reason: Option<String>,
}
```

**バリデーション**:

- 確認フレーズが正しいこと
- アクティブなサブスクリプションがないこと

## 4. コマンドハンドラーの実装例

```rust
use async_trait::async_trait;

#[async_trait]
pub trait CommandHandler<C: Command> {
    async fn handle(
        &self,
        command: C,
        metadata: CommandMetadata,
    ) -> Result<C::Result, C::Error>;
}

// 実装例
pub struct CreateWordHandler {
    word_repository: Arc<dyn WordRepository>,
    event_store: Arc<dyn EventStore>,
}

#[async_trait]
impl CommandHandler<CreateWord> for CreateWordHandler {
    async fn handle(
        &self,
        command: CreateWord,
        metadata: CommandMetadata,
    ) -> Result<WordId, WordError> {
        // バリデーション
        command.validate()?;
        
        // 重複チェック
        if let Some(_) = self.word_repository
            .find_by_text(&command.text)
            .await?
        {
            return Err(WordError::DuplicateWord);
        }
        
        // アグリゲート作成
        let word = Word::create(
            command.text,
            command.initial_meaning,
            metadata.user_id,
        )?;
        
        // イベント生成
        let event = DomainEvent::WordCreated {
            word_id: word.id(),
            text: word.text().to_string(),
            created_by: metadata.user_id,
            created_at: Utc::now(),
        };
        
        // 永続化
        self.event_store
            .append_events(
                &word.id().to_string(),
                0,
                vec![event],
            )
            .await?;
        
        Ok(word.id())
    }
}
```

## 5. コマンドバリデーション

### 共通バリデーションルール

```rust
pub mod validators {
    use validator::ValidationError;
    
    pub fn validate_word_text(text: &str) -> Result<(), ValidationError> {
        if text.trim().is_empty() {
            return Err(ValidationError::new("empty_text"));
        }
        
        if !text.chars().all(|c| c.is_alphabetic() || c.is_whitespace() || c == '-') {
            return Err(ValidationError::new("invalid_characters"));
        }
        
        Ok(())
    }
    
    pub fn validate_categories(categories: &[Category]) -> Result<(), ValidationError> {
        if categories.is_empty() {
            return Err(ValidationError::new("no_categories"));
        }
        
        if categories.len() > 5 {
            return Err(ValidationError::new("too_many_categories"));
        }
        
        Ok(())
    }
}
```

## 6. コマンドの冪等性

冪等性を保証するコマンドには、冪等性キーを含めます：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotentCommand<C: Command> {
    pub idempotency_key: IdempotencyKey,
    pub command: C,
}

impl<C: Command> IdempotentCommand<C> {
    pub fn new(command: C) -> Self {
        Self {
            idempotency_key: IdempotencyKey::generate(),
            command,
        }
    }
}
```

## 7. コマンドのセキュリティ

### 認可チェック

```rust
#[async_trait]
pub trait AuthorizationChecker {
    async fn can_execute<C: Command>(
        &self,
        user_id: UserId,
        command: &C,
    ) -> Result<(), AuthorizationError>;
}

pub struct RbacAuthorizationChecker {
    user_repository: Arc<dyn UserRepository>,
}

#[async_trait]
impl AuthorizationChecker for RbacAuthorizationChecker {
    async fn can_execute<C: Command>(
        &self,
        user_id: UserId,
        command: &C,
    ) -> Result<(), AuthorizationError> {
        let user = self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AuthorizationError::UserNotFound)?;
        
        // コマンドタイプに基づいて権限チェック
        match command.required_role() {
            Role::Admin if !user.is_admin() => {
                Err(AuthorizationError::InsufficientPermissions)
            }
            _ => Ok(()),
        }
    }
}
```

## 更新履歴

- 2025-07-25: 初版作成
