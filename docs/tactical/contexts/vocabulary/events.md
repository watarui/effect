# Vocabulary Context - ドメインイベント

## 概要

Vocabulary Context で発生するすべてのドメインイベントのカタログです。各イベントは Event Sourcing パターンに従い、不変で追記のみ可能な形式で記録されます。

## 実装済みイベント一覧

| イベント名 | 説明 | 発生タイミング |
|-----------|------|---------------|
| EntryCreated | 新しい語彙エントリが作成された | 新しいスペリングの初回登録時 |
| ItemCreated | 語彙項目が作成された | エントリに新しい意味が追加された時 |
| ItemUpdated | 項目の特定フィールドが更新された | 項目の任意のフィールドが変更された時 |
| ItemPublished | 項目が公開された | Draft から Published に変更時 |

## イベント詳細

### 1. EntryCreated

語彙エントリ（見出し語）が作成されたことを表すイベント。

**イベント構造**:

- entry_id: エントリID
- spelling: 綴り
- occurred_at: 発生日時

**発生条件**:

- 新しいスペリングが初めて登録される時
- 自動的に発生（ItemCreated の前提条件）

### 2. ItemCreated

語彙項目（特定の意味）が作成されたことを表すイベント。

**イベント構造**:

- item_id: 項目ID
- entry_id: エントリID
- spelling: 綴り
- definitions: 定義のリスト
- part_of_speech: 品詞
- register: レジスター
- domain: 分野
- created_by: 作成者ID
- occurred_at: 発生日時

**発生条件**:

- CreateItem コマンドが成功した時
- 必ず対応する EntryCreated が先行する

### 3. ItemUpdated

項目の特定フィールドが更新されたことを表すイベント。

**イベント構造**:

- item_id: 項目ID
- field_name: 更新されたフィールド名
- old_value: 変更前の値（JSON）
- new_value: 変更後の値（JSON）
- updated_by: 更新者ID
- occurred_at: 発生日時

**発生条件**:

- UpdateItem コマンドで特定フィールドが変更された時
- 楽観的ロックチェックを通過した場合のみ

### 4. ItemPublished

項目が公開されたことを表すイベント。

**イベント構造**:

- item_id: 項目ID
- published_by: 公開者ID
- occurred_at: 発生日時

**発生条件**:

- 項目のステータスが Draft から Published に変更された時
- 管理者権限を持つユーザーが実行した場合のみ

## 今後実装予定のイベント

以下のイベントは、AI Integration Context や競合解決機能の実装時に追加予定：

- **AIGenerationRequested**: AI による内容生成の要求
- **AIGenerationCompleted**: AI 生成の完了
- **AIGenerationFailed**: AI 生成の失敗
- **UpdateConflicted**: 楽観的ロックによる競合検出
- **ItemDeleted**: 項目の論理削除

## イベントストリーム

イベントは以下の順序で発生します：

1. EntryCreated → ItemCreated（新しい spelling の場合）
2. ItemCreated または ItemUpdated（既存 spelling への追加）
3. ItemPublished は内容が完成した後に発生

## Event Sourcing による実装

すべてのイベントは Event Store に以下の形式で保存される：

```sql
events (
  event_id UUID,
  aggregate_id UUID,        -- entry_id または item_id
  aggregate_type VARCHAR,   -- "VocabularyEntry" または "VocabularyItem"
  event_type VARCHAR,       -- "EntryCreated", "ItemCreated" など
  event_data JSONB,         -- イベントの詳細データ
  event_version INTEGER,    -- 集約のバージョン
  occurred_at TIMESTAMPTZ
)
```

これにより：

- 完全な変更履歴の保持
- 任意の時点の状態再現
- 監査証跡の提供
