# User Context - ドメインイベント

## 概要

User Context で発生するドメインイベントのカタログです。シンプルさを重視し、必要最小限のイベントのみを定義しています。

## イベント一覧

| イベント名 | 説明 | 発生タイミング | 優先度 |
|-----------|------|-------------|--------|
| UserCreated | 新規ユーザーが作成された | 初回ログイン時 | 高 |
| UserDeleted | ユーザーが削除された | アカウント削除時 | 高 |
| UserProfileUpdated | プロファイルが更新された | ユーザー情報の変更時 | 中 |
| UserRoleChanged | ユーザーロールが変更された | 管理者による権限変更時 | 中 |

## イベント詳細

### 1. UserCreated

新規ユーザーが作成されたことを表すイベント。

**イベント構造**:

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| event_id | EventId | イベントID |
| occurred_at | DateTime | 発生日時 |
| user_id | UserId | ユーザーID |
| email | String | メールアドレス |
| display_name | String（オプション） | 表示名 |
| photo_url | String（オプション） | プロフィール画像URL |
| provider_type | ProviderType | 抽象化されたプロバイダー種別 |
| initial_role | UserRole | 初期ロール |

**発生条件**:

- 認証プロバイダーで初めて認証された時
- OAuth での初回ログイン成功時

**他コンテキストへの影響**:

- Progress Context: 初期データの作成
- Learning Context: ユーザー用の初期設定作成
- Vocabulary Context: ユーザー参照の追加

### 2. UserDeleted

ユーザーが削除されたことを表すイベント。

**イベント構造**:

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| event_id | EventId | イベントID |
| occurred_at | DateTime | 発生日時 |
| user_id | UserId | 削除対象ユーザーID |
| deletion_type | DeletionType | 削除種別 |
| deleted_by | UserId | 削除実行者ID |
| reason | String（オプション） | 削除理由 |

**DeletionType の値**:

- SelfDeletion: ユーザー自身による削除
- AdminDeletion: 管理者による削除

**発生条件**:

- ユーザー自身がアカウントを削除した時
- 管理者がアカウントを削除した時

**他コンテキストへの影響**:

- 全コンテキスト: カスケード削除の開始
- 論理削除として処理（deleted_at を設定）

### 3. UserProfileUpdated

ユーザープロファイルが更新されたことを表すイベント。

**イベント構造**:

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| event_id | EventId | イベントID |
| occurred_at | DateTime | 発生日時 |
| user_id | UserId | ユーザーID |
| updated_fields | 文字列リスト | 更新されたフィールド名 |
| changes | Map | 変更内容の詳細 |

**FieldChange 構造**:

- old_value: 変更前の値
- new_value: 変更後の値

**発生条件**:

- 表示名の変更
- プロフィール画像の更新
- 学習目標の変更
- 難易度設定の変更

**他コンテキストへの影響**:

- Learning Context: 学習設定の更新が必要な場合のみ

### 4. UserRoleChanged

ユーザーロールが変更されたことを表すイベント。

**イベント構造**:

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| event_id | EventId | イベントID |
| occurred_at | DateTime | 発生日時 |
| user_id | UserId | 対象ユーザーID |
| old_role | UserRole | 変更前のロール |
| new_role | UserRole | 変更後のロール |
| changed_by | UserId | 変更実行者ID |
| reason | String（オプション） | 変更理由 |

**発生条件**:

- 管理者がユーザーを Admin に昇格させた時
- Admin 権限を剥奪した時

**他コンテキストへの影響**:

- 全コンテキスト: 権限の再評価が必要

## 削除されたイベント

シンプルさを保つため、以下のイベントは定義しない：

### 削除理由

1. **LearningGoalChanged**
   - UserProfileUpdated に統合可能
   - 学習目標はプロファイルの一部

2. **UserReactivated**
   - ログイン時に last_active_at を更新するだけで十分
   - 特別なイベントは不要

3. **UserLastActiveUpdated**
   - 高頻度イベントでシステムに負荷
   - データベースのフィールド更新で十分

## イベント処理の考慮事項

### シンプルさの維持

- 必要最小限のイベントのみ定義
- 高頻度イベントは発行しない
- 複雑なイベントチェーンを避ける

### プライバシー保護

- 個人情報は最小限に
- イベントログへの適切なアクセス制御
- 論理削除で監査証跡を保持

### 他コンテキストとの連携

User Context のイベントは最小限に：

- UserCreated: 初期データ作成のトリガー
- UserDeleted: カスケード削除のトリガー
- その他は必要に応じて通知

### パフォーマンス最適化

- イベント数を最小化
- バッチ処理は不要（イベントが少ないため）
- Pub/Sub で非同期処理
