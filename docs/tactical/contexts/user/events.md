# User Context - ドメインイベント

## 概要

User Context で発生するドメインイベントのカタログです。ユーザーのライフサイクル、プロファイル変更、権限管理に関するイベントを管理します。

## イベント一覧

| イベント名 | 説明 | 発生タイミング |
|-----------|------|-------------|
| UserCreated | 新規ユーザーが作成された | 初回ログイン時 |
| UserProfileUpdated | プロファイルが更新された | ユーザー情報の変更時 |
| LearningGoalChanged | 学習目標が変更された | 目標設定の更新時 |
| UserRoleChanged | ユーザーロールが変更された | 管理者による権限変更時 |
| UserDeleted | ユーザーが削除された | アカウント削除時 |
| UserReactivated | ユーザーが再アクティブ化された | 休止後の再ログイン時 |
| UserLastActiveUpdated | 最終アクティブ日時が更新された | ユーザー活動時 |

## イベント詳細

### 1. UserCreated

新規ユーザーが作成されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- email: メールアドレス
- display_name: 表示名
- photo_url: プロフィール画像URL（オプション）
- provider: 認証プロバイダー（"google.com"）
- initial_role: 初期ロール（通常は User）

**発生条件**:

- Firebase Auth で初めて認証された時
- Google OAuth での初回ログイン成功時

**他コンテキストへの影響**:

- Progress Context: 初期プロジェクションの作成
- Learning Context: ユーザー用の初期設定作成

### 2. UserProfileUpdated

ユーザープロファイルが更新されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- updated_fields: 更新されたフィールドのリスト
- old_values: 変更前の値（監査用）
- new_values: 変更後の値

**発生条件**:

- 表示名の変更
- プロフィール画像の更新
- 難易度設定の変更

### 3. LearningGoalChanged

学習目標が変更されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- old_goal: 変更前の目標
- new_goal: 変更後の目標

**目標タイプ**:

- IeltsScore: IELTS スコア目標
- GeneralLevel: CEFR レベル目標
- NoSpecificGoal: 目標なし

**発生条件**:

- ユーザーが設定画面で目標を変更した時
- 初回目標設定時

**他コンテキストへの影響**:

- Learning Context: 学習戦略の調整
- Progress Context: 進捗評価基準の更新

### 4. UserRoleChanged

ユーザーロールが変更されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- old_role: 変更前のロール
- new_role: 変更後のロール
- changed_by: 変更実行者（Admin のみ）
- reason: 変更理由（オプション）

**発生条件**:

- 管理者がユーザーを Admin に昇格させた時
- Admin 権限を剥奪した時

### 5. UserDeleted

ユーザーが削除されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- deletion_type: 削除タイプ（SelfDeletion, AdminDeletion）
- deleted_by: 削除実行者
- reason: 削除理由（オプション）

**発生条件**:

- ユーザー自身がアカウントを削除した時
- 管理者がアカウントを削除した時

**注意事項**:

- 論理削除のみ（データは保持）
- GDPR 対応のための物理削除は別途バッチ処理

### 6. UserReactivated

ユーザーが再アクティブ化されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- last_active_before: 前回のアクティブ日時
- inactive_duration: 非アクティブ期間

**発生条件**:

- 長期間（30日以上）非アクティブ後のログイン
- 休止状態からの復帰

### 7. UserLastActiveUpdated

最終アクティブ日時が更新されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- previous_active_at: 前回のアクティブ日時
- activity_type: アクティビティタイプ（Login, Learning, ProfileUpdate など）

**発生条件**:

- ログイン時
- 学習セッション開始時
- 重要な操作実行時

**注意事項**:

- 高頻度イベントのため、バッチ処理やサンプリングを考慮
- 5分以内の連続アクティビティは集約

## イベント処理の考慮事項

### プライバシー保護

- 個人情報を含むイベントは暗号化
- イベントログへの適切なアクセス制御
- GDPR 準拠のための配慮

### 他コンテキストとの連携

User Context のイベントは多くの他コンテキストに影響：

- 非同期処理で疎結合を維持
- イベントの順序性は保証しない
- 冪等性を考慮した設計

### パフォーマンス最適化

- UserLastActiveUpdated は高頻度のため特別扱い
- イベントの集約とバッチ処理
- 重要度に応じた処理優先度
