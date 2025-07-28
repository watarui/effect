# Learning Context - リポジトリインターフェース

## 概要

Learning Context には 2 つの集約が存在し、それぞれに対応するリポジトリを定義します：

- `LearningSession`：学習セッションの管理
- `UserItemRecord`：ユーザーの項目別学習記録

## LearningSessionRepository

学習セッションの永続化を担当するリポジトリです。

### インターフェース定義

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 学習セッションのリポジトリ
#[async_trait]
pub trait LearningSessionRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    // ===== 基本的なCRUD操作 =====

    /// IDでセッションを取得
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<LearningSession>, Self::Error>;

    /// セッションを保存（新規作成または更新）
    async fn save(&self, session: &LearningSession) -> Result<(), Self::Error>;

    /// セッションを削除（通常は使用しない）
    async fn delete(&self, id: &SessionId) -> Result<(), Self::Error>;

    // ===== ユーザー関連のクエリ =====

    /// ユーザーのアクティブなセッションを取得
    /// （InProgressステータスのセッション）
    async fn find_active_by_user(
        &self,
        user_id: &UserId
    ) -> Result<Option<LearningSession>, Self::Error>;

    /// ユーザーの全セッションをページネーションで取得
    async fn find_by_user_paginated(
        &self,
        user_id: &UserId,
        page_request: &PageRequest,
    ) -> Result<Page<LearningSession>, Self::Error>;

    /// ユーザーの特定期間のセッションを取得
    async fn find_by_user_and_date_range(
        &self,
        user_id: &UserId,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<LearningSession>, Self::Error>;

    // ===== 統計関連のクエリ =====

    /// ユーザーの完了セッション数を取得
    async fn count_completed_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<u64, Self::Error>;

    /// ユーザーの本日の完了セッション数を取得
    async fn count_completed_today_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<u64, Self::Error>;

    // ===== 管理用クエリ =====

    /// 長時間放置されたセッションを取得
    /// （タイムアウト処理用）
    async fn find_stale_sessions(
        &self,
        older_than: DateTime<Utc>,
    ) -> Result<Vec<LearningSession>, Self::Error>;
}
```

### 使用例

```rust
// アプリケーションサービスでの使用例
pub struct StartLearningSessionUseCase<R: LearningSessionRepository> {
    repository: Arc<R>,
}

impl<R: LearningSessionRepository> StartLearningSessionUseCase<R> {
    pub async fn execute(&self, user_id: UserId, items: Vec<ItemId>) -> Result<SessionId> {
        // アクティブなセッションがないことを確認
        if let Some(_) = self.repository.find_active_by_user(&user_id).await? {
            return Err(DomainError::ActiveSessionExists);
        }

        // 新しいセッションを作成
        let session = LearningSession::new(user_id, items)?;
        let session_id = session.id().clone();

        // 保存
        self.repository.save(&session).await?;

        Ok(session_id)
    }
}
```

## UserItemRecordRepository

ユーザーの項目別学習記録を管理するリポジトリです。

### インターフェース定義

```rust
/// ユーザー項目記録のリポジトリ
#[async_trait]
pub trait UserItemRecordRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    // ===== 基本的なCRUD操作 =====

    /// ユーザーと項目の組み合わせで記録を取得
    async fn find_by_user_and_item(
        &self,
        user_id: &UserId,
        item_id: &ItemId,
    ) -> Result<Option<UserItemRecord>, Self::Error>;

    /// 記録を保存（新規作成または更新）
    async fn save(&self, record: &UserItemRecord) -> Result<(), Self::Error>;

    /// 複数の記録を一括保存（バッチ処理用）
    async fn save_batch(&self, records: &[UserItemRecord]) -> Result<(), Self::Error>;

    // ===== 学習状態のクエリ =====

    /// ユーザーの未学習項目を取得
    async fn find_unlearned_by_user(
        &self,
        user_id: &UserId,
        limit: u32,
    ) -> Result<Vec<UserItemRecord>, Self::Error>;

    /// ユーザーの復習が必要な項目を取得
    async fn find_due_for_review_by_user(
        &self,
        user_id: &UserId,
        as_of: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<UserItemRecord>, Self::Error>;

    /// ユーザーのマスター済み項目を取得
    async fn find_mastered_by_user(
        &self,
        user_id: &UserId,
        mastery_type: Option<MasteryType>,
    ) -> Result<Vec<UserItemRecord>, Self::Error>;

    // ===== カテゴリ別クエリ =====

    /// カテゴリ別の学習進捗を取得
    async fn find_by_user_and_category(
        &self,
        user_id: &UserId,
        category: &Category,
        page_request: &PageRequest,
    ) -> Result<Page<UserItemRecord>, Self::Error>;

    // ===== 統計用クエリ =====

    /// ユーザーの学習統計サマリーを取得
    async fn get_user_statistics(
        &self,
        user_id: &UserId,
    ) -> Result<UserLearningStatistics, Self::Error>;

    /// マスタリーレベル別の項目数を取得
    async fn count_by_mastery_level(
        &self,
        user_id: &UserId,
    ) -> Result<HashMap<MasteryLevel, u64>, Self::Error>;

    // ===== バルク操作 =====

    /// ユーザーの全記録を削除（アカウント削除時）
    async fn delete_all_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<u64, Self::Error>;
}
```

### 補助的な型定義

```rust
/// マスタリータイプ
#[derive(Debug, Clone, Copy)]
pub enum MasteryType {
    ShortTerm,
    LongTerm,
}

/// ユーザー学習統計
#[derive(Debug)]
pub struct UserLearningStatistics {
    pub total_items: u64,
    pub unlearned_count: u64,
    pub learning_count: u64,
    pub short_term_mastered_count: u64,
    pub long_term_mastered_count: u64,
    pub average_accuracy: f64,
    pub total_study_time_minutes: u64,
}

/// カテゴリ（CEFR レベルやスキルタイプ）
#[derive(Debug, Clone)]
pub enum Category {
    CefrLevel(CefrLevel),
    SkillType(SkillType),
    Custom(String),
}
```

## 実装上の考慮事項

### 1. パフォーマンス最適化

```rust
// インデックスの推奨
// LearningSession
// - (user_id, status) - アクティブセッション検索用
// - (user_id, created_at) - 期間検索用
// - (status, updated_at) - タイムアウト処理用

// UserItemRecord
// - (user_id, item_id) - プライマリキー相当
// - (user_id, mastery_state) - 状態別検索用
// - (user_id, next_review_date) - 復習項目検索用
```

### 2. トランザクション境界

```rust
// セッション完了時の処理例
pub async fn complete_session(
    session_repo: &dyn LearningSessionRepository,
    record_repo: &dyn UserItemRecordRepository,
    session_id: SessionId,
) -> Result<()> {
    // 1. セッションを更新（1つのトランザクション）
    let mut session = session_repo.find_by_id(&session_id).await?
        .ok_or(DomainError::NotFound)?;
    session.complete();
    session_repo.save(&session).await?;

    // 2. UserItemRecordの更新は別トランザクション
    // （イベント駆動で非同期に処理するのが理想）
    for result in session.results() {
        if let Some(mut record) = record_repo
            .find_by_user_and_item(&session.user_id(), &result.item_id)
            .await?
        {
            record.update_from_result(result);
            record_repo.save(&record).await?;
        }
    }

    Ok(())
}
```

### 3. エラーハンドリング

```rust
/// Learning Context 固有のリポジトリエラー
#[derive(Debug, thiserror::Error)]
pub enum LearningRepositoryError {
    #[error("Session not found")]
    SessionNotFound,

    #[error("Active session already exists for user")]
    ActiveSessionExists,

    #[error("Invalid session state transition")]
    InvalidStateTransition,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
```

## テスト実装の例

```rust
/// テスト用のインメモリ実装
pub struct InMemoryUserItemRecordRepository {
    records: Arc<Mutex<HashMap<(UserId, ItemId), UserItemRecord>>>,
}

impl InMemoryUserItemRecordRepository {
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserItemRecordRepository for InMemoryUserItemRecordRepository {
    type Error = LearningRepositoryError;

    async fn find_by_user_and_item(
        &self,
        user_id: &UserId,
        item_id: &ItemId,
    ) -> Result<Option<UserItemRecord>, Self::Error> {
        let records = self.records.lock().unwrap();
        Ok(records.get(&(user_id.clone(), item_id.clone())).cloned())
    }

    async fn save(&self, record: &UserItemRecord) -> Result<(), Self::Error> {
        let mut records = self.records.lock().unwrap();
        let key = (record.user_id().clone(), record.item_id().clone());
        records.insert(key, record.clone());
        Ok(())
    }

    // 他のメソッドも同様に実装...
}
```

## 更新履歴

- 2025-07-28: 初版作成（DDD 原則に基づいた設計）
