# Learning Context - リポジトリ設計

## 概要

Learning Context は CQRS パターンを採用しており、Command 側と Query 側で異なるリポジトリを使用します。

## Command 側のリポジトリ

### EventStoreLearningRepository

Event Sourcing パターンに基づき、集約の状態をイベントとして保存します。

**主要な責務**:

- 集約のイベントストリームの読み込み
- 新しいイベントの追記
- スナップショットの管理
- 楽観的ロックの実装

**インターフェース**:

```rust
trait EventStoreRepository {
    // セッションの読み込み
    async fn find_session_by_id(&self, id: SessionId) -> Result<Option<LearningSession>>;
    async fn find_active_session_by_user(&self, user_id: UserId) -> Result<Option<LearningSession>>;
    
    // 学習記録の読み込み
    async fn find_record_by_user_and_item(&self, user_id: UserId, item_id: ItemId) -> Result<Option<UserItemRecord>>;
    
    // イベントの保存
    async fn save_session(&self, session: &mut LearningSession) -> Result<()>;
    async fn save_record(&self, record: &mut UserItemRecord) -> Result<()>;
    
    // スナップショット
    async fn save_snapshot(&self, aggregate_id: Uuid, snapshot: &[u8]) -> Result<()>;
    async fn load_from_snapshot(&self, aggregate_id: Uuid) -> Result<Option<Vec<u8>>>;
}
```

**実装の詳細**:

1. **イベントストリームの読み込み**
   - スナップショットがあれば、そこから開始
   - スナップショット以降のイベントを適用
   - 集約の現在の状態を再構築

2. **イベントの保存**
   - 集約から発生したイベントを取得
   - Event Store に追記
   - Event Bus にイベントを発行

3. **検索用の補助テーブル**
   - `active_sessions` テーブルで user_id → session_id のマッピング
   - `user_item_records` テーブルで (user_id, item_id) → aggregate_id のマッピング

## Query 側のリポジトリ

### ReadModelRepository

非正規化されたビューから高速に読み取ります。

**主要な責務**:

- セッション情報の高速取得
- 学習記録の検索・フィルタリング
- 統計情報の提供
- 分析用データの出力

**インターフェース**:

```rust
trait ReadModelRepository {
    // セッション情報
    async fn get_active_session(&self, user_id: Uuid) -> Result<Option<ActiveSessionView>>;
    async fn get_session(&self, session_id: Uuid) -> Result<Option<SessionDetailView>>;
    async fn get_session_history(&self, query: SessionHistoryQuery) -> Result<SessionHistoryView>;
    
    // 学習記録
    async fn get_user_item_records(&self, user_id: Uuid, item_ids: &[Uuid]) -> Result<Vec<UserItemRecordView>>;
    async fn get_mastery_status_counts(&self, user_id: Uuid) -> Result<MasteryStatusCounts>;
    async fn get_due_for_review(&self, user_id: Uuid, limit: u32) -> Result<Vec<UserItemRecordView>>;
    
    // 進捗・統計
    async fn get_learning_progress(&self, user_id: Uuid, period: ProgressPeriod) -> Result<LearningProgressView>;
    async fn get_learning_stats(&self, user_id: Uuid) -> Result<LearningStatsView>;
}
```

**データ構造**:

```rust
// 非正規化されたセッションビュー
struct SessionView {
    session_id: Uuid,
    user_id: Uuid,
    started_at: DateTime<Utc>,
    session_data: serde_json::Value,  // JSONB
    summary: serde_json::Value,       // 事前計算された統計
    status: String,
}

// 非正規化された学習記録ビュー
struct LearningRecordView {
    user_id: Uuid,
    item_id: Uuid,
    mastery_status: String,
    mastery_data: serde_json::Value,  // JSONB（統計、履歴など）
    last_reviewed: DateTime<Utc>,
    next_review: Option<DateTime<Utc>>,
}
```

## Projection 側のリポジトリ

### ProjectionRepository

イベントから Read Model への投影を管理します。

**主要な責務**:

- イベントハンドリング
- Read Model の更新
- 投影状態の管理
- 習熟度計算

**インターフェース**:

```rust
trait ProjectionRepository {
    // Read Model の更新
    async fn apply_session_started(&self, event: SessionStarted) -> Result<()>;
    async fn apply_item_presented(&self, event: ItemPresented) -> Result<()>;
    async fn apply_correctness_judged(&self, event: CorrectnessJudged) -> Result<()>;
    async fn apply_session_completed(&self, event: SessionCompleted) -> Result<()>;
    
    // 習熟度計算
    async fn calculate_mastery_transition(&self, event: &CorrectnessJudged) -> Result<Option<MasteryTransition>>;
    async fn update_mastery_status(&self, transition: MasteryTransition) -> Result<()>;
    
    // 投影状態の管理
    async fn get_projection_state(&self) -> Result<ProjectionState>;
    async fn update_projection_state(&self, state: ProjectionState) -> Result<()>;
}
```

## 実装上の考慮事項

### 1. トランザクション境界

**Command 側**:

- Event Store への書き込みと Event Bus への発行は同一トランザクション
- 失敗時は全体をロールバック

**Query 側**:

- 読み取り専用のため、トランザクションは最小限
- キャッシュとの整合性を考慮

**Projection 側**:

- イベント単位でトランザクション
- 冪等性を保証

### 2. 楽観的ロック

Event Store でのバージョン管理：

```sql
-- イベント保存時のチェック
INSERT INTO events (stream_id, event_version, ...)
VALUES (?, ?, ...)
ON CONFLICT (stream_id, event_version) DO NOTHING;
```

### 3. キャッシング戦略

**Query Service**:

- Redis で Read Model をキャッシュ
- TTL:
  - アクティブセッション: キャッシュなし
  - セッション詳細: 5分
  - 統計データ: 1時間

**Command Service**:

- キャッシュは使用しない（整合性重視）

### 4. パフォーマンス最適化

**Event Store**:

- aggregate_id にインデックス
- occurred_at にインデックス（時系列クエリ用）

**Read Model**:

- user_id, started_at に複合インデックス
- mastery_status にインデックス
- JSONB の GIN インデックス（検索用）

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

CQRS パターンにより：

- **Write 側**: イベントの完全性と監査証跡を保証
- **Read 側**: 高速な読み取りと柔軟な検索
- **Projection**: 非同期での最終的一貫性
- **Analytics**: 分析用に最適化されたデータモデル

各リポジトリは単一責任原則に従い、それぞれの役割に特化した実装となっています。
