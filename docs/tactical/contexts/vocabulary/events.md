# Vocabulary Context - ドメインイベント

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

**イベント構造**:

- event_id: イベントの一意識別子
- occurred_at: 発生日時
- aggregate_id: エントリID
- aggregate_version: 集約のバージョン
- entry_id: エントリID
- spelling: 綴り
- created_by: 作成者（ユーザー、システム、インポート）

**発生条件**:

- 新しいスペリングが初めて登録される時
- 自動的に発生（ItemCreated の前提条件）

### 2. ItemCreated

語彙項目（特定の意味）が作成されたことを表すイベント。

**イベント構造**:

- event_id: イベントの一意識別子
- occurred_at: 発生日時
- aggregate_id: 項目ID
- aggregate_version: 集約のバージョン
- item_id: 項目ID
- entry_id: エントリID
- spelling: 綴り
- disambiguation: 意味の区別
- creation_method: 作成方法（AI生成、手動入力、インポート）
- created_by: 作成者
- initial_status: 初期ステータス

**発生条件**:

- CreateItem コマンドが成功した時
- 必ず対応する EntryCreated が先行する

### 3. FieldUpdated

項目の特定フィールドが更新されたことを表すイベント。

**イベント構造**:

- event_id: イベントの一意識別子
- occurred_at: 発生日時
- aggregate_id: 項目ID
- aggregate_version: 集約のバージョン
- item_id: 項目ID
- field_path: 更新されたフィールドのパス（JSONPath 形式）
- old_value: 変更前の値
- new_value: 変更後の値
- updated_by: 更新者
- update_reason: 更新理由（任意）

**発生条件**:

- UpdateItem コマンドで特定フィールドが変更された時
- 楽観的ロックチェックを通過した場合のみ

### 4. ItemPublished

項目が公開されたことを表すイベント。

**イベント構造**:

- event_id: イベントの一意識別子
- occurred_at: 発生日時
- aggregate_id: 項目ID
- aggregate_version: 集約のバージョン
- item_id: 項目ID
- published_by: 公開者
- previous_status: 変更前のステータス

**発生条件**:

- 項目のステータスが Draft から Published に変更された時
- 管理者権限を持つユーザーが実行した場合のみ

### 5. AIGenerationRequested

AI による内容生成が要求されたことを表すイベント。

**イベント構造**:

- event_id: イベントの一意識別子
- occurred_at: 発生日時
- aggregate_id: 項目ID
- aggregate_version: 集約のバージョン
- item_id: 項目ID
- generation_type: 生成タイプ（全体生成、部分生成）
- requested_fields: 生成対象フィールドのリスト
- requested_by: 要求者

**発生条件**:

- AI による内容生成が要求された時
- AI Integration Context との非同期連携の開始

### 6. AIGenerationCompleted

AI による内容生成が完了したことを表すイベント。

**イベント構造**:

- event_id: イベントの一意識別子
- occurred_at: 発生日時
- aggregate_id: 項目ID
- aggregate_version: 集約のバージョン
- item_id: 項目ID
- request_id: 元の生成要求ID
- generated_fields: 生成されたフィールドと値のマップ
- generation_duration: 生成にかかった時間

**発生条件**:

- AI が内容を正常に生成し、項目に適用された時
- AI Integration Context からの非同期応答

### 7. UpdateConflicted

更新が競合したことを表すイベント。

**イベント構造**:

- event_id: イベントの一意識別子
- occurred_at: 発生日時
- aggregate_id: 項目ID
- aggregate_version: 集約のバージョン
- item_id: 項目ID
- expected_version: 期待されていたバージョン
- actual_version: 実際のバージョン
- conflicted_fields: 競合したフィールドのリスト
- attempted_by: 更新を試みたユーザー

**発生条件**:

- 楽観的ロックチェックで競合が検出された時
- 自動マージが不可能な変更があった場合

## イベントストリーム

イベントは以下の順序で発生します：

1. EntryCreated → ItemCreated
2. ItemCreated → AIGenerationRequested（AI生成の場合）
3. AIGenerationRequested → AIGenerationCompleted または AIGenerationFailed
4. FieldUpdated は ItemCreated 後、任意のタイミングで発生
5. ItemPublished は通常、内容が完成した後に発生
