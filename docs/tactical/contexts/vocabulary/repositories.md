# Vocabulary Context - リポジトリ設計

## 概要

Vocabulary Context は CQRS パターンを採用しており、Command 側と Query 側で異なるリポジトリを使用します。

## Command 側のリポジトリ

### EventStoreVocabularyRepository

Event Sourcing パターンに基づき、集約の状態をイベントとして保存します。

**主要な責務**:

- 集約のイベントストリームの読み込み
- 新しいイベントの追記
- スナップショットの管理
- 楽観的ロックの実装

**インターフェース**:

```rust
trait EventStoreRepository {
    // 集約の読み込み
    async fn find_by_spelling(&self, spelling: &str) -> Result<Option<VocabularyEntry>>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VocabularyEntry>>;
    
    // イベントの保存
    async fn save(&self, entry: &mut VocabularyEntry) -> Result<()>;
    
    // スナップショット
    async fn save_snapshot(&self, entry: &VocabularyEntry) -> Result<()>;
    async fn load_from_snapshot(&self, id: Uuid) -> Result<Option<VocabularyEntry>>;
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
   - `vocabulary_entries` テーブルで spelling → entry_id のマッピングを管理
   - Event Store での検索を効率化

## Query 側のリポジトリ

### ReadModelRepository

非正規化されたビューから高速に読み取ります。

**主要な責務**:

- 項目情報の高速取得
- 検索・フィルタリング
- 統計情報の提供

**インターフェース**:

```rust
trait ReadModelRepository {
    // 基本的な取得
    async fn get_item(&self, item_id: Uuid) -> Result<Option<VocabularyItemView>>;
    async fn get_entry(&self, entry_id: Uuid) -> Result<Option<VocabularyEntryView>>;
    
    // 検索
    async fn search_items(&self, query: &str, filters: SearchFilters) -> Result<Vec<VocabularyItemView>>;
    async fn get_items_by_status(&self, status: &str) -> Result<Vec<VocabularyItemView>>;
    async fn get_items_by_cefr(&self, level: &str) -> Result<Vec<VocabularyItemView>>;
    
    // 統計
    async fn get_stats(&self) -> Result<VocabularyStats>;
}
```

**データ構造**:

```rust
// 非正規化されたビュー
struct VocabularyItemView {
    item_id: Uuid,
    entry_id: Uuid,
    spelling: String,
    definitions: serde_json::Value,  // JSONB
    synonyms: serde_json::Value,     // JSONB
    antonyms: serde_json::Value,     // JSONB
    // フィルタリング用の個別カラム
    status: String,
    cefr_level: Option<String>,
    definition_count: i32,
    example_count: i32,
}
```

## Projection 側のリポジトリ

### ProjectionRepository

イベントから Read Model への投影を管理します。

**主要な責務**:

- イベントハンドリング
- Read Model の更新
- 投影状態の管理

**インターフェース**:

```rust
trait ProjectionRepository {
    // Read Model の更新
    async fn apply_entry_created(&self, event: EntryCreated) -> Result<()>;
    async fn apply_item_created(&self, event: ItemCreated) -> Result<()>;
    async fn apply_item_updated(&self, event: ItemUpdated) -> Result<()>;
    
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
- TTL: 5分（頻繁に更新されるデータ）
- キャッシュキー: `vocabulary:item:{item_id}`

**Command Service**:

- キャッシュは使用しない（整合性重視）

### 4. パフォーマンス最適化

**Event Store**:

- aggregate_id にインデックス
- occurred_at にインデックス（時系列クエリ用）

**Read Model**:

- spelling, status, cefr_level にインデックス
- JSONB の GIN インデックス（全文検索用）

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

各リポジトリは単一責任原則に従い、それぞれの役割に特化した実装となっています。
