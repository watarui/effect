# User Context - 集約設計

## 概要

User Context は、Effect プロジェクトにおけるユーザー認証、プロファイル管理、学習設定を担当するコンテキストです。
Firebase Authentication を活用し、Google OAuth によるシンプルで安全な認証フローを提供します。

### 主要な責務

- **認証管理**: Firebase Auth による Google OAuth 認証
- **プロファイル管理**: ユーザー情報、学習目標の管理
- **権限管理**: Admin/User のロールベース権限制御
- **セキュリティ**: アクセストークンの管理、権限制御

### 設計方針

- Firebase Auth への依存を最小限に抑える（Anti-Corruption Layer）
- ユーザー集約はシンプルに保つ（認証は Firebase に委譲）
- プライバシーとセキュリティを最優先
- 将来の認証プロバイダー追加に備えた抽象化

## 集約の設計

### 1. UserProfile（ユーザープロファイル）- 集約ルート

ユーザーの基本情報と学習設定を管理します。

**主要な属性**:

- user_id: ユーザー識別子
- email: メールアドレス
- display_name: 表示名
- photo_url: プロフィール画像URL
- learning_goal: 学習目標
- difficulty_preference: 難易度の好み（CEFR レベル）
- role: ユーザーロール（Admin/User）
- account_status: アカウント状態
- created_at: 作成日時
- last_active_at: 最終アクティブ日時
- version: バージョン（楽観的ロック）

**AccountStatus**:

- Active: アクティブ状態
- Deleted: 削除済み（論理削除）

**UserRole**:

- Admin: 管理者（全ユーザーのデータ閲覧、システム設定変更）
- User: 通常ユーザー（自分のデータのみ）

**不変条件**:

- email は一度設定されたら変更不可
- 削除済みアカウントは復活不可
- role の変更は Admin のみ可能

### 2. LearningGoal（学習目標）- 値オブジェクト

ユーザーの学習目標を表現します。シンプルに保ちます。

**目標タイプ**:

- IeltsScore: IELTS スコア目標（例: 7.0）
- GeneralLevel: 一般的なレベル目標（CEFR レベル）
- NoSpecificGoal: 特に目標なし

## 認証管理

### Firebase Auth との統合

Anti-Corruption Layer として Firebase Auth を抽象化：

**AuthenticationProvider インターフェース**:

- `verify_token`: トークンの検証
- `refresh_token`: トークンのリフレッシュ
- `revoke_token`: トークンの無効化

**AuthenticatedUser**:

- uid: Firebase UID
- email: メールアドレス
- email_verified: メール確認済みか
- provider_id: プロバイダー識別子（"google.com"）
- display_name: 表示名
- photo_url: プロフィール画像
- claims: カスタムクレーム

## 設計上の重要な決定

### シンプルさの追求

User Context は必要最小限の機能に絞る：

- 複雑な権限管理は不要（Admin/User の2つのみ）
- 通知設定は実装しない（通知機能自体がない）
- 複雑なプリファレンスは持たない

### Firebase Auth への依存管理

1. **Anti-Corruption Layer**: Firebase 固有の実装を隠蔽
2. **最小限の依存**: 認証のみ Firebase に依存
3. **プロバイダー抽象化**: 将来の認証方式追加に対応

### プライバシーとセキュリティ

1. **個人情報の最小化**: 必要最小限の情報のみ保持
2. **論理削除**: ユーザーデータは物理削除せず論理削除
3. **監査証跡**: 重要な操作はイベントとして記録

### 他コンテキストとの連携

- **Learning Context**: user_id による紐付けのみ
- **Progress Context**: user_id でデータを参照
- **Vocabulary Context**: 作成者情報として user_id を保持

### 将来の拡張性

現在は Google OAuth のみだが、将来的に以下を追加可能：

- Apple Sign In
- Microsoft Account
- Email/Password（非推奨）

ただし、シンプルさを保つため当面は Google OAuth のみで十分。
