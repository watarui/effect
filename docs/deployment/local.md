# ローカル環境構築ガイド

## 概要

このガイドでは、effect アプリケーションをローカル環境で動作させる手順を説明します。

## 前提条件

- Docker Desktop がインストールされていること
- Rust 1.75 以上がインストールされていること
- PostgreSQL クライアント（psql）がインストールされていること

## クイックスタート

### 1. 環境構築スクリプト

```bash
#!/bin/bash
# scripts/setup-local.sh

set -e

echo "🚀 Setting up effect local environment..."

# Docker サービスの起動
echo "📦 Starting Docker services..."
docker compose up -d

# 環境変数の設定
echo "🔧 Setting environment variables..."
export DATABASE_URL="postgresql://effect:effect_pass@localhost:5432/effect_db"  # pragma: allowlist secret
export PUBSUB_EMULATOR_HOST="localhost:8085"

# データベースの準備を待つ
echo "⏳ Waiting for PostgreSQL..."
until pg_isready -h localhost -p 5432 -U effect; do
  sleep 1
done

# マイグレーションの実行
echo "🗄️ Running database migrations..."
sqlx database create
sqlx migrate run

# Pub/Sub トピックの作成
echo "📨 Creating Pub/Sub topics..."
curl -X PUT http://localhost:8085/v1/projects/effect-local/topics/domain-events
curl -X PUT http://localhost:8085/v1/projects/effect-local/topics/saga-events

# サブスクリプションの作成
echo "📮 Creating subscriptions..."
curl -X PUT http://localhost:8085/v1/projects/effect-local/subscriptions/query-service-sub \
  -H "Content-Type: application/json" \
  -d '{"topic": "projects/effect-local/topics/domain-events"}'

curl -X PUT http://localhost:8085/v1/projects/effect-local/subscriptions/saga-executor-sub \
  -H "Content-Type: application/json" \
  -d '{"topic": "projects/effect-local/topics/saga-events"}'

echo "✅ Local environment setup complete!"
```

### 2. 実行

```bash
# 実行権限を付与
chmod +x scripts/setup-local.sh

# セットアップ実行
./scripts/setup-local.sh

# サービスの起動
make dev
```

## 詳細設定

### Docker Compose 設定

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:15-alpine
    container_name: effect_postgres
    environment:
      POSTGRES_USER: effect
      POSTGRES_PASSWORD: effect_pass
      POSTGRES_DB: effect_db
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./scripts/init.sql:/docker-entrypoint-initdb.d/init.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U effect"]
      interval: 10s
      timeout: 5s
      retries: 5

  pubsub-emulator:
    image: gcr.io/google.com/cloudsdktool/cloud-sdk:emulators
    container_name: effect_pubsub
    command: gcloud beta emulators pubsub start --host-port=0.0.0.0:8085
    ports:
      - "8085:8085"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8085"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    container_name: effect_redis
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:
```

### 環境変数ファイル

```bash
# .env.local
# Database
DATABASE_URL=postgresql://effect:effect_pass@localhost:5432/effect_db  # pragma: allowlist secret
TEST_DATABASE_URL=postgresql://effect:effect_pass@localhost:5432/effect_test_db  # pragma: allowlist secret

# Pub/Sub
PUBSUB_EMULATOR_HOST=localhost:8085
PUBSUB_PROJECT_ID=effect-local

# Redis
REDIS_URL=redis://localhost:6379

# Application
RUST_LOG=debug,hyper=info,sqlx=info
RUST_BACKTRACE=1

# Service Ports
API_GATEWAY_PORT=8080
COMMAND_SERVICE_PORT=50051
QUERY_SERVICE_PORT=50052
SAGA_EXECUTOR_PORT=50053

# GraphQL
GRAPHQL_PLAYGROUND=true
GRAPHQL_INTROSPECTION=true

# AI Services
GEMINI_API_KEY=your-api-key-here
GEMINI_MODEL=gemini-pro

# Security (開発用)
JWT_SECRET=development-secret-key-change-in-production
CORS_ORIGIN=http://localhost:3000
```

## サービス起動

### 個別起動

```bash
# Terminal 1: API Gateway
cargo run --bin api-gateway

# Terminal 2: Command Service
cargo run --bin command-service

# Terminal 3: Query Service
cargo run --bin query-service

# Terminal 4: Saga Executor
cargo run --bin saga-executor
```

### 統合起動スクリプト

```bash
#!/bin/bash
# scripts/start-services.sh

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# プロセスIDを保存する配列
declare -a PIDS

# 終了時にすべてのプロセスを停止
cleanup() {
    echo -e "\n${YELLOW}Stopping all services...${NC}"
    for pid in "${PIDS[@]}"; do
        kill $pid 2>/dev/null
    done
    exit
}

trap cleanup SIGINT SIGTERM

# サービス起動関数
start_service() {
    local name=$1
    local bin=$2

    echo -e "${GREEN}Starting $name...${NC}"
    cargo run --bin $bin &
    PIDS+=($!)
}

# メイン処理
echo -e "${GREEN}Starting effect services...${NC}\n"

# 環境変数の読み込み
source .env.local

# サービスの起動
start_service "API Gateway" "api-gateway"
sleep 2
start_service "Command Service" "command-service"
sleep 2
start_service "Query Service" "query-service"
sleep 2
start_service "Saga Executor" "saga-executor"

echo -e "\n${GREEN}All services started!${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}\n"

# サービスが終了するまで待機
wait
```

## 動作確認

### ヘルスチェック

```bash
# API Gateway
curl http://localhost:8080/health

# GraphQL Playground
open http://localhost:8080/playground

# gRPC サービス
grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check
```

### サンプルリクエスト

```graphql
# GraphQL Playground で実行
mutation CreateWord {
  createWord(input: {
    text: "example"
    meaning: "例"
    difficulty: 3
    category: "General"
    tags: ["test"]
  }) {
    word {
      id
      text
      meaning
    }
  }
}

query GetWords {
  words(limit: 10) {
    edges {
      node {
        id
        text
        meaning
        difficulty
      }
    }
    totalCount
  }
}
```

## トラブルシューティング

### PostgreSQL 接続エラー

```bash
# PostgreSQL の状態確認
docker compose ps postgres
docker compose logs postgres

# 接続テスト
psql -h localhost -U effect -d effect_db -c "SELECT 1"

# 再起動
docker compose restart postgres
```

### Pub/Sub エミュレータの問題

```bash
# エミュレータの状態確認
curl http://localhost:8085/v1/projects/effect-local/topics

# ログ確認
docker compose logs pubsub-emulator

# トピックの再作成
./scripts/setup-pubsub.sh
```

### ポート競合

```bash
# 使用中のポート確認
lsof -i :8080
lsof -i :5432
lsof -i :8085

# 別のポートを使用
API_GATEWAY_PORT=8081 cargo run --bin api-gateway
```

## 開発用ツール

### データベース管理

```bash
# pgAdmin の起動
docker run -d \
  --name pgadmin \
  -p 5050:80 \
  -e PGADMIN_DEFAULT_EMAIL=admin@effect.local \
  -e PGADMIN_DEFAULT_PASSWORD=admin \
  dpage/pgadmin4

# アクセス
open http://localhost:5050
```

### ログ監視

```bash
# すべてのログを表示
docker compose logs -f

# 特定のサービスのログ
docker compose logs -f postgres

# Rust アプリケーションのログレベル変更
RUST_LOG=trace cargo run --bin api-gateway
```

## クリーンアップ

```bash
# サービスの停止
docker compose down

# データも含めて削除
docker compose down -v

# ビルドキャッシュのクリア
cargo clean
```
