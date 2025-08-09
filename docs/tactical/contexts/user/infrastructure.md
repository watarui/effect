# User Context - インフラストラクチャ

## 概要

User Context の技術選択、デプロイ構成、非機能要件の定義です。単一サービスとしてシンプルに実装します。

## 技術スタック

### データストア

**PostgreSQL**

- 用途: ユーザープロファイルの永続化
- バージョン: 15以上（gen_random_uuid() のため）
- 接続プール: 最小 5、最大 20

**Redis**

- 用途: セッションキャッシュ、クエリキャッシュ
- TTL設定:
  - セッション: 24時間
  - ユーザープロファイル: 30分
  - ユーザーリスト: 5分
  - 統計情報: 1時間
- 構成: Google Cloud Memorystore

### 認証プロバイダー

**Firebase Authentication（現在）**

- Google OAuth 2.0
- ID トークン検証
- カスタムクレーム管理

**将来対応予定**

- Auth0
- AWS Cognito
- Supabase Auth

### メッセージング

**Google Cloud Pub/Sub**

- イベント発行用
- トピック構成:
  - `user-events`: ドメインイベント発行

### コンテナ・オーケストレーション

**Google Cloud Run**

- 単一サービスをコンテナとしてデプロイ
- 自動スケーリング
- サーバーレス課金

## デプロイ構成

```yaml
services:
  user-service:
    image: gcr.io/effect-project/user-service
    env:
      - DATABASE_URL
      - REDIS_URL
      - PUBSUB_TOPIC
      - AUTH_PROVIDER  # firebase, auth0, cognito
      - FIREBASE_PROJECT_ID
      - FIREBASE_SERVICE_ACCOUNT
    scaling:
      min_instances: 1
      max_instances: 10
    resources:
      cpu: 1
      memory: 512Mi
```

## データベース設計

### テーブル構造

```sql
CREATE TABLE users (
    user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_user_id VARCHAR(255) NOT NULL,
    provider_type VARCHAR(50) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(100),
    photo_url VARCHAR(500),
    learning_goal JSONB NOT NULL DEFAULT '{"type": "none"}',
    difficulty_preference VARCHAR(2) NOT NULL DEFAULT 'B1',
    role VARCHAR(20) NOT NULL DEFAULT 'USER',
    account_status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1,
    
    CONSTRAINT uk_provider_user UNIQUE (provider_user_id, provider_type),
    CONSTRAINT chk_role CHECK (role IN ('ADMIN', 'USER')),
    CONSTRAINT chk_status CHECK (account_status IN ('ACTIVE', 'DELETED')),
    CONSTRAINT chk_difficulty CHECK (difficulty_preference IN ('A1', 'A2', 'B1', 'B2', 'C1', 'C2'))
);

-- インデックス
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_provider ON users(provider_user_id, provider_type);
CREATE INDEX idx_users_created ON users(created_at DESC);
CREATE INDEX idx_users_active ON users(last_active_at DESC);
CREATE INDEX idx_users_deleted ON users(deleted_at) WHERE deleted_at IS NOT NULL;
```

### マイグレーション

```sql
-- 001_create_users_table.sql
CREATE TABLE users (...);

-- 002_add_provider_abstraction.sql
ALTER TABLE users 
  RENAME COLUMN firebase_uid TO provider_user_id;
ALTER TABLE users 
  ADD COLUMN provider_type VARCHAR(50) NOT NULL DEFAULT 'google';

-- 003_add_soft_delete.sql
ALTER TABLE users 
  ADD COLUMN deleted_at TIMESTAMPTZ;
CREATE INDEX idx_users_deleted ON users(deleted_at) 
  WHERE deleted_at IS NOT NULL;
```

## 非機能要件

### 認証・認可

**トークン管理**

- トークン検証ミドルウェアで認証処理
- キャッシュを使用して検証結果を 5 分間保持
- 認証プロバイダーに検証を委譲

**権限チェック**

ロールベースのアクセス制御：

- Admin: すべてのリソースへのアクセス可能
- User: 自分のデータのみ読み書き可能

### レート制限

**実装方法**: Redis によるスライディングウィンドウ方式

**制限値**:

| エンドポイント | 認証なし | 認証済み | Admin |
|--------------|---------|---------|-------|
| 認証 | 10/hour | - | - |
| 読み取り | 60/min | 600/min | 無制限 |
| 書き込み | - | 60/min | 無制限 |

### ロギング

**Google Cloud Logging**

```json
{
  "timestamp": "2025-01-01T00:00:00Z",
  "severity": "INFO",
  "trace": "projects/effect/traces/abc123",
  "service": "user-service",
  "message": "User authenticated",
  "labels": {
    "user_id": "user123",
    "provider": "google",
    "action": "sign_in"
  }
}
```

**ログレベル**:

- Production: INFO
- Staging: DEBUG
- Development: TRACE

### モニタリング

**Google Cloud Monitoring**

基本メトリクス:

- リクエスト数
- レスポンスタイム（p50, p95, p99）
- エラー率
- CPU/メモリ使用率

**カスタムメトリクス**:

- 認証成功率の追跡
- プロファイル更新頻度の計測
- アクティブユーザー数の監視

**アラート設定**:

- 認証エラー率 > 5%
- レスポンスタイム p99 > 500ms
- データベース接続エラー

### セキュリティ

**ネットワーク**:

- HTTPS のみ許可
- Cloud Armor で DDoS 対策
- 内部通信は VPC 内

**データ保護**:

- 個人情報の暗号化（at rest）
- TLS 1.3（in transit）
- Secret Manager でシークレット管理

**監査ログ**:

重要操作の記録：

- 操作実行者
- 操作内容
- 対象リソース
- 操作結果
- タイムスタンプ

### バックアップ・DR

**データベース**:

- 自動バックアップ（毎日）
- Point-in-time recovery（7日間）
- マルチリージョンレプリケーション

**復旧手順**:

1. 最新のバックアップからリストア
2. イベントログから差分を再生
3. キャッシュのクリア
4. ヘルスチェック

### パフォーマンス目標

| メトリクス | 目標値 |
|-----------|--------|
| 認証応答時間 | p99 < 200ms |
| クエリ応答時間 | p99 < 100ms |
| 更新応答時間 | p99 < 150ms |
| 可用性 | 99.9% |

## 開発・テスト環境

### ローカル開発

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: effect_user
      POSTGRES_USER: effect
      POSTGRES_PASSWORD: effect_password
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  firebase-emulator:
    image: andreysenov/firebase-tools
    command: firebase emulators:start --only auth
    ports:
      - "9099:9099"  # Auth emulator
```

### テスト戦略

- ユニットテスト: モックを使用した各コンポーネントのテスト
- 統合テスト: データベースのテストインスタンスを使用
- E2E テスト: 認証フロー全体の検証

### CI/CD パイプライン

**GitHub Actions での自動化**:

1. テスト実行
   - ユニットテスト
   - リンターチェック
   - カバレッジレポート

2. デプロイ
   - main ブランチへのマージ時に自動デプロイ
   - Google Cloud Run へのコンテナデプロイ
   - ブルーグリーンデプロイメント

## 認証プロバイダーの設定

### Firebase（現在）

```yaml
firebase:
  project_id: "effect-project"
  service_account_path: "/secrets/firebase-sa.json"
  auth:
    providers:
      - google.com
```

### Auth0（将来）

```yaml
auth0:
  domain: "effect.auth0.com"
  client_id: "..."
  client_secret: "..."
  audience: "https://api.effect.app"
```

### AWS Cognito（将来）

```yaml
cognito:
  user_pool_id: "us-east-1_xxxxx"
  client_id: "..."
  region: "us-east-1"
```

## コスト最適化

1. **Cloud Run の自動スケーリング**
   - 最小インスタンス: 1（コールドスタート対策）
   - 最大インスタンス: 10（ユーザー数に応じて調整）

2. **キャッシュ活用**
   - Redis でクエリ結果をキャッシュ
   - トークン検証結果のキャッシュ

3. **データベース最適化**
   - 適切なインデックス
   - クエリの最適化
   - コネクションプーリング
