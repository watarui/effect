# User Context - リポジトリ設計

## 概要

User Context には 1 つの主要な集約が存在します：

- `UserProfile`：ユーザー情報と設定の管理

このコンテキストは他の全てのコンテキストから参照される中心的な存在であり、
基本的なユーザー認証（Google OAuth）、プロフィール管理、設定管理を担当します。MVP として最小限の機能に絞った設計です。

## UserProfileRepository

ユーザープロフィールの永続化を担当するリポジトリです。

### 主要な責務

- ユーザーの基本的な CRUD 操作
- 認証用情報の管理
- ユーザー設定の保存と取得
- アカウント状態の管理

### インターフェース設計

**基本的な CRUD 操作**:

- `find_by_id`: ID でユーザーを取得
- `find_by_email`: メールアドレスでユーザーを取得
- `find_by_firebase_uid`: Firebase UID でユーザーを取得
- `save`: ユーザーを保存（新規作成または更新）
- `delete`: ユーザーを削除（論理削除のみ）

**認証関連**:

- `find_for_authentication`: 認証用の情報を取得
- `update_last_login`: 最終ログイン日時を更新

**ユーザー管理**:

- `find_all_paginated`: 全ユーザーをページネーションで取得（Admin用）
- `count_total`: 総ユーザー数を取得
- `count_active`: アクティブユーザー数を取得

**設定管理**:

- `get_learning_preferences`: 学習設定を取得
- `update_learning_preferences`: 学習設定を更新

## 実装上の考慮事項

### Firebase Auth との統合

Firebase Auth を Anti-Corruption Layer で包む：

```
Firebase Auth → AuthenticationProvider → UserProfileRepository
```

- Firebase UID と内部 UserId のマッピング
- トークン検証は Firebase に委譲
- ユーザー作成は Firebase のコールバックで実行

### 論理削除の実装

- `deleted_at` フィールドで論理削除を管理
- 削除済みユーザーは検索から除外
- GDPR 対応の物理削除は別途バッチ処理

### キャッシング戦略

- 認証情報は短時間キャッシュ（5分）
- プロフィール情報は中時間キャッシュ（30分）
- 設定変更時は即座にキャッシュ無効化

### セキュリティ考慮事項

- パスワードは保存しない（Firebase Auth に委譲）
- 個人情報へのアクセスは認証済みユーザーのみ
- Admin ロールのチェックは全操作で実施

## 他コンテキストとの連携

### 読み取り専用の参照

他のコンテキストから UserProfile を参照する場合：

**UserReadModel**:

- user_id: ユーザー識別子
- display_name: 表示名
- role: ロール（権限チェック用）

この読み取りモデルは：

- 他コンテキストでキャッシュ可能
- イベント経由で更新通知
- 最小限の情報のみ公開

### イベント発行

User Context が発行するイベントは全コンテキストに影響：

- UserCreated: 初期データの作成トリガー
- UserDeleted: カスケード削除の開始
- UserRoleChanged: 権限の再評価

## 将来の拡張ポイント

現在の最小構成から、将来的に追加可能な機能：

1. **追加の認証プロバイダー**
   - Apple Sign In
   - Microsoft Account
   - ただし、当面は Google OAuth のみで十分

2. **詳細なプロフィール**
   - 学習履歴の要約
   - 成果バッジ
   - ただし、Progress Context で管理する方が適切

3. **通知設定**
   - 通知機能を実装する場合に追加
   - 現在は通知機能自体がないため不要

シンプルさを保つため、本当に必要になるまで追加しない方針。
