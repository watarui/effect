# User Context - リポジトリ設計

## 概要

User Context には 1 つの主要な集約と認証抽象化層が存在します：

- `UserProfile`：ユーザー情報と設定の管理
- `AuthenticationPort`：認証プロバイダーの抽象インターフェース

このコンテキストは他の全てのコンテキストから参照される中心的な存在であり、
ポート&アダプターパターンにより認証プロバイダーを抽象化し、プロファイル管理、設定管理を担当します。

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
- `find_by_provider_user_id`: 認証プロバイダーのユーザー ID で取得
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

## AuthenticationPort インターフェース

認証プロバイダーを抽象化するポートインターフェースです。

### インターフェース定義

**認証プロバイダーの抽象インターフェース**:

- authenticate: ID トークンを検証してユーザー情報を取得
- verify_token: アクセストークンを検証
- refresh_token: リフレッシュトークンを使用して新しいトークンペアを取得
- revoke_access: ユーザーのアクセスを取り消し

### データ型

**AuthenticatedUser（プロバイダー非依存）**:

- provider_user_id: プロバイダー固有の ID
- email: メールアドレス
- email_verified: メール確認済みか
- provider_type: プロバイダー種別
- display_name: 表示名
- photo_url: プロフィール画像 URL
- custom_claims: カスタムクレーム

**ProviderType**:

- Google
- Apple（将来対応）
- Microsoft（将来対応）

**TokenClaims**:

- provider_user_id: プロバイダーユーザー ID
- email: メールアドレス
- role: ユーザーロール
- expires_at: 有効期限

**TokenPair**:

- access_token: アクセストークン
- refresh_token: リフレッシュトークン
- expires_in: 有効期間（秒）

### FirebaseAuthAdapter 実装

現在のデフォルト実装として Firebase Authentication を使用：

**主要な機能**:

- Firebase Admin SDK で ID トークンを検証
- Firebase 固有の情報を抽象化された型にマッピング
- Google OAuth をプライマリプロバイダーとして使用
- カスタムクレームの管理

## 実装上の考慮事項

### 認証フロー

```
[Client] → [OAuth Provider] → [ID Token]
    ↓
[API Gateway] → [AuthenticationPort] → [Adapter]
    ↓
[UserProfileRepository] → [UserProfile]
```

- provider_user_id と内部 user_id のマッピング
- トークン検証は認証プロバイダーに委譲
- ユーザー作成は初回認証時に自動実行

### 論理削除の実装

- `deleted_at` フィールドで論理削除を管理
- 削除済みユーザーは検索から除外
- 物理削除は行わない（監査証跡のため）

### キャッシング戦略

- 認証情報は短時間キャッシュ（5分）
- プロフィール情報は中時間キャッシュ（30分）
- 設定変更時は即座にキャッシュ無効化

### セキュリティ考慮事項

- パスワードは保存しない（認証プロバイダーに委譲）
- 個人情報へのアクセスは認証済みユーザーのみ
- Admin ロールのチェックは全操作で実施
- トークンはメモリ内でのみ扱い、永続化しない

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

## アダプターの切り替え

### 設定ベースの切り替え

環境変数 `AUTH_PROVIDER` で認証プロバイダーを指定：

- firebase: Firebase Authentication（デフォルト）
- auth0: Auth0（将来実装）
- cognito: AWS Cognito（将来実装）
- supabase: Supabase Auth（将来実装）

ファクトリーパターンで適切なアダプターを生成し、ドメイン層は実装詳細に依存しない。

### 追加可能なアダプター

1. **Auth0Adapter**
   - Auth0 の Universal Login に対応
   - 複数のソーシャルプロバイダーを統合

2. **CognitoAdapter**
   - AWS Cognito User Pools に対応
   - AWS エコシステムとの統合が容易

3. **SupabaseAuthAdapter**
   - Supabase Auth に対応
   - PostgreSQL との統合がシームレス

## 将来の拡張ポイント

現在の最小構成から、将来的に追加可能な機能：

1. **マルチプロバイダー対応**
   - 同一ユーザーが複数の認証方法をリンク
   - アカウント統合機能

2. **2要素認証 (2FA)**
   - TOTP（Google Authenticator など）
   - SMS 認証（非推奨）

3. **セッション管理**
   - リフレッシュトークンの自動更新
   - マルチデバイス対応

シンプルさを保つため、本当に必要になるまで追加しない方針。
