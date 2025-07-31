# Effect インフラストラクチャ詳細

## 概要

Effect プロジェクトは、マイクロサービスアーキテクチャと DDD の学習を目的として、本格的な分散システムインフラストラクチャを採用しています。ローカル開発環境では完全分離型のデータベース構成により、実際の本番環境に近い環境で開発・学習が可能です。

## データベース構成

### 完全分離型アーキテクチャ

各マイクロサービスは独自のデータベースインスタンスを持ちます：

| サービス | データベース名 | ポート | 用途 |
|---------|--------------|--------|-----|
| Event Store | event_store_db | 5432 | イベントソーシング用の中央イベントストア |
| Learning Service | learning_db | 5433 | 学習セッション、UserItemRecord |
| Vocabulary Service | vocabulary_db | 5434 | 語彙エントリ管理 |
| User Service | user_db | 5435 | ユーザープロファイル、認証情報 |
| Progress Service | progress_db | 5436 | 進捗プロジェクション |
| Algorithm Service | algorithm_db | 5437 | 学習記録、アルゴリズムデータ |
| AI Service | ai_db | 5438 | AI 生成タスク、チャットセッション |
| Saga Service | saga_db | 5439 | Saga 実行状態管理 |

### PostgreSQL 設定

```yaml
# 各 PostgreSQL インスタンスの共通設定
image: postgres:18beta2-alpine3.22
environment:
  POSTGRES_USER: effect
  POSTGRES_PASSWORD: effect_password
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U effect"]
  interval: 10s
  timeout: 5s
  retries: 5
```

### データ永続化

各データベースは独立したボリュームで永続化されます：

```yaml
volumes:
  postgres_event_store_data:
  postgres_learning_data:
  postgres_vocabulary_data:
  postgres_user_data:
  postgres_progress_data:
  postgres_algorithm_data:
  postgres_ai_data:
  postgres_saga_data:
```

## Redis 構成

### 用途

Redis はキャッシュレイヤーとして使用されます：

- セッションストア
- 一時的なデータキャッシュ
- レート制限カウンター

## Google Pub/Sub 構成

### 用途

Google Pub/Sub はイベントバスとして使用されます：

- ドメインイベントの配信
- 非同期メッセージング
- イベント駆動アーキテクチャの基盤

### ローカル開発

ローカル開発では Google Pub/Sub エミュレータを使用：

```yaml
pubsub:
  image: gcr.io/google.com/cloudsdktool/google-cloud-cli:emulators
  command: gcloud beta emulators pubsub start --host-port=0.0.0.0:8085
  ports:
    - "8085:8085"
  environment:
    PUBSUB_PROJECT_ID: effect-local
```

### 設定

```yaml
redis:
  image: redis:8.2-rc1-alpine3.22
  ports:
    - "6379:6379"
  volumes:
    - redis_data:/data
  command: redis-server --appendonly yes
```

## 開発支援ツール

### pgAdmin

PostgreSQL の GUI 管理ツール：

```yaml
pgadmin:
  image: dpage/pgadmin4:latest
  ports:
    - "5050:80"
  environment:
    PGADMIN_DEFAULT_EMAIL: admin@example.com
    PGADMIN_DEFAULT_PASSWORD: admin
```

アクセス: <http://localhost:5050>

### RedisInsight

Redis の GUI 管理ツール：

```yaml
redisinsight:
  image: redislabs/redisinsight:latest
  ports:
    - "8001:8001"
```

アクセス: <http://localhost:8001>

## モニタリングスタック

### Prometheus

メトリクス収集：

```yaml
prometheus:
  image: prom/prometheus:latest
  ports:
    - "9090:9090"
  volumes:
    - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
```

### Grafana

メトリクス可視化：

```yaml
grafana:
  image: grafana/grafana:latest
  ports:
    - "3000:3000"
  environment:
    GF_SECURITY_ADMIN_PASSWORD: admin
```

アクセス: <http://localhost:3000>

### Jaeger

分散トレーシング：

```yaml
jaeger:
  image: jaegertracing/all-in-one:latest
  ports:
    - "16686:16686"  # Jaeger UI
    - "14268:14268"  # HTTP collector
```

アクセス: <http://localhost:16686>

## ネットワーク構成

すべてのサービスは `effect-network` という共通のネットワークに接続されます：

```yaml
networks:
  effect-network:
    driver: bridge
```

これにより：

- サービス間はサービス名で通信可能
- 外部からは公開されたポートのみアクセス可能
- セキュアな内部通信が実現

## 環境変数管理

### .env ファイル

プロジェクトルートの `.env` ファイルで環境変数を管理：

```bash
# Database
POSTGRES_USER=effect
POSTGRES_PASSWORD=effect_password

# Service Ports
POSTGRES_EVENT_STORE_PORT=5432
POSTGRES_LEARNING_PORT=5433
POSTGRES_VOCABULARY_PORT=5434
POSTGRES_USER_PORT=5435
POSTGRES_PROGRESS_PORT=5436
POSTGRES_ALGORITHM_PORT=5437
POSTGRES_AI_PORT=5438
POSTGRES_SAGA_PORT=5439

# Redis
REDIS_PORT=6379
```

### サービス固有の環境変数

各サービスは独自の環境変数を持ちます：

```yaml
learning-service:
  environment:
    - DATABASE_URL=postgresql://effect:effect_password@postgres-learning:5432/learning_db
    - REDIS_URL=redis://redis:6379
    - SERVICE_PORT=50051
```

## セキュリティ考慮事項

### ローカル開発環境

- すべてのパスワードは開発用のデフォルト値
- 本番環境では必ず変更すること

### ネットワークセキュリティ

- データベースは内部ネットワークのみ
- 必要最小限のポートのみ公開
- サービス間通信は内部ネットワーク経由

### データ保護

- ボリュームによるデータ永続化
- 定期的なバックアップ推奨（本番環境）

## トラブルシューティング

### ポート競合

既に使用中のポートがある場合は、`.env` ファイルで変更：

```bash
POSTGRES_LEARNING_PORT=15433  # デフォルトの 5433 から変更
```

### メモリ不足

Docker Desktop のメモリ割り当てを増やす：

- 推奨: 8GB 以上
- 設定: Docker Desktop > Preferences > Resources

### データベース接続エラー

```bash
# ヘルスチェック状態を確認
docker compose ps

# 特定のデータベースログを確認
docker compose logs postgres-learning
```

## パフォーマンス最適化

### Docker Compose 設定

```yaml
services:
  service-name:
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 256M
```

### データベース接続プール

各サービスで適切な接続プール設定：

- 最小接続数: 5
- 最大接続数: 20
- アイドルタイムアウト: 60秒

## 今後の拡張予定

1. **Kubernetes 対応**
   - Helm チャートの作成
   - ConfigMap/Secret 管理

2. **CI/CD パイプライン**
   - GitHub Actions との統合
   - 自動テスト環境の構築

3. **本番環境対応**
   - Cloud Run へのデプロイ設定
   - マネージドサービスとの統合
