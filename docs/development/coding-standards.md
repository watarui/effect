# コーディング規約

## Rust スタイルガイド

### 基本原則

1. **公式スタイルガイドに従う**: `rustfmt` の設定を使用
2. **明示的なエラーハンドリング**: `Result` 型を活用
3. **所有権の明確化**: 借用と所有権を適切に使い分け
4. **ゼロコスト抽象化**: パフォーマンスを意識した設計

### 命名規則

```rust
// モジュール: snake_case
mod event_store;

// 型: PascalCase
struct WordAggregate;
trait EventHandler;
enum LearningMode;

// 関数・メソッド: snake_case
fn calculate_next_interval() {}

// 定数: SCREAMING_SNAKE_CASE
const MAX_RETRY_COUNT: u32 = 3;

// 変数: snake_case
let word_count = 10;
```

### プロジェクト構造

```
service_name/
├── src/
│   ├── main.rs           # エントリーポイント
│   ├── config.rs         # 設定
│   ├── domain/           # ドメイン層
│   │   ├── mod.rs
│   │   ├── aggregates/
│   │   ├── events/
│   │   └── value_objects/
│   ├── application/      # アプリケーション層
│   │   ├── mod.rs
│   │   ├── commands/
│   │   └── queries/
│   ├── infrastructure/   # インフラ層
│   │   ├── mod.rs
│   │   ├── repositories/
│   │   └── adapters/
│   └── ports/           # ポート定義
│       ├── mod.rs
│       ├── primary/     # 入力ポート
│       └── secondary/   # 出力ポート
```

## エラーハンドリング

### カスタムエラー型

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Word not found: {id}")]
    WordNotFound { id: Uuid },

    #[error("Invalid word: {reason}")]
    InvalidWord { reason: String },

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

// Result 型エイリアス
pub type Result<T> = std::result::Result<T, DomainError>;
```

### エラー処理パターン

```rust
// 早期リターン
pub async fn get_word(&self, id: Uuid) -> Result<Word> {
    let word = self.repository
        .find_by_id(id)
        .await?
        .ok_or(DomainError::WordNotFound { id })?;

    Ok(word)
}

// エラー変換
pub async fn create_word(&self, input: CreateWordInput) -> Result<Word> {
    let word = Word::new(input)
        .map_err(|e| DomainError::InvalidWord {
            reason: e.to_string()
        })?;

    self.repository.save(&word).await?;
    Ok(word)
}
```

## 非同期プログラミング

### async/await の使用

```rust
// 非同期関数
pub async fn process_event(&self, event: DomainEvent) -> Result<()> {
    match event {
        DomainEvent::WordCreated(e) => {
            self.handle_word_created(e).await?;
        }
        // ...
    }
    Ok(())
}

// 並行処理
use futures::future::join_all;

pub async fn process_batch(&self, events: Vec<DomainEvent>) -> Vec<Result<()>> {
    let futures = events.into_iter()
        .map(|event| self.process_event(event));

    join_all(futures).await
}
```

### タイムアウト処理

```rust
use tokio::time::{timeout, Duration};

pub async fn with_timeout<T>(
    duration: Duration,
    future: impl Future<Output = T>,
) -> Result<T> {
    timeout(duration, future)
        .await
        .map_err(|_| DomainError::Timeout)
}
```

## テスト

### 単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_create_word() {
        // Arrange
        let mut mock_repo = MockWordRepository::new();
        mock_repo
            .expect_save()
            .with(predicate::always())
            .times(1)
            .returning(|_| Ok(()));

        let service = WordService::new(Arc::new(mock_repo));

        // Act
        let input = CreateWordInput {
            text: "test".to_string(),
            meaning: "テスト".to_string(),
            difficulty: 3,
        };
        let result = service.create_word(input).await;

        // Assert
        assert!(result.is_ok());
        let word = result.unwrap();
        assert_eq!(word.text, "test");
    }
}
```

### 統合テスト

```rust
// tests/integration/word_service_test.rs
use effect::test_helpers::*;

#[tokio::test]
async fn test_word_lifecycle() {
    let app = TestApp::spawn().await;

    // 単語作成
    let word = app.create_word("test", "テスト").await;
    assert_eq!(word.text, "test");

    // 単語取得
    let fetched = app.get_word(word.id).await;
    assert_eq!(fetched.id, word.id);
}
```

## ドキュメンテーション

### モジュールドキュメント

```rust
//! # Word Domain Module
//!
//! This module contains the core domain logic for word management.
//!
//! ## Example
//!
//! ```rust
//! use effect::domain::word::Word;
//!
//! let word = Word::new("example", "例").unwrap();
//! assert_eq!(word.text(), "example");
//! ```

/// Represents a word in the learning system.
///
/// # Fields
///
/// * `id` - Unique identifier
/// * `text` - The word itself
/// * `meaning` - Translation or definition
pub struct Word {
    // ...
}
```

## パフォーマンス最適化

### 文字列処理

```rust
// 避ける: 頻繁な String 生成
fn bad_concat(words: &[&str]) -> String {
    let mut result = String::new();
    for word in words {
        result = result + word + " ";  // 新しい String を生成
    }
    result
}

// 推奨: 事前確保と push_str
fn good_concat(words: &[&str]) -> String {
    let capacity = words.iter().map(|w| w.len() + 1).sum();
    let mut result = String::with_capacity(capacity);
    for word in words {
        result.push_str(word);
        result.push(' ');
    }
    result
}
```

### Clone の最小化

```rust
// 避ける: 不要な clone
fn process_word(word: Word) -> Result<()> {
    let word_copy = word.clone();  // 不要
    save_word(word_copy)?;
    Ok(())
}

// 推奨: 参照を使用
fn process_word(word: &Word) -> Result<()> {
    save_word(word)?;
    Ok(())
}
```

## セキュリティ

### 入力検証

```rust
use validator::Validate;

#[derive(Debug, Validate)]
pub struct CreateWordInput {
    #[validate(length(min = 1, max = 100))]
    pub text: String,

    #[validate(length(min = 1, max = 500))]
    pub meaning: String,

    #[validate(range(min = 1, max = 5))]
    pub difficulty: u8,
}

impl CreateWordInput {
    pub fn validate_and_sanitize(self) -> Result<Self> {
        self.validate()?;

        Ok(Self {
            text: self.text.trim().to_lowercase(),
            meaning: self.meaning.trim().to_string(),
            difficulty: self.difficulty,
        })
    }
}
```

### SQL インジェクション対策

```rust
// 常にパラメータ化クエリを使用
sqlx::query!(
    "INSERT INTO words (id, text, meaning) VALUES ($1, $2, $3)",
    word.id,
    word.text,
    word.meaning
)
.execute(&pool)
.await?;
```
