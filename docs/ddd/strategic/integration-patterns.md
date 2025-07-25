# 統合パターン

## 概要

このドキュメントでは、Effect の各境界づけられたコンテキスト間の統合パターンについて詳細に説明します。

## 統合パターン一覧

### 1. Shared Kernel（共有カーネル）

**使用箇所**: User Context ↔ 他のすべてのコンテキスト

**特徴**:

- 小さく安定した共有モデル
- 変更時は全チームの合意が必要
- 密結合だが管理可能な範囲

### 2. Published Language（公開ホスト言語）

**使用箇所**: Word Management → Learning Context

**特徴**:

- 明確に定義された API
- バージョニング対応
- ドキュメント化された仕様

### 3. Domain Events（ドメインイベント）

**使用箇所**: Learning Context → Progress Context

**特徴**:

- 非同期通信
- 疎結合
- イベントソーシング対応

### 4. Anti-Corruption Layer（腐敗防止層）

**使用箇所**: 外部システムとの統合

**特徴**:

- 外部モデルからの保護
- 変換ロジックの集約
- テスト可能性の向上

## 詳細な実装パターン

### Shared Kernel の実装

```rust
// shared/kernel/src/lib.rs
pub mod user {
    use uuid::Uuid;
    use serde::{Serialize, Deserialize};

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    pub struct UserId(pub Uuid);

    impl UserId {
        pub fn new() -> Self {
            Self(Uuid::new_v4())
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct UserInfo {
        pub id: UserId,
        pub display_name: String,
        pub email: String,
    }
}

pub mod common {
    use chrono::{DateTime, Utc};

    #[derive(Clone, Debug)]
    pub struct AuditInfo {
        pub created_by: UserId,
        pub created_at: DateTime<Utc>,
        pub updated_by: Option<UserId>,
        pub updated_at: Option<DateTime<Utc>>,
    }
}
```

**使用例**:

```rust
// 各コンテキストでの利用
use shared_kernel::user::{UserId, UserInfo};

pub struct Word {
    pub id: WordId,
    pub text: String,
    pub created_by: UserId,  // Shared Kernel
}
```

### Published Language の実装

```rust
// word-management/src/public_api.rs
use async_trait::async_trait;

/// Word Management Context が公開する API
#[async_trait]
pub trait WordServiceApi {
    /// 学習用の単語データを取得
    async fn get_words_for_learning(
        &self,
        criteria: LearningCriteria,
    ) -> Result<Vec<WordData>, WordServiceError>;

    /// 単語の詳細情報を取得
    async fn get_word_details(
        &self,
        word_id: WordId,
    ) -> Result<WordDetails, WordServiceError>;
}

/// 標準化されたリクエスト/レスポンス
#[derive(Serialize, Deserialize)]
pub struct LearningCriteria {
    pub user_id: UserId,
    pub categories: Vec<Category>,
    pub difficulty_range: (u8, u8),
    pub limit: usize,
}

#[derive(Serialize, Deserialize)]
pub struct WordData {
    pub id: WordId,
    pub text: String,
    pub phonetic: String,
    pub primary_meaning: String,
    pub difficulty: u8,
}

/// バージョニング対応
pub mod v1 {
    pub use super::{WordServiceApi, WordData, LearningCriteria};
}
```

**クライアント実装**:

```rust
// learning-context/src/infrastructure/word_service_client.rs
pub struct WordServiceClient {
    base_url: String,
    client: reqwest::Client,
}

impl WordServiceClient {
    pub async fn get_words_for_learning(
        &self,
        criteria: LearningCriteria,
    ) -> Result<Vec<WordData>> {
        let response = self.client
            .post(&format!("{}/api/v1/words/for-learning", self.base_url))
            .json(&criteria)
            .send()
            .await?;

        response.json().await
    }
}
```

### Domain Events の実装

```rust
// shared/events/src/lib.rs
use serde::{Serialize, Deserialize};

/// すべてのドメインイベントの基底トレイト
pub trait DomainEvent: Serialize + Send + Sync {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> String;
    fn occurred_at(&self) -> DateTime<Utc>;
}

/// Learning Context のイベント
#[derive(Serialize, Deserialize)]
pub enum LearningDomainEvent {
    SessionStarted {
        session_id: SessionId,
        user_id: UserId,
        word_ids: Vec<WordId>,
        started_at: DateTime<Utc>,
    },
    QuestionAnswered {
        session_id: SessionId,
        word_id: WordId,
        is_correct: bool,
        response_time_ms: u32,
        answered_at: DateTime<Utc>,
    },
    SessionCompleted {
        session_id: SessionId,
        total_questions: u32,
        correct_answers: u32,
        completed_at: DateTime<Utc>,
    },
}

impl DomainEvent for LearningDomainEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::SessionStarted { .. } => "learning.session.started",
            Self::QuestionAnswered { .. } => "learning.question.answered",
            Self::SessionCompleted { .. } => "learning.session.completed",
        }
    }
    // ...
}
```

**イベントパブリッシャー**:

```rust
// learning-context/src/infrastructure/event_publisher.rs
pub struct EventPublisher {
    bus: Arc<dyn EventBus>,
}

impl EventPublisher {
    pub async fn publish(&self, event: impl DomainEvent) -> Result<()> {
        let envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            event_type: event.event_type().to_string(),
            aggregate_id: event.aggregate_id(),
            payload: serde_json::to_value(&event)?,
            occurred_at: event.occurred_at(),
        };

        self.bus.publish(envelope).await
    }
}
```

**イベントサブスクライバー**:

```rust
// progress-context/src/infrastructure/event_handlers.rs
pub struct LearningEventHandler {
    progress_service: Arc<ProgressService>,
}

#[async_trait]
impl EventHandler for LearningEventHandler {
    async fn handle(&self, envelope: EventEnvelope) -> Result<()> {
        match envelope.event_type.as_str() {
            "learning.session.completed" => {
                let event: SessionCompleted = serde_json::from_value(envelope.payload)?;
                self.progress_service.update_statistics(event).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

### Anti-Corruption Layer の実装

```rust
// word-management/src/infrastructure/dictionary_acl.rs
pub struct DictionaryACL {
    external_client: ExternalDictionaryClient,
}

impl DictionaryACL {
    /// 外部 API のデータを内部ドメインモデルに変換
    pub async fn enrich_word_data(&self, word: &str) -> Result<EnrichedWordData> {
        // 外部 API 呼び出し
        let external_response = self.external_client
            .lookup_word(word)
            .await
            .map_err(|e| DomainError::ExternalServiceError(e.to_string()))?;

        // ドメインモデルへの変換
        Ok(EnrichedWordData {
            phonetic_ipa: self.convert_pronunciation(external_response.pronunciation),
            audio_url: external_response.audio_files.first().map(|f| f.url.clone()),
            etymology: self.convert_etymology(external_response.etymology),
        })
    }

    fn convert_pronunciation(&self, external: ExternalPronunciation) -> String {
        // 外部形式から IPA 形式への変換ロジック
        match external {
            ExternalPronunciation::Ipa(ipa) => ipa,
            ExternalPronunciation::Respelling(resp) => self.respelling_to_ipa(resp),
        }
    }
}

/// 外部サービスのモデル（変更される可能性がある）
struct ExternalDictionaryResponse {
    word: String,
    pronunciation: ExternalPronunciation,
    audio_files: Vec<AudioFile>,
    etymology: Option<String>,
}

/// 内部ドメインモデル（安定）
pub struct EnrichedWordData {
    pub phonetic_ipa: String,
    pub audio_url: Option<String>,
    pub etymology: Option<String>,
}
```

## 統合テスト戦略

### Contract Testing

```rust
#[cfg(test)]
mod contract_tests {
    use super::*;

    #[tokio::test]
    async fn word_service_contract() {
        let mock_service = MockWordService::new();

        // Published Language の契約を検証
        let criteria = LearningCriteria {
            user_id: UserId::new(),
            categories: vec![Category::IELTS],
            difficulty_range: (3, 7),
            limit: 10,
        };

        let result = mock_service.get_words_for_learning(criteria).await;

        assert!(result.is_ok());
        let words = result.unwrap();
        assert!(!words.is_empty());
        assert!(words.iter().all(|w| w.difficulty >= 3 && w.difficulty <= 7));
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn learning_to_progress_event_flow() {
    let event_bus = InMemoryEventBus::new();
    let learning_context = setup_learning_context(&event_bus);
    let progress_context = setup_progress_context(&event_bus);

    // Learning Context でセッション完了
    let session_id = learning_context
        .start_session(user_id, word_ids)
        .await
        .unwrap();

    learning_context
        .complete_session(session_id)
        .await
        .unwrap();

    // Progress Context でイベントが処理されることを確認
    tokio::time::sleep(Duration::from_millis(100)).await;

    let stats = progress_context
        .get_user_statistics(user_id)
        .await
        .unwrap();

    assert_eq!(stats.total_sessions, 1);
}
```

## マイグレーション戦略

### 現在: モノリシック

```rust
// すべてが単一プロセス内
pub struct MonolithicApp {
    word_service: WordService,
    learning_service: LearningService,
    progress_service: ProgressService,
}
```

### 移行期: モジュラーモノリス

```rust
// 論理的な分離、物理的には単一プロセス
pub mod contexts {
    pub mod word_management { /* ... */ }
    pub mod learning { /* ... */ }
    pub mod progress { /* ... */ }
}
```

### 将来: マイクロサービス

```yaml
# docker-compose.yml
services:
  word-management:
    image: effect/word-management:latest

  learning:
    image: effect/learning:latest

  progress:
    image: effect/progress:latest
```

## 更新履歴

- 2025-07-25: 初版作成
