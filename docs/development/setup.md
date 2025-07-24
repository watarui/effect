# 開発環境セットアップ

## 必要なツール

### 必須

- **Rust**: 1.75以上
- **PostgreSQL**: 15以上
- **Docker & Docker Compose**: 最新版
- **Google Cloud SDK**: Pub/Sub エミュレータ用

### 推奨

- **cargo-watch**: ファイル変更の自動検知
- **sqlx-cli**: データベースマイグレーション
- **grpcurl**: gRPC デバッグ用

## セットアップ手順

### 1. リポジトリのクローン

```bash
git clone <repository-url>
cd effect
```

### 2. Rust のインストール

```bash
# rustup がない場合
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 最新版へ更新
rustup update

# 必要なコンポーネント
rustup component add rustfmt clippy
```

### 3. 開発ツールのインストール

```bash
# cargo-watch
cargo install cargo-watch

# sqlx-cli
cargo install sqlx-cli --features postgres

# protobuf コンパイラ
brew install protobuf  # macOS
# または
sudo apt install protobuf-compiler  # Ubuntu
```

### 4. Docker 環境の準備

```bash
# Docker Compose ファイルの作成
cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: effect
      POSTGRES_PASSWORD: effect_pass
      POSTGRES_DB: effect_db
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  pubsub-emulator:
    image: gcr.io/google.com/cloudsdktool/cloud-sdk:emulators
    command: gcloud beta emulators pubsub start --host-port=0.0.0.0:8085
    ports:
      - "8085:8085"

volumes:
  postgres_data:
EOF

# サービスの起動
docker compose up -d
```

### 5. 環境変数の設定

```bash
# .env ファイルの作成
cat > .env << 'EOF'
# Database
DATABASE_URL=postgresql://effect:effect_pass@localhost:5432/effect_db  # pragma: allowlist secret

# Pub/Sub
PUBSUB_EMULATOR_HOST=localhost:8085
PUBSUB_PROJECT_ID=effect-local

# Application
RUST_LOG=info,effect=debug
API_PORT=8080
GRPC_PORT=50051

# AI Service
GEMINI_API_KEY=your-api-key-here
EOF
```

### 6. データベースのセットアップ

```bash
# データベースの作成
createdb -h localhost -U effect effect_db

# マイグレーションの実行
sqlx database create
sqlx migrate run
```

### 7. 依存関係のインストール

```bash
# すべての依存関係をダウンロード
cargo build
```

## 開発用スクリプト

### Makefile の作成

```makefile
.PHONY: help
help:
 @echo "Available commands:"
 @echo "  make dev        - Start development environment"
 @echo "  make test       - Run all tests"
 @echo "  make fmt        - Format code"
 @echo "  make lint       - Run linter"
 @echo "  make migrate    - Run database migrations"
 @echo "  make clean      - Clean build artifacts"

.PHONY: dev
dev:
 docker compose up -d
 cargo watch -x "run --bin api-gateway"

.PHONY: test
test:
 cargo test --all

.PHONY: fmt
fmt:
 cargo fmt --all

.PHONY: lint
lint:
 cargo clippy --all -- -D warnings

.PHONY: migrate
migrate:
 sqlx migrate run

.PHONY: clean
clean:
 cargo clean
 docker compose down -v
```

## サービスの起動

### 個別サービスの起動

```bash
# API Gateway
cargo run --bin api-gateway

# Command Service
cargo run --bin command-service

# Query Service
cargo run --bin query-service

# Saga Executor
cargo run --bin saga-executor
```

### すべてのサービスを起動

```bash
# 並列実行スクリプト
cat > scripts/start-all.sh << 'EOF'
#!/bin/bash
trap 'kill $(jobs -p)' EXIT

cargo run --bin api-gateway &
cargo run --bin command-service &
cargo run --bin query-service &
cargo run --bin saga-executor &

wait
EOF

chmod +x scripts/start-all.sh
./scripts/start-all.sh
```

## 動作確認

### GraphQL Playground

```bash
# ブラウザで開く
open http://localhost:8080/playground
```

### ヘルスチェック

```bash
# API Gateway
curl http://localhost:8080/health

# gRPC サービス
grpcurl -plaintext localhost:50051 effect.health.v1.Health/Check
```

### Pub/Sub エミュレータの確認

```bash
# トピックの作成
curl -X PUT http://localhost:8085/v1/projects/effect-local/topics/domain-events

# サブスクリプションの作成
curl -X PUT http://localhost:8085/v1/projects/effect-local/subscriptions/query-service-sub \
  -H "Content-Type: application/json" \
  -d '{
    "topic": "projects/effect-local/topics/domain-events"
  }'
```

## トラブルシューティング

### PostgreSQL 接続エラー

```bash
# 接続テスト
psql -h localhost -U effect -d effect_db

# Docker ログ確認
docker compose logs postgres
```

### Pub/Sub エミュレータエラー

```bash
# 環境変数の確認
echo $PUBSUB_EMULATOR_HOST

# エミュレータの再起動
docker compose restart pubsub-emulator
```

### ビルドエラー

```bash
# クリーンビルド
cargo clean
cargo build

# 依存関係の更新
cargo update
```

## IDE 設定

### VS Code

推奨拡張機能:

- rust-analyzer
- Even Better TOML
- GraphQL
- Docker

### IntelliJ IDEA

プラグイン:

- Rust
- GraphQL
- Docker Integration
