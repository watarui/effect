# テスト戦略

## 概要

このドキュメントでは、DDD に基づいた Effect プロジェクトのテスト戦略を定義します。
ドメインモデル、アグリゲート、ドメインサービスのテスト方法を具体的に説明します。

## テストピラミッド

```
         E2E Tests
        /    5%    \
       /           \
      / Integration \
     /    Tests     \
    /      20%      \
   /                 \
  /   Unit Tests     \
 /       75%         \
/___________________\
```

## 1. ドメインモデルのユニットテスト

### 値オブジェクトのテスト

```rust
#[cfg(test)]
mod email_tests {
    use super::*;
    
    #[test]
    fn test_valid_email_creation() {
        // Given
        let email_str = "user@example.com";
        
        // When
        let result = Email::new(email_str.to_string());
        
        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "user@example.com");
    }
    
    #[test]
    fn test_email_normalization() {
        // Given
        let email_str = " USER@EXAMPLE.COM ";
        
        // When
        let result = Email::new(email_str.to_string());
        
        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "user@example.com");
    }
    
    #[test]
    fn test_invalid_email_rejected() {
        // Given
        let invalid_emails = vec![
            "",
            "not-an-email",
            "@example.com",
            "user@",
            "user @example.com",
        ];
        
        // When & Then
        for email in invalid_emails {
            let result = Email::new(email.to_string());
            assert!(
                result.is_err(),
                "Email '{}' should be invalid",
                email
            );
        }
    }
    
    // プロパティベーステスト
    #[test]
    fn prop_email_idempotent() {
        use proptest::prelude::*;
        
        proptest!(|(email: String)| {
            if let Ok(email1) = Email::new(email.clone()) {
                let email2 = Email::new(email1.as_str().to_string()).unwrap();
                prop_assert_eq!(email1, email2);
            }
        });
    }
}
```

### エンティティのテスト

```rust
#[cfg(test)]
mod word_tests {
    use super::*;
    
    // テストヘルパー
    fn create_test_word() -> Word {
        Word::create(
            "test".to_string(),
            "テスト".to_string(),
            UserId::new(),
        ).unwrap()
    }
    
    #[test]
    fn test_word_creation_assigns_id() {
        // When
        let word1 = create_test_word();
        let word2 = create_test_word();
        
        // Then
        assert_ne!(word1.id(), word2.id());
    }
    
    #[test]
    fn test_add_meaning_increments_version() {
        // Given
        let mut word = create_test_word();
        let initial_version = word.version();
        
        // When
        let result = word.add_meaning(
            "試験".to_string(),
            PartOfSpeech::Noun,
            UserId::new(),
        );
        
        // Then
        assert!(result.is_ok());
        assert_eq!(word.version(), initial_version + 1);
    }
    
    #[test]
    fn test_business_rule_max_meanings() {
        // Given
        let mut word = create_test_word();
        let user_id = UserId::new();
        
        // When: 最大数まで追加
        for i in 1..10 {
            let result = word.add_meaning(
                format!("meaning{}", i),
                PartOfSpeech::Noun,
                user_id,
            );
            assert!(result.is_ok());
        }
        
        // Then: 次の追加は失敗
        let result = word.add_meaning(
            "too many".to_string(),
            PartOfSpeech::Noun,
            user_id,
        );
        assert!(matches!(
            result,
            Err(DomainError::TooManyMeanings)
        ));
    }
}
```

## 2. アグリゲートのテスト

### アグリゲート不変条件のテスト

```rust
#[cfg(test)]
mod learning_session_tests {
    use super::*;
    
    #[test]
    fn test_session_invariants() {
        // Given
        let user_id = UserId::new();
        let word_ids = vec![WordId::new(); 5];
        let config = SessionConfig::default();
        
        // When
        let result = LearningSession::start(user_id, word_ids, config);
        
        // Then
        assert!(result.is_ok());
        let (session, events) = result.unwrap();
        assert!(session.is_in_progress());
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::SessionStarted { .. }
        ));
    }
    
    #[test]
    fn test_cannot_submit_answer_to_completed_session() {
        // Given
        let mut session = create_test_session();
        session.complete().unwrap();
        
        // When
        let result = session.submit_answer(
            QuestionId::new(),
            "answer".to_string(),
        );
        
        // Then
        assert!(matches!(
            result,
            Err(DomainError::SessionNotInProgress)
        ));
    }
}
```

### イベント生成のテスト

```rust
#[test]
fn test_session_events_sequence() {
    // Given
    let mut session = create_test_session();
    let question_id = session.generate_question(WordId::new()).unwrap();
    
    // When
    let events = session.submit_answer(
        question_id,
        "correct answer".to_string(),
    ).unwrap();
    
    // Then
    assert_eq!(events.len(), 1);
    match &events[0] {
        DomainEvent::AnswerSubmitted {
            session_id,
            question_id: q_id,
            is_correct,
            ..
        } => {
            assert_eq!(*session_id, session.id());
            assert_eq!(*q_id, question_id);
            assert!(*is_correct);
        }
        _ => panic!("Unexpected event type"),
    }
}
```

## 3. ドメインサービスのテスト

### モックを使用したテスト

```rust
use mockall::automock;

#[automock]
#[async_trait]
pub trait WordRepository: Send + Sync {
    async fn find_by_id(&self, id: WordId) -> Result<Option<Word>, RepositoryError>;
    async fn find_by_category(
        &self,
        category: Category,
        limit: usize,
    ) -> Result<Vec<Word>, RepositoryError>;
}

#[cfg(test)]
mod word_selection_service_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_word_selection_prioritizes_overdue() {
        // Given
        let mut mock_word_repo = MockWordRepository::new();
        let mut mock_progress_repo = MockUserProgressRepository::new();
        
        // 期限切れの単語
        let overdue_words = vec![WordId::new(); 10];
        mock_progress_repo
            .expect_find_overdue_words()
            .returning(move |_, _| Ok(overdue_words.clone()));
        
        // 新規単語
        let new_words = vec![WordId::new(); 10];
        mock_word_repo
            .expect_find_unlearned_words()
            .returning(move |_, _| Ok(new_words.clone()));
        
        let service = SmartWordSelectionService {
            word_repository: Arc::new(mock_word_repo),
            progress_repository: Arc::new(mock_progress_repo),
        };
        
        // When
        let result = service
            .select_words_for_session(
                UserId::new(),
                20,
                SelectionCriteria::default(),
            )
            .await;
        
        // Then
        assert!(result.is_ok());
        let selected = result.unwrap();
        assert_eq!(selected.len(), 20);
        // 最初の10個は期限切れの単語
        assert_eq!(&selected[..10], &overdue_words[..]);
    }
}
```

### テストダブルの使用

```rust
// フェイク実装
pub struct FakeWordRepository {
    words: HashMap<WordId, Word>,
}

impl FakeWordRepository {
    pub fn new() -> Self {
        Self {
            words: HashMap::new(),
        }
    }
    
    pub fn with_words(words: Vec<Word>) -> Self {
        let mut repo = Self::new();
        for word in words {
            repo.words.insert(word.id(), word);
        }
        repo
    }
}

#[async_trait]
impl WordRepository for FakeWordRepository {
    async fn find_by_id(&self, id: WordId) -> Result<Option<Word>, RepositoryError> {
        Ok(self.words.get(&id).cloned())
    }
    
    async fn save(&self, word: &Word) -> Result<(), RepositoryError> {
        // In-memory implementation
        Ok(())
    }
}
```

## 4. 統合テスト

### リポジトリの統合テスト

```rust
#[cfg(test)]
mod repository_integration_tests {
    use super::*;
    use sqlx::PgPool;
    
    #[sqlx::test]
    async fn test_word_repository_crud(pool: PgPool) {
        // Given
        let repo = PostgresWordRepository::new(pool);
        let word = create_test_word();
        
        // When: Save
        let save_result = repo.save(&word).await;
        assert!(save_result.is_ok());
        
        // When: Find
        let find_result = repo.find_by_id(word.id()).await;
        assert!(find_result.is_ok());
        let found = find_result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), word.id());
        
        // When: Delete
        let delete_result = repo.delete(word.id()).await;
        assert!(delete_result.is_ok());
        
        // Then: Not found
        let not_found = repo.find_by_id(word.id()).await.unwrap();
        assert!(not_found.is_none());
    }
    
    #[sqlx::test]
    async fn test_optimistic_locking(pool: PgPool) {
        // Given
        let repo = PostgresWordRepository::new(pool);
        let word = create_test_word();
        repo.save(&word).await.unwrap();
        
        // When: 同時更新をシミュレート
        let mut word1 = repo.find_by_id(word.id()).await.unwrap().unwrap();
        let mut word2 = repo.find_by_id(word.id()).await.unwrap().unwrap();
        
        word1.add_meaning("meaning1".to_string(), UserId::new()).unwrap();
        word2.add_meaning("meaning2".to_string(), UserId::new()).unwrap();
        
        // Then: 最初の更新は成功
        assert!(repo.save(&word1).await.is_ok());
        
        // 2番目の更新は失敗
        let result = repo.save(&word2).await;
        assert!(matches!(
            result,
            Err(RepositoryError::OptimisticLockError)
        ));
    }
}
```

### イベントストアの統合テスト

```rust
#[cfg(test)]
mod event_store_integration_tests {
    use super::*;
    
    #[sqlx::test]
    async fn test_event_store_append_and_load(pool: PgPool) {
        // Given
        let event_store = PostgresEventStore::new(pool);
        let aggregate_id = SessionId::new().to_string();
        
        let events = vec![
            DomainEvent::SessionStarted {
                session_id: SessionId::new(),
                user_id: UserId::new(),
                word_count: 20,
                started_at: Utc::now(),
            },
        ];
        
        // When: Append
        let result = event_store
            .append_events(&aggregate_id, 0, events.clone())
            .await;
        assert!(result.is_ok());
        
        // When: Load
        let loaded = event_store
            .load_events(&aggregate_id)
            .await
            .unwrap();
        
        // Then
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].event, events[0]);
    }
}
```

## 5. E2E テスト

### GraphQL API テスト

```rust
#[cfg(test)]
mod graphql_e2e_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_complete_learning_flow() {
        // Given: テスト環境のセットアップ
        let app = test_helpers::create_test_app().await;
        let client = test_helpers::create_test_client(&app);
        let user = test_helpers::create_test_user().await;
        
        // When: ログイン
        let login_response = client
            .post("/graphql")
            .json(&json!({
                "query": r#"
                    mutation Login($email: String!, $password: String!) {
                        login(email: $email, password: $password) {
                            token
                            user { id email }
                        }
                    }
                "#,
                "variables": {
                    "email": user.email,
                    "password": "testpass123"
                }
            }))
            .send()
            .await
            .unwrap();
        
        let token = login_response
            .json::<serde_json::Value>()
            .await
            .unwrap()["data"]["login"]["token"]
            .as_str()
            .unwrap();
        
        // When: セッション開始
        let start_session_response = client
            .post("/graphql")
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "query": r#"
                    mutation StartSession($mode: LearningMode!) {
                        startLearningSession(mode: $mode) {
                            id
                            questions {
                                id
                                wordId
                                questionText
                                options
                            }
                        }
                    }
                "#,
                "variables": {
                    "mode": "MULTIPLE_CHOICE"
                }
            }))
            .send()
            .await
            .unwrap();
        
        // Then: セッションが作成される
        assert_eq!(start_session_response.status(), 200);
        let session_data = start_session_response
            .json::<serde_json::Value>()
            .await
            .unwrap();
        assert!(session_data["data"]["startLearningSession"]["id"].is_string());
    }
}
```

## 6. テストユーティリティ

### テストビルダー

```rust
pub struct WordBuilder {
    text: String,
    meanings: Vec<(String, PartOfSpeech)>,
    phonetic: Option<String>,
    difficulty: u8,
}

impl WordBuilder {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            meanings: vec![],
            phonetic: None,
            difficulty: 5,
        }
    }
    
    pub fn with_meaning(mut self, text: impl Into<String>, pos: PartOfSpeech) -> Self {
        self.meanings.push((text.into(), pos));
        self
    }
    
    pub fn with_phonetic(mut self, phonetic: impl Into<String>) -> Self {
        self.phonetic = Some(phonetic.into());
        self
    }
    
    pub fn with_difficulty(mut self, difficulty: u8) -> Self {
        self.difficulty = difficulty;
        self
    }
    
    pub fn build(self) -> Word {
        let mut word = Word::create(
            self.text,
            self.meanings[0].0.clone(),
            UserId::new(),
        ).unwrap();
        
        // 追加の意味を設定
        for (text, pos) in self.meanings.into_iter().skip(1) {
            word.add_meaning(text, pos, UserId::new()).unwrap();
        }
        
        if let Some(phonetic) = self.phonetic {
            word.set_phonetic(phonetic).unwrap();
        }
        
        word.set_difficulty(self.difficulty).unwrap();
        
        word
    }
}

// 使用例
#[test]
fn test_with_builder() {
    let word = WordBuilder::new("test")
        .with_meaning("テスト", PartOfSpeech::Noun)
        .with_meaning("試験", PartOfSpeech::Noun)
        .with_phonetic("test")
        .with_difficulty(3)
        .build();
    
    assert_eq!(word.text().as_str(), "test");
    assert_eq!(word.meanings().len(), 2);
}
```

### アサーションヘルパー

```rust
pub mod assertions {
    use super::*;
    
    pub fn assert_domain_event_type<T: Into<DomainEvent>>(
        events: &[DomainEvent],
        expected_type: &str,
    ) {
        assert!(
            events.iter().any(|e| e.event_type() == expected_type),
            "Expected event type '{}' not found in {:?}",
            expected_type,
            events.iter().map(|e| e.event_type()).collect::<Vec<_>>()
        );
    }
    
    pub fn assert_aggregate_version(aggregate: &impl Versioned, expected: u32) {
        assert_eq!(
            aggregate.version(),
            expected,
            "Expected version {} but got {}",
            expected,
            aggregate.version()
        );
    }
}
```

## 7. テストデータ管理

### フィクスチャ

```rust
pub mod fixtures {
    use super::*;
    use once_cell::sync::Lazy;
    
    pub static IELTS_WORDS: Lazy<Vec<Word>> = Lazy::new(|| {
        vec![
            WordBuilder::new("sustainable")
                .with_meaning("持続可能な", PartOfSpeech::Adjective)
                .with_difficulty(7)
                .build(),
            WordBuilder::new("infrastructure")
                .with_meaning("インフラ", PartOfSpeech::Noun)
                .with_difficulty(8)
                .build(),
            // ... more test data
        ]
    });
    
    pub fn create_test_user_progress() -> UserProgress {
        UserProgress::new(
            UserId::new(),
            WordId::new(),
            SM2Parameters::default(),
        )
    }
}
```

### データベースシード

```rust
pub async fn seed_test_database(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    // トランザクション内で実行
    let mut tx = pool.begin().await?;
    
    // テストユーザーの作成
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, display_name, password_hash)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO NOTHING
        "#,
        TEST_USER_ID,
        "test@example.com",
        "Test User",
        hash_password("testpass123")?
    )
    .execute(&mut tx)
    .await?;
    
    // テスト単語の作成
    for word in &*fixtures::IELTS_WORDS {
        // ... insert word data
    }
    
    tx.commit().await?;
    Ok(())
}
```

## 8. テストのベストプラクティス

### 1. Arrange-Act-Assert パターン

```rust
#[test]
fn test_example() {
    // Arrange (Given)
    let word = create_test_word();
    let user_id = UserId::new();
    
    // Act (When)
    let result = word.add_meaning("new meaning", user_id);
    
    // Assert (Then)
    assert!(result.is_ok());
}
```

### 2. 1つのテストで1つの概念

```rust
// Good: 各テストが1つの概念をテスト
#[test]
fn test_email_normalization() { /* ... */ }

#[test]
fn test_email_validation() { /* ... */ }

// Bad: 複数の概念を1つのテストで
#[test]
fn test_email() { /* normalization と validation を両方テスト */ }
```

### 3. テスト名は仕様を表現

```rust
#[test]
fn word_with_ten_meanings_cannot_accept_more() { /* ... */ }

#[test]
fn session_in_completed_state_rejects_new_answers() { /* ... */ }
```

## 更新履歴

- 2025-07-25: 初版作成
