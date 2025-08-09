# User Context - 集約設計

## 概要

User Context は、Effect プロジェクトにおけるユーザー認証、プロファイル管理、学習設定を担当するコンテキストです。
ポート&アダプターアーキテクチャにより認証プロバイダーを抽象化し、現在は Firebase Authentication + Google OAuth を実装として使用しますが、将来的に他の認証プロバイダーに容易に切り替え可能な設計となっています。

### 主要な責務

- **認証抽象化**: AuthenticationPort インターフェースによる認証の抽象化
- **認証管理**: 認証プロバイダーを通じた OAuth 認証
- **プロファイル管理**: ユーザー情報、学習目標の管理
- **権限管理**: Admin/User のロールベース権限制御
- **セキュリティ**: アクセストークンの管理、権限制御

### 設計方針

- ポート&アダプターパターンによる認証プロバイダーの完全な抽象化
- ドメインモデルから認証プロバイダー固有の詳細を排除
- ユーザー集約はシンプルに保つ（認証は認証プロバイダーに委譲）
- プライバシーとセキュリティを最優先
- 設定変更のみで認証プロバイダーを切り替え可能

## 集約の設計

### 1. UserProfile（ユーザープロファイル）- 集約ルート

ユーザーの基本情報と学習設定を管理します。

**主要な属性**:

- user_id: ユーザー識別子（システム内部 ID）
- provider_user_id: 認証プロバイダーのユーザー ID（Firebase UID など）
- email: メールアドレス
- display_name: 表示名
- photo_url: プロフィール画像 URL
- learning_goal: 学習目標
- difficulty_preference: 難易度の好み（CEFR レベル）
- role: ユーザーロール（Admin/User）
- account_status: アカウント状態
- created_at: 作成日時
- last_active_at: 最終アクティブ日時
- deleted_at: 削除日時（論理削除）
- version: バージョン（楽観的ロック）

**AccountStatus**:

- Active: アクティブ状態
- Deleted: 削除済み（論理削除）

**UserRole**:

- Admin: 管理者（全ユーザーのデータ閲覧、システム設定変更）
- User: 通常ユーザー（自分のデータのみ）

**不変条件**:

- user_id は一度設定されたら変更不可
- provider_user_id と email の組み合わせは一意
- 削除済みアカウント（deleted_at が設定済み）はアクセス不可
- role の変更は Admin のみ可能

### 2. LearningGoal（学習目標）- 値オブジェクト

ユーザーの学習目標を表現します。シンプルに保ちます。

**目標タイプ**:

- IeltsScore: IELTS スコア目標（例: 7.0）
- GeneralLevel: 一般的なレベル目標（CEFR レベル）
- NoSpecificGoal: 特に目標なし

## 認証管理

### ポート&アダプターアーキテクチャ

認証プロバイダーを完全に抽象化し、ドメインから実装詳細を分離：

**AuthenticationPort インターフェース**:

```rust
// ドメイン層のポート定義
trait AuthenticationPort {
    async fn authenticate(&self, token: &str) -> Result<AuthenticatedUser>;
    async fn verify_token(&self, token: &str) -> Result<TokenClaims>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair>;
    async fn revoke_access(&self, user_id: &str) -> Result<()>;
}
```

**FirebaseAuthAdapter （現在の実装）**:

```rust
// インフラ層のアダプター実装
struct FirebaseAuthAdapter {
    firebase_app: FirebaseApp,
    // Firebase 固有の設定
}

impl AuthenticationPort for FirebaseAuthAdapter {
    // Firebase Auth を使用した実装
    // Firebase 固有の詳細はここに隠蔽
}
```

**AuthenticatedUser （プロバイダー非依存）**:

- user_id: システム内部のユーザー ID
- provider_user_id: プロバイダー固有の ID
- email: メールアドレス
- email_verified: メール確認済みか
- provider_type: プロバイダー種別（"google", "apple" など）
- display_name: 表示名
- photo_url: プロフィール画像 URL
- custom_claims: カスタムクレーム（ロールなど）

## 設計上の重要な決定

### シンプルさの追求

User Context は必要最小限の機能に絞る：

- 複雑な権限管理は不要（Admin/User の 2 つのみ）
- 通知設定は実装しない（通知機能自体がない）
- 複雑なプリファレンスは持たない

### 認証プロバイダーの管理

1. **完全な抽象化**: ポート&アダプターパターンで実装を分離
2. **設定ベースの切り替え**: 環境変数または設定ファイルでアダプターを選択
3. **プロバイダー抽象化**: 新しい認証方式はアダプター追加のみ
4. **Anti-Corruption Layer**: プロバイダー固有の詳細はアダプター内に隠蔽

### プライバシーとセキュリティ

1. **個人情報の最小化**: 必要最小限の情報のみ保持
2. **論理削除**: deleted_at フィールドで管理（物理削除はしない）
3. **監査証跡**: 重要な操作はイベントとして記録
4. **トークン管理**: 認証トークンの安全な管理と検証

### 他コンテキストとの連携

- **Learning Context**: user_id による紐付けのみ
- **Progress Context**: user_id でデータを参照
- **Vocabulary Context**: 作成者情報として user_id を保持
