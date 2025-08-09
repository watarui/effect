# Learning Context - リポジトリ設計

## 概要

Learning Context はシンプルなリポジトリパターンを採用し、セッション管理に特化した軽量な実装となっています。

## 主要リポジトリ

### SessionRepository

Redis を使用したセッション状態の一時管理を行います。

**主要な責務**:

- セッション状態の保存・取得
- タイムアウト管理
- アクティブセッションの追跡

**インターフェース**:

```rust
trait SessionRepository {
    // セッション管理
    async fn save_session(&self, session: &LearningSession) -> Result<()>;
    async fn find_session_by_id(&self, id: SessionId) -> Result<Option<LearningSession>>;
    async fn find_active_session_by_user(&self, user_id: UserId) -> Result<Option<LearningSession>>;
    async fn delete_session(&self, id: SessionId) -> Result<()>;
    
    // セッション状態の更新
    async fn update_session_item(&self, session_id: SessionId, item: &SessionItem) -> Result<()>;
    async fn mark_session_completed(&self, session_id: SessionId) -> Result<()>;
}
```

**実装の詳細**:

1. **Redis での状態管理**
   - セッション状態を JSON として保存
   - TTL 2時間で自動削除
   - ユーザーごとのアクティブセッション管理

2. **最小限の永続化**
   - セッション完了時のサマリーのみ PostgreSQL に保存
   - 詳細な履歴は Progress Context に委譲

### UserItemRecordRepository

最小限の学習記録を管理します。

**主要な責務**:

- 最新の学習状態のみ保持
- 正答率の簡易計算
- Progress Context への同期用データ提供

**インターフェース**:

```rust
trait UserItemRecordRepository {
    // 記録の取得・更新
    async fn find_or_create(&self, user_id: UserId, item_id: ItemId) -> Result<UserItemRecord>;
    async fn update_after_judgment(&self, record: &UserItemRecord) -> Result<()>;
    
    // 簡易検索
    async fn find_by_user(&self, user_id: UserId, limit: usize) -> Result<Vec<UserItemRecord>>;
    async fn count_by_status(&self, user_id: UserId) -> Result<HashMap<String, usize>>;
}
```

**データ構造**:

```rust
// 最小限の学習記録
struct UserItemRecord {
    user_id: Uuid,
    item_id: Uuid,
    last_seen: DateTime<Utc>,
    correct_count: u32,
    total_count: u32,
    // 詳細な履歴は Progress Context で管理
}
```

### EventPublisher

Progress Context への通知を管理します。

**主要な責務**:

- ドメインイベントの発行
- Pub/Sub への送信

**インターフェース**:

```rust
trait EventPublisher {
    // Progress Context への通知
    async fn publish_session_started(&self, event: SessionStarted) -> Result<()>;
    async fn publish_session_completed(&self, event: SessionCompleted) -> Result<()>;
    async fn publish_correctness_judged(&self, event: CorrectnessJudged) -> Result<()>;
}
```

## 実装上の考慮事項

### 1. トランザクション境界

**セッション操作**:

- Redis 操作は原子性を保証
- PostgreSQL への保存は最小限

**イベント発行**:

- Fire-and-forget パターン
- Progress Context での処理は非同期

### 2. データ整合性

Redis と PostgreSQL の整合性：

```rust
// セッション完了時の処理
async fn complete_session(session_id: SessionId) {
    // 1. Redis から削除
    redis.delete(&session_id).await?;
    
    // 2. サマリーを PostgreSQL に保存
    postgres.save_summary(&summary).await?;
    
    // 3. Progress Context へ通知
    publisher.publish_completed(event).await?;
}
```

### 3. キャッシング戦略

**セッション状態**:

- Redis がメインストレージ
- TTL 2時間で自動削除

**項目詳細**:

- Vocabulary Context から取得した結果をキャッシュ
- TTL 5分

### 4. パフォーマンス最適化

**Redis**:

- ユーザーID をキープレフィックスに使用
- パイプラインで複数操作をバッチ処理

**PostgreSQL**:

- 最小限のインデックス
- user_id, item_id の複合インデックスのみ

### 5. エラーハンドリング

```rust
// リポジトリエラーの統一的な処理
enum RepositoryError {
    NotFound,
    VersionConflict,
    DatabaseError(String),
    SerializationError(String),
}
```

## まとめ

シンプルなリポジトリパターンにより：

- **セッション管理**: Redis による高速な状態管理
- **学習記録**: 最小限の永続化
- **履歴管理**: Progress Context への委譲
- **通知**: 非同期でのイベント発行

各リポジトリは単一責任原則に従い、セッション実行に特化した軽量な実装となっています。
