# Vocabulary Context イベントカタログ

## 概要

Vocabulary Context で発生するすべてのドメインイベントのカタログです。各イベントは Event Sourcing パターンに従い、不変で追記のみ可能な形式で記録されます。

## イベント一覧

| イベント名 | 説明 | 発生タイミング |
|-----------|------|-------------|
| EntryCreated | 新しい語彙エントリが作成された | 新しいスペリングの初回登録時 |
| ItemCreated | 語彙項目が作成された | エントリに新しい意味が追加された時 |
| FieldUpdated | 項目のフィールドが更新された | 項目の任意のフィールドが変更された時 |
| ItemPublished | 項目が公開された | Draft から Published に変更時 |
| AIGenerationRequested | AI生成が要求された | AI による内容生成を要求時 |
| AIGenerationCompleted | AI生成が完了した | AI が内容を生成し適用された時 |
| AIGenerationFailed | AI生成が失敗した | AI 生成でエラーが発生した時 |
| UpdateConflicted | 更新が競合した | 楽観的ロックで競合が発生した時 |
| ItemDeleted | 項目が削除された | 項目がソフトデリートされた時 |

## イベント詳細

### 1. EntryCreated

語彙エントリ（見出し語）が作成されたことを表すイベント。

```rust
pub struct EntryCreated {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: EntryId,
    pub aggregate_version: u32,
    
    // イベントペイロード
    pub entry_id: EntryId,
    pub spelling: String,
    pub created_by: CreatedBy,
}

pub enum CreatedBy {
    User(UserId),
    System,
    Import { source: String },
}
```

**発生条件**:

- 新しいスペリングが初めて登録される時
- 自動的に発生（ItemCreated の前提条件）

**例**:

```json
{
  "event_id": "evt_01234567-89ab-cdef-0123-456789abcdef",
  "occurred_at": "2025-08-03T10:30:00Z",
  "aggregate_id": "ent_apple",
  "aggregate_version": 1,
  "entry_id": "ent_apple",
  "spelling": "apple",
  "created_by": {
    "User": "usr_12345"
  }
}
```

### 2. ItemCreated

語彙項目（特定の意味）が作成されたことを表すイベント。

```rust
pub struct ItemCreated {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: ItemId,
    pub aggregate_version: u32,
    
    // イベントペイロード
    pub item_id: ItemId,
    pub entry_id: EntryId,
    pub spelling: String,
    pub disambiguation: String,
    pub creation_method: CreationMethod,
    pub created_by: CreatedBy,
    pub initial_status: ItemStatus,
}

pub enum CreationMethod {
    AiGeneration,
    ManualInput { initial_content: Option<InitialContent> },
    Import { source: String },
}

pub enum ItemStatus {
    Draft,
    PendingAI,
    Published,
}
```

**発生条件**:

- CreateItem コマンドが成功した時
- 必ず対応する EntryCreated が先行する

**例**:

```json
{
  "event_id": "evt_23456789-abcd-ef01-2345-6789abcdef01",
  "occurred_at": "2025-08-03T10:30:05Z",
  "aggregate_id": "itm_apple_fruit",
  "aggregate_version": 1,
  "item_id": "itm_apple_fruit",
  "entry_id": "ent_apple",
  "spelling": "apple",
  "disambiguation": "(fruit)",
  "creation_method": "AiGeneration",
  "created_by": {
    "User": "usr_12345"
  },
  "initial_status": "PendingAI"
}
```

### 3. FieldUpdated

項目の特定フィールドが更新されたことを表すイベント。

```rust
pub struct FieldUpdated {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: ItemId,
    pub aggregate_version: u32,
    
    // イベントペイロード
    pub item_id: ItemId,
    pub field_path: String,  // JSONPath 形式
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub updated_by: UserId,
    pub update_reason: Option<String>,
}
```

**発生条件**:

- UpdateItem コマンドで項目が更新された時
- 各フィールドの変更ごとに個別のイベント

**フィールドパスの例**:

- `pronunciation`: 発音
- `definitions[0].meaning`: 最初の定義の意味
- `examples[2]`: 3番目の例文
- `cefr_level`: CEFR レベル

**例**:

```json
{
  "event_id": "evt_34567890-bcde-f012-3456-789abcdef012",
  "occurred_at": "2025-08-03T11:00:00Z",
  "aggregate_id": "itm_apple_fruit",
  "aggregate_version": 5,
  "item_id": "itm_apple_fruit",
  "field_path": "definitions[0].meaning",
  "old_value": "A round fruit",
  "new_value": "A round fruit with red, green, or yellow skin",
  "updated_by": "usr_67890",
  "update_reason": "Added more detail to definition"
}
```

### 4. ItemPublished

項目が公開されたことを表すイベント。

```rust
pub struct ItemPublished {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: ItemId,
    pub aggregate_version: u32,
    
    // イベントペイロード
    pub item_id: ItemId,
    pub published_by: PublishedBy,
    pub content_completeness: f32,  // 0.0-1.0
}

pub enum PublishedBy {
    User(UserId),
    System,  // AI生成完了時の自動公開
}
```

**発生条件**:

- Draft または PendingAI から Published への状態遷移
- AI 生成完了時の自動公開

### 5. AIGenerationRequested

AI による内容生成が要求されたことを表すイベント。

```rust
pub struct AIGenerationRequested {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: ItemId,
    pub aggregate_version: u32,
    
    // イベントペイロード
    pub item_id: ItemId,
    pub generation_type: GenerationType,
    pub requested_by: UserId,
    pub priority: Priority,
}

pub enum GenerationType {
    Full,           // 全項目生成
    Definitions,    // 定義のみ
    Examples,       // 例文のみ
    Regenerate,     // 再生成
}

pub enum Priority {
    High,
    Normal,
    Low,
}
```

**発生条件**:

- RequestAIGeneration コマンド実行時
- 項目作成時の自動生成要求

### 6. AIGenerationCompleted

AI による内容生成が完了したことを表すイベント。

```rust
pub struct AIGenerationCompleted {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: ItemId,
    pub aggregate_version: u32,
    
    // イベントペイロード
    pub item_id: ItemId,
    pub request_id: EventId,  // AIGenerationRequested の ID
    pub generated_content: GeneratedContent,
    pub ai_model: String,
    pub generation_time_ms: u64,
}

pub struct GeneratedContent {
    pub pronunciation: Option<String>,
    pub phonetic_respelling: Option<String>,
    pub definitions: Vec<Definition>,
    pub examples: Vec<Example>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    pub collocations: Vec<Collocation>,
    pub usage_notes: Option<String>,
}
```

### 7. UpdateConflicted

更新時に競合が発生したことを表すイベント。

```rust
pub struct UpdateConflicted {
    // イベントメタデータ
    pub event_id: EventId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: ItemId,
    pub aggregate_version: u32,
    
    // イベントペイロード
    pub item_id: ItemId,
    pub attempted_by: UserId,
    pub expected_version: u32,
    pub actual_version: u32,
    pub conflicting_fields: Vec<String>,
    pub resolution_strategy: ResolutionStrategy,
}

pub enum ResolutionStrategy {
    ManualResolve,     // ユーザーによる手動解決が必要
    AutoMerged,        // 自動マージ成功
    Retry,             // リトライを推奨
}
```

## イベントの順序保証

同一集約（ItemId）のイベントは、以下の順序で発生します：

1. ItemCreated（必ず最初）
2. FieldUpdated / AIGenerationRequested（任意の順序）
3. AIGenerationCompleted（AIGenerationRequested の後）
4. ItemPublished（任意のタイミング）
5. ItemDeleted（最後、以降イベントなし）

## Projection への影響

各イベントは以下の Read Model を更新します：

| イベント | ItemSearchView | ItemDetailView | ConflictResolutionView | ChangeHistoryView |
|---------|---------------|----------------|----------------------|-------------------|
| EntryCreated | - | - | - | ✓ |
| ItemCreated | ✓ | ✓ | - | ✓ |
| FieldUpdated | ✓ | ✓ | - | ✓ |
| ItemPublished | ✓ | ✓ | - | ✓ |
| AIGenerationCompleted | ✓ | ✓ | - | ✓ |
| UpdateConflicted | - | - | ✓ | ✓ |

## バージョニング戦略

イベントのスキーマ変更時は：

1. **後方互換性を維持**: 新フィールドは Optional
2. **バージョンフィールド追加**: `schema_version: u32`
3. **アップキャスト実装**: 古いイベントを新形式に変換
4. **非互換変更時**: 新イベントタイプを作成

## 更新履歴

- 2025-08-03: 初版作成
