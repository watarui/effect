# DDD 実装ガイド

## 概要

このドキュメントでは、DDD の概念を Rust コードに落とし込む際の具体的なパターンとサンプルコードを提供します。

## エンティティの実装

### 基本パターン

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ID 値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WordId(Uuid);

impl WordId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

// エンティティの実装
#[derive(Debug, Clone)]
pub struct Word {
    // 識別子
    id: WordId,
    
    // プロパティ
    text: WordText,
    phonetic_ipa: Phonetic,
    
    // 子エンティティ
    meanings: Vec<Meaning>,
    
    // メタデータ
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: u32, // 楽観的ロック用
}

impl Word {
    // ファクトリメソッド
    pub fn create(
        text: String,
        initial_meaning: String,
        created_by: UserId,
    ) -> Result<Self, DomainError> {
        let word_text = WordText::new(text)?;
        let meaning = Meaning::new(initial_meaning, created_by)?;
        
        Ok(Self {
            id: WordId::new(),
            text: word_text,
            phonetic_ipa: Phonetic::empty(),
            meanings: vec![meaning],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 0,
        })
    }
    
    // ビジネスロジック
    pub fn add_meaning(
        &mut self,
        text: String,
        added_by: UserId,
    ) -> Result<MeaningId, DomainError> {
        // 不変条件のチェック
        if self.meanings.len() >= 10 {
            return Err(DomainError::TooManyMeanings);
        }
        
        let meaning = Meaning::new(text, added_by)?;
        let meaning_id = meaning.id();
        
        self.meanings.push(meaning);
        self.updated_at = Utc::now();
        self.version += 1;
        
        Ok(meaning_id)
    }
    
    // ゲッター（不変参照のみ）
    pub fn id(&self) -> WordId {
        self.id
    }
    
    pub fn text(&self) -> &WordText {
        &self.text
    }
}
```

## 値オブジェクトの実装

### 基本パターン

```rust
use std::fmt;

// シンプルな値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Result<Self, ValidationError> {
        let email = email.trim().to_lowercase();
        
        if Self::is_valid(&email) {
            Ok(Self(email))
        } else {
            Err(ValidationError::InvalidEmail)
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    fn is_valid(email: &str) -> bool {
        // 簡易的なバリデーション
        email.contains('@') && email.len() >= 3
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// 複雑な値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SM2Parameters {
    repetition_count: u32,
    easiness_factor: f32,
    interval_days: u32,
}

impl SM2Parameters {
    pub fn new() -> Self {
        Self {
            repetition_count: 0,
            easiness_factor: 2.5,
            interval_days: 1,
        }
    }
    
    // ビジネスロジックを含むメソッド
    pub fn calculate_next(
        &self,
        quality: QualityRating,
    ) -> (Self, NextReviewDate) {
        let mut new_params = self.clone();
        
        // SM-2 アルゴリズムの実装
        new_params.update_easiness_factor(quality);
        new_params.calculate_interval(quality);
        
        let next_date = NextReviewDate::from_days(new_params.interval_days);
        
        (new_params, next_date)
    }
    
    fn update_easiness_factor(&mut self, quality: QualityRating) {
        let q = quality.value() as f32;
        self.easiness_factor = (self.easiness_factor + 0.1 
            - (5.0 - q) * (0.08 + (5.0 - q) * 0.02))
            .max(1.3);
    }
}
```

## アグリゲートの実装

### アグリゲートルート

```rust
use std::collections::HashMap;

// アグリゲートルート
pub struct LearningSession {
    // アグリゲート ID
    id: SessionId,
    
    // 不変条件を守るための状態
    state: SessionState,
    
    // 子エンティティ（アグリゲート内でのみ操作）
    questions: Vec<Question>,
    
    // その他のプロパティ
    user_id: UserId,
    config: SessionConfig,
}

impl LearningSession {
    // アグリゲートの生成はファクトリメソッドで
    pub fn start(
        user_id: UserId,
        word_ids: Vec<WordId>,
        config: SessionConfig,
    ) -> Result<(Self, Vec<DomainEvent>), DomainError> {
        // 不変条件のチェック
        if word_ids.is_empty() {
            return Err(DomainError::NoWordsSelected);
        }
        
        if word_ids.len() > config.max_words() {
            return Err(DomainError::TooManyWords);
        }
        
        let session = Self {
            id: SessionId::new(),
            state: SessionState::InProgress,
            questions: vec![],
            user_id,
            config,
        };
        
        // ドメインイベントの生成
        let event = DomainEvent::SessionStarted {
            session_id: session.id,
            user_id,
            word_count: word_ids.len() as u32,
            started_at: Utc::now(),
        };
        
        Ok((session, vec![event]))
    }
    
    // アグリゲート操作は必ず Result を返す
    pub fn submit_answer(
        &mut self,
        question_id: QuestionId,
        answer: String,
    ) -> Result<Vec<DomainEvent>, DomainError> {
        // 状態チェック
        self.ensure_in_progress()?;
        
        // 子エンティティの操作
        let question = self.questions
            .iter_mut()
            .find(|q| q.id() == question_id)
            .ok_or(DomainError::QuestionNotFound)?;
        
        let is_correct = question.submit_answer(answer)?;
        
        // イベント生成
        let event = DomainEvent::AnswerSubmitted {
            session_id: self.id,
            question_id,
            is_correct,
            submitted_at: Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    // 不変条件の確認
    fn ensure_in_progress(&self) -> Result<(), DomainError> {
        match self.state {
            SessionState::InProgress => Ok(()),
            _ => Err(DomainError::SessionNotInProgress),
        }
    }
}
```

## ドメインサービスの実装

### インターフェース定義

```rust
use async_trait::async_trait;

// ドメインサービスのトレイト
#[async_trait]
pub trait WordSelectionService: Send + Sync {
    async fn select_words_for_session(
        &self,
        user_id: UserId,
        criteria: SelectionCriteria,
    ) -> Result<Vec<WordId>, DomainError>;
}

// 実装
pub struct SmartWordSelectionService {
    word_repository: Arc<dyn WordRepository>,
    progress_repository: Arc<dyn UserProgressRepository>,
}

#[async_trait]
impl WordSelectionService for SmartWordSelectionService {
    async fn select_words_for_session(
        &self,
        user_id: UserId,
        criteria: SelectionCriteria,
    ) -> Result<Vec<WordId>, DomainError> {
        // 復習が必要な単語を取得
        let review_words = self.progress_repository
            .find_words_for_review(user_id, Utc::today())
            .await?;
        
        // 新規学習単語を取得
        let new_words = self.word_repository
            .find_unlearned_words(user_id, criteria)
            .await?;
        
        // ビジネスロジック：優先順位付けと選択
        let selected = self.prioritize_and_select(
            review_words,
            new_words,
            criteria.target_count,
        );
        
        Ok(selected)
    }
}
```

## リポジトリの実装

### トレイト定義

```rust
#[async_trait]
pub trait WordRepository: Send + Sync {
    // 基本的な CRUD
    async fn find_by_id(&self, id: WordId) -> Result<Option<Word>, RepositoryError>;
    async fn save(&self, word: &Word) -> Result<(), RepositoryError>;
    async fn delete(&self, id: WordId) -> Result<(), RepositoryError>;
    
    // ドメイン固有のクエリ
    async fn find_by_text(&self, text: &str) -> Result<Option<Word>, RepositoryError>;
    async fn find_by_category(
        &self,
        category: Category,
        limit: usize,
    ) -> Result<Vec<Word>, RepositoryError>;
}

// infrastructure 層での実装
pub struct PostgresWordRepository {
    pool: PgPool,
}

#[async_trait]
impl WordRepository for PostgresWordRepository {
    async fn find_by_id(&self, id: WordId) -> Result<Option<Word>, RepositoryError> {
        let record = sqlx::query_as!(
            WordRecord,
            r#"
            SELECT id, text, phonetic_ipa, created_at, updated_at, version
            FROM words
            WHERE id = $1
            "#,
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::from)?;
        
        match record {
            Some(r) => {
                let word = self.reconstruct_word(r).await?;
                Ok(Some(word))
            }
            None => Ok(None),
        }
    }
    
    // プライベートメソッドで再構築ロジック
    async fn reconstruct_word(&self, record: WordRecord) -> Result<Word, RepositoryError> {
        // 関連エンティティの取得
        let meanings = self.fetch_meanings(record.id).await?;
        
        // アグリゲートの再構築
        Word::reconstruct(
            WordId::from_uuid(record.id),
            record.text,
            record.phonetic_ipa,
            meanings,
            record.created_at,
            record.updated_at,
            record.version,
        )
        .map_err(RepositoryError::from)
    }
}
```

## ドメインイベントの実装

### イベント定義

```rust
use serde::{Deserialize, Serialize};

// すべてのドメインイベントを列挙
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DomainEvent {
    // Word Management Context
    WordCreated {
        word_id: WordId,
        text: String,
        created_by: UserId,
        created_at: DateTime<Utc>,
    },
    WordMeaningAdded {
        word_id: WordId,
        meaning_id: MeaningId,
        text: String,
        added_by: UserId,
    },
    
    // Learning Context
    SessionStarted {
        session_id: SessionId,
        user_id: UserId,
        word_count: u32,
        started_at: DateTime<Utc>,
    },
    AnswerSubmitted {
        session_id: SessionId,
        question_id: QuestionId,
        is_correct: bool,
        submitted_at: DateTime<Utc>,
    },
}

// イベントのメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: EventId,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event: DomainEvent,
    pub version: u32,
    pub occurred_at: DateTime<Utc>,
}
```

## エラーハンドリング

### ドメインエラーの定義

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    // バリデーションエラー
    #[error("Invalid email format")]
    InvalidEmail,
    
    #[error("Word text must be between 1 and 100 characters")]
    InvalidWordLength,
    
    // ビジネスルール違反
    #[error("Cannot add more than 10 meanings to a word")]
    TooManyMeanings,
    
    #[error("Session is not in progress")]
    SessionNotInProgress,
    
    // リソース関連
    #[error("Word not found: {0}")]
    WordNotFound(WordId),
    
    // 同時実行制御
    #[error("Optimistic lock failed: entity was modified")]
    OptimisticLockError,
}

// リポジトリエラー
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Entity not found")]
    NotFound,
    
    #[error("Duplicate key: {0}")]
    DuplicateKey(String),
}
```

## テストの実装

### ドメインモデルのテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_word_creation_with_valid_data() {
        // Given
        let text = "sustainable".to_string();
        let meaning = "持続可能な".to_string();
        let user_id = UserId::new();
        
        // When
        let result = Word::create(text.clone(), meaning.clone(), user_id);
        
        // Then
        assert!(result.is_ok());
        let word = result.unwrap();
        assert_eq!(word.text().as_str(), "sustainable");
        assert_eq!(word.meanings().len(), 1);
    }
    
    #[test]
    fn test_adding_too_many_meanings_fails() {
        // Given
        let mut word = create_test_word();
        let user_id = UserId::new();
        
        // When: 10個まで追加
        for i in 0..9 {
            let result = word.add_meaning(format!("meaning{}", i), user_id);
            assert!(result.is_ok());
        }
        
        // Then: 11個目は失敗
        let result = word.add_meaning("one more".to_string(), user_id);
        assert!(matches!(result, Err(DomainError::TooManyMeanings)));
    }
}

// モックの使用
#[cfg(test)]
mod service_tests {
    use super::*;
    use mockall::mock;
    
    mock! {
        WordRepo {}
        
        #[async_trait]
        impl WordRepository for WordRepo {
            async fn find_by_id(&self, id: WordId) -> Result<Option<Word>, RepositoryError>;
            async fn save(&self, word: &Word) -> Result<(), RepositoryError>;
            async fn delete(&self, id: WordId) -> Result<(), RepositoryError>;
            async fn find_by_text(&self, text: &str) -> Result<Option<Word>, RepositoryError>;
            async fn find_by_category(
                &self,
                category: Category,
                limit: usize,
            ) -> Result<Vec<Word>, RepositoryError>;
        }
    }
    
    #[tokio::test]
    async fn test_word_selection_service() {
        // Given
        let mut mock_repo = MockWordRepo::new();
        mock_repo
            .expect_find_by_category()
            .with(eq(Category::IELTS), eq(20))
            .returning(|_, _| Ok(vec![create_test_word()]));
        
        let service = SmartWordSelectionService {
            word_repository: Arc::new(mock_repo),
            progress_repository: Arc::new(MockProgressRepo::new()),
        };
        
        // When
        let result = service
            .select_words_for_session(
                UserId::new(),
                SelectionCriteria::default(),
            )
            .await;
        
        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}
```

## 実装のベストプラクティス

### 1. 早期リターンでネストを減らす

```rust
// Good
pub fn process(&mut self) -> Result<(), DomainError> {
    if !self.is_valid() {
        return Err(DomainError::InvalidState);
    }
    
    // メイン処理
    Ok(())
}

// Bad
pub fn process(&mut self) -> Result<(), DomainError> {
    if self.is_valid() {
        // メイン処理
        Ok(())
    } else {
        Err(DomainError::InvalidState)
    }
}
```

### 2. Builder パターンの活用

```rust
pub struct SessionConfigBuilder {
    mode: Option<LearningMode>,
    word_count: Option<u32>,
    time_limit: Option<Duration>,
}

impl SessionConfigBuilder {
    pub fn new() -> Self {
        Self {
            mode: None,
            word_count: None,
            time_limit: None,
        }
    }
    
    pub fn mode(mut self, mode: LearningMode) -> Self {
        self.mode = Some(mode);
        self
    }
    
    pub fn word_count(mut self, count: u32) -> Self {
        self.word_count = Some(count);
        self
    }
    
    pub fn build(self) -> Result<SessionConfig, DomainError> {
        Ok(SessionConfig {
            mode: self.mode.ok_or(DomainError::MissingMode)?,
            word_count: self.word_count.unwrap_or(20),
            time_limit: self.time_limit,
        })
    }
}
```

### 3. カスタムイテレータ

```rust
pub struct WordIterator<'a> {
    words: &'a [Word],
    index: usize,
    filter: Box<dyn Fn(&Word) -> bool>,
}

impl<'a> Iterator for WordIterator<'a> {
    type Item = &'a Word;
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.words.len() {
            let word = &self.words[self.index];
            self.index += 1;
            
            if (self.filter)(word) {
                return Some(word);
            }
        }
        None
    }
}
```

## 更新履歴

- 2025-07-25: 初版作成
