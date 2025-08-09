# Learning Context - インフラストラクチャ

## 概要

Learning Context の技術選択、デプロイ構成、非機能要件の定義です。

## 技術スタック

### データストア

**PostgreSQL**

- 最小限の永続化: 基本的な学習記録のみ
- バージョン: 15以上（JSONB、gen_random_uuid() のため）

**Redis**

- 用途: セッション状態の一時保存（メイン）、項目詳細のキャッシュ
- TTL設定:
  - セッション状態: 2時間
  - 項目詳細: 5分
- 構成: Google Cloud Memorystore

### メッセージング

**Google Cloud Pub/Sub**

- Progress Context への通知用
- トピック構成:
  - `learning-to-progress`: Progress Context への通知専用

### コンテナ・オーケストレーション

**Google Cloud Run**

- 単一サービスをコンテナとしてデプロイ
- 自動スケーリング
- サーバーレス課金

## デプロイ構成

```yaml
services:
  learning-service:
    image: gcr.io/effect-project/learning-service
    env:
      - DATABASE_URL
      - REDIS_URL
      - PUBSUB_TOPIC
      - ALGORITHM_SERVICE_URL
      - VOCABULARY_SERVICE_URL
    scaling:
      min_instances: 2
      max_instances: 20
    resources:
      cpu: 2
      memory: 1Gi
```

## 非機能要件

### 認証・認可

**Firebase Authentication**

- JWT トークンによる認証
- API Gateway でトークン検証

**権限モデル**:

- 一般ユーザー: 自分のセッション・記録のみアクセス可能
- 管理者: 全ユーザーのデータ閲覧（分析用）

### レート制限

**実装方法**: Redis を使用したスライディングウィンドウ

**制限値**:

- セッション開始: 10 req/hour/user
- クエリ操作: 600 req/min/user
- 分析API: 60 req/min/user

**実装例**:

```rust
async fn check_session_rate_limit(user_id: &str, redis: &Redis) -> Result<bool> {
    let key = format!("rate_limit:session:{}", user_id);
    let window = 3600; // 1時間
    let max_requests = 10;
    
    let current = redis.incr(&key).await?;
    if current == 1 {
        redis.expire(&key, window).await?;
    }
    Ok(current <= max_requests)
}
```

### ロギング

**Google Cloud Logging**

- 構造化ログ（JSON形式）
- トレースID による分散トレーシング
- ログレベル: INFO（本番）、DEBUG（開発）

**ログ形式**:

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "severity": "INFO",
  "trace": "projects/effect/traces/abc123",
  "service": "learning-command",
  "message": "Session started",
  "labels": {
    "user_id": "user123",
    "session_id": "session456",
    "item_count": 25
  }
}
```

### モニタリング

**Google Cloud Monitoring**

基本メトリクス:

- リクエスト数
- レスポンスタイム（p50, p95, p99）
- エラー率
- CPU/メモリ使用率

**カスタムメトリクス**:

```rust
// セッション完了率
metric!("learning.session.completion_rate", {
    "user_id" => user_id,
    "session_type" => session_type,
});

// 平均応答時間
histogram!("learning.response_time_ms", response_time);

// 習熟度変化
counter!("learning.mastery_transitions", {
    "from_status" => old_status,
    "to_status" => new_status,
});
```

**アラート設定**:

- セッション完了率 < 70%
- レスポンスタイム p99 > 500ms
- エラー率 > 1%
- イベント処理遅延 > 5秒

### セキュリティ

**ネットワーク**:

- Cloud Run サービス間は内部通信
- HTTPS のみ許可
- Cloud Armor で DDoS 対策

**データ**:

- データベース接続は Cloud SQL Proxy 経由
- シークレットは Secret Manager で管理
- 個人学習データの暗号化

### バックアップ・DR

**データベース**:

- 自動バックアップ（毎日）
- Point-in-time recovery（7日間）
- マルチリージョンレプリケーション（DR用）

**Event Store**:

- イベントは不変なので、バックアップから完全復元可能
- スナップショット + イベント再生で高速復旧

### パフォーマンス目標

- セッション開始: p99 < 200ms
- 項目提示: p99 < 100ms
- クエリ応答: p99 < 150ms
- 分析API: p99 < 1秒
- 可用性: 99.9%

## 学習セッション特有の考慮事項

### リアルタイム性

**3秒タイマーの実装**:

```rust
// クライアント側でタイマー管理
// サーバー側では timestamp で検証
async fn validate_auto_confirmation(
    presented_at: DateTime<Utc>,
    confirmed_at: DateTime<Utc>,
) -> bool {
    let elapsed = confirmed_at - presented_at;
    elapsed >= Duration::seconds(3)
}
```

### セッション管理

**タイムアウト処理**:

- 30分間操作なし → セッション自動中断
- Cloud Scheduler で定期的にチェック

```rust
// タイムアウトセッションの検出
async fn find_timed_out_sessions() -> Vec<SessionId> {
    let timeout_threshold = Utc::now() - Duration::minutes(30);
    
    sqlx::query!(
        "SELECT session_id FROM learning_sessions 
         WHERE status = 'InProgress' 
         AND last_activity_at < $1",
        timeout_threshold
    )
    .fetch_all(&pool)
    .await
}
```

### 外部サービス連携

**Algorithm Context**:

- gRPC による同期通信
- タイムアウト: 3秒
- サーキットブレーカー実装

**Vocabulary Context**:

- gRPC による同期通信
- 積極的なキャッシュ（1時間）
- バッチ取得で効率化

## 開発・テスト環境

### ローカル開発

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: effect_learning
      POSTGRES_USER: effect
      POSTGRES_PASSWORD: effect_password
    ports:
      - "5433:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6380:6379"

  pubsub-emulator:
    image: gcr.io/google.com/cloudsdktool/cloud-sdk
    command: gcloud beta emulators pubsub start --host-port=0.0.0.0:8085
    ports:
      - "8085:8085"
```

### CI/CD パイプライン

```yaml
# .github/workflows/learning-service.yml
steps:
  - name: Run tests
    run: |
      cargo test --all-features
      cargo clippy -- -D warnings

  - name: Build and push
    run: |
      docker build -t gcr.io/$PROJECT/learning-service .
      docker push gcr.io/$PROJECT/learning-service

  - name: Deploy
    run: |
      gcloud run deploy learning-service \
        --image gcr.io/$PROJECT/learning-service \
        --platform managed \
        --region us-central1
```
