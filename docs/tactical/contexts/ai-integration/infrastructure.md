# AI Integration Context - インフラストラクチャ

## 概要

AI Integration Context は単一サービスとして構成され、タスクキューベースの非同期処理により外部 AI サービスとの統合を管理します。

## サービス構成

### ai-integration-service

**責務**:

- AI タスクの受付と管理
- タスクキューの運用
- ワーカープールによる並列処理
- プロバイダー API の抽象化
- リアルタイム通知の配信

**技術スタック**:

- Language: Rust
- Framework: Axum (gRPC + WebSocket)
- Queue: Redis
- Database: PostgreSQL
- Cache: Redis

**デプロイ設定**:

```yaml
Service: ai-integration-service
Platform: Google Cloud Run
CPU: 2
Memory: 2Gi
Min Instances: 1
Max Instances: 20
Concurrency: 100
Environment Variables:
  - WORKER_POOL_SIZE: 10
  - MAX_QUEUE_SIZE: 10000
  - TASK_TIMEOUT: 300s
```

## データストア

### Redis (タスクキュー)

**用途**:

- タスクキューの管理
- 処理中タスクのロック
- キャッシュストレージ

**キュー構造**:

```yaml
Queues:
  - pending_tasks: 待機中タスク (Priority Queue)
  - processing_tasks: 処理中タスク (Hash)
  - dead_letter_queue: 失敗タスク (List)
  - rate_limits: レート制限カウンタ (Hash)
  - circuit_breakers: Circuit Breaker 状態 (Hash)
```

### PostgreSQL (永続化)

**スキーマ**:

```sql
-- タスクテーブル
tasks (
  task_id UUID PRIMARY KEY,
  task_type VARCHAR(50) NOT NULL,
  status VARCHAR(20) NOT NULL,
  requested_by VARCHAR(100) NOT NULL,
  request_content JSONB NOT NULL,
  response_content JSONB,
  created_at TIMESTAMP NOT NULL,
  started_at TIMESTAMP,
  completed_at TIMESTAMP,
  retry_count INT DEFAULT 0,
  error_info JSONB
)

-- チャットセッション
chat_sessions (
  session_id UUID PRIMARY KEY,
  user_id VARCHAR(100) NOT NULL,
  item_id VARCHAR(100),
  status VARCHAR(20) NOT NULL,
  started_at TIMESTAMP NOT NULL,
  last_activity TIMESTAMP NOT NULL,
  context JSONB
)

-- チャットメッセージ
chat_messages (
  message_id UUID PRIMARY KEY,
  session_id UUID REFERENCES chat_sessions,
  role VARCHAR(20) NOT NULL,
  content TEXT NOT NULL,
  tokens_used INT,
  timestamp TIMESTAMP NOT NULL
)

-- 使用統計
usage_stats (
  stat_id UUID PRIMARY KEY,
  user_id VARCHAR(100),
  provider VARCHAR(50),
  task_type VARCHAR(50),
  tokens_used INT,
  cost DECIMAL(10,4),
  success BOOLEAN,
  created_at TIMESTAMP NOT NULL
)
```

## ワーカープール設定

### ワーカー構成

```yaml
Worker Pool:
  - Size: 10 (環境変数で調整可能)
  - Type: 非同期ワーカー
  - Isolation: タスクごとにタイムアウト
  - Restart Policy: 自動再起動

Task Processing:
  - Claim Strategy: Atomic (Redis BLPOP)
  - Timeout: 5分/タスク
  - Retry: 最大3回
  - Backoff: 指数バックオフ
```

### スケーリング戦略

**水平スケーリング**:

- Cloud Run の自動スケーリング
- CPU 使用率: 70% で新インスタンス
- 同時実行数: 100リクエスト/インスタンス

**垂直スケーリング**:

- ワーカープールサイズの動的調整
- メモリ使用量に基づく調整

## プロバイダー管理

### API キー設定

```yaml
Providers:
  gemini:
    api_key: ${GEMINI_API_KEY}
    endpoint: https://generativelanguage.googleapis.com
    priority: 1
    rate_limit: 60/min
    
  openai:
    api_key: ${OPENAI_API_KEY}
    endpoint: https://api.openai.com
    priority: 2
    rate_limit: 3500/min
    
  claude:
    api_key: ${CLAUDE_API_KEY}
    endpoint: https://api.anthropic.com
    priority: 3
    rate_limit: 1000/min
    
  unsplash:
    api_key: ${UNSPLASH_API_KEY}
    endpoint: https://api.unsplash.com
    priority: 1  # 画像用
    rate_limit: 50/hour
```

### Circuit Breaker 設定

```yaml
Circuit Breaker:
  failure_threshold: 5
  success_threshold: 2
  timeout: 60s
  half_open_requests: 1
```

## 監視とロギング

### メトリクス

**重要指標**:

- タスク処理時間 (P50, P95, P99)
- キュー長
- ワーカー使用率
- プロバイダー別成功率
- API 使用量
- コスト追跡

**監視ツール**:

- Cloud Monitoring
- Cloud Logging
- OpenTelemetry
- Custom Dashboards

### アラート設定

```yaml
Alerts:
  - Queue Length > 1000: 警告
  - Task Timeout Rate > 5%: エラー
  - Provider Error Rate > 10%: 重大
  - Circuit Breaker Open: 警告
  - Monthly Cost > 80% Budget: 警告
  - Worker Pool Saturation > 90%: 警告
```

## セキュリティ

### ネットワーク

- VPC 内での通信
- Cloud Run サービス認証
- WebSocket は JWT 認証

### シークレット管理

- Secret Manager による API キー管理
- 自動ローテーション対応
- 最小権限の原則

### データ保護

- PII 検出と自動マスキング
- TLS 1.3 による暗号化
- 監査ログの記録

## CI/CD パイプライン

### ビルド

```yaml
steps:
  - cargo test
  - cargo build --release
  - docker build -t ai-integration-service
  - docker push to Artifact Registry
```

### デプロイ

```yaml
environments:
  - dev: 自動デプロイ (main ブランチ)
  - staging: 手動承認後
  - production: Blue-Green デプロイ
```

## コスト最適化

### API 使用量管理

- ユーザー別の月次制限
- プロバイダー別のコスト追跡
- 自動フォールバック

### リソース最適化

- 低負荷時のスケールダウン
- キャッシュによる API 呼び出し削減
- バッチ処理による効率化

## 災害復旧

### バックアップ

- タスクデータ: 日次バックアップ
- チャット履歴: リアルタイムレプリケーション
- 設定: Git 管理

### 復旧手順

1. Redis キューの復元
2. 未完了タスクの再処理
3. Circuit Breaker のリセット
4. ワーカープールの再起動

## パフォーマンスチューニング

### Redis 最適化

- Connection Pooling
- Pipeline による一括処理
- 適切な TTL 設定

### ワーカー最適化

- 非同期 I/O の活用
- バッチ処理
- メモリプールの使用

### API 呼び出し最適化

- HTTP/2 接続の再利用
- 並列リクエスト
- レスポンスストリーミング
