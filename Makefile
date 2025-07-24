.PHONY: help dev dev-infra dev-services down clean build test fmt lint check migrate setup install-tools

# デフォルトターゲット
help:
	@echo "使用可能なコマンド:"
	@echo "  make dev           - 開発環境全体を起動 (インフラ + サービス)"
	@echo "  make dev-infra     - インフラのみ起動 (PostgreSQL, Pub/Sub)"
	@echo "  make dev-services  - サービスのみ起動"
	@echo "  make down          - 全てのコンテナを停止"
	@echo "  make clean         - コンテナ、ボリューム、キャッシュを削除"
	@echo "  make build         - 全サービスをビルド"
	@echo "  make test          - 全テストを実行"
	@echo "  make fmt           - コードフォーマット"
	@echo "  make lint          - リントチェック"
	@echo "  make check         - ビルドチェック"
	@echo "  make migrate       - データベースマイグレーション実行"
	@echo "  make setup         - 初期セットアップ"
	@echo "  make install-tools - 開発ツールのインストール"

# 開発環境の起動
dev: dev-infra
	@echo "🚀 開発環境を起動しています..."
	docker compose --profile services up -d
	@echo "✅ 開発環境が起動しました"
	@echo "📊 pgAdmin: http://localhost:5050"
	@echo "🌐 API Gateway: http://localhost:8080"
	@echo "📝 GraphQL Playground: http://localhost:8080/playground"

# インフラのみ起動
dev-infra:
	@echo "🏗️  インフラを起動しています..."
	docker compose up -d postgres pubsub-emulator
	@echo "⏳ PostgreSQL の起動を待っています..."
	@docker compose exec -T postgres pg_isready -U effect || sleep 5
	@echo "✅ インフラが起動しました"

# サービスのみ起動
dev-services:
	@echo "🚀 サービスを起動しています..."
	docker compose --profile services up -d api-gateway command-service query-service saga-executor
	@echo "✅ サービスが起動しました"

# 全て停止
down:
	@echo "🛑 全てのコンテナを停止しています..."
	docker compose --profile services --profile tools down
	@echo "✅ 停止しました"

# クリーンアップ
clean: down
	@echo "🧹 クリーンアップを実行しています..."
	docker compose --profile services --profile tools down -v
	rm -rf target/
	rm -rf services/*/target/
	rm -rf shared/*/target/
	@echo "✅ クリーンアップが完了しました"

# ビルド
build:
	@echo "🔨 全サービスをビルドしています..."
	cargo build --all
	@echo "✅ ビルドが完了しました"

# テスト実行
test:
	@echo "🧪 テストを実行しています..."
	cargo test --all -- --nocapture
	@echo "✅ テストが完了しました"

# フォーマット
fmt:
	@echo "✨ コードをフォーマットしています..."
	cargo fmt --all
	@echo "✅ フォーマットが完了しました"

# リントチェック
lint:
	@echo "🔍 リントチェックを実行しています..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ リントチェックが完了しました"

# ビルドチェック
check:
	@echo "🔍 ビルドチェックを実行しています..."
	cargo check --all
	@echo "✅ ビルドチェックが完了しました"

# データベースマイグレーション
migrate:
	@echo "🗄️  マイグレーションを実行しています..."
	@echo "⚠️  sqlx-cli がインストールされていることを確認してください"
	@echo "実行: cargo install sqlx-cli --no-default-features --features postgres"
	# sqlx migrate run --database-url postgresql://effect:effect_password@localhost:5432/effect_db  # pragma: allowlist secret
	@echo "✅ マイグレーションが完了しました (実装後に有効化)"

# 初期セットアップ
setup: install-tools
	@echo "🔧 初期セットアップを実行しています..."
	@if [ ! -f .env ]; then cp .env.example .env && echo "✅ .env ファイルを作成しました"; fi
	@echo "📦 pre-commit をインストールしています..."
	pre-commit install
	@echo "✅ セットアップが完了しました"

# 開発ツールのインストール
install-tools:
	@echo "🔧 開発ツールをインストールしています..."
	@echo "Rust ツール:"
	rustup component add rustfmt clippy
	@echo ""
	@echo "以下のツールを手動でインストールしてください:"
	@echo "  - sqlx-cli: cargo install sqlx-cli --no-default-features --features postgres"
	@echo "  - cargo-watch: cargo install cargo-watch"
	@echo "  - pre-commit: pip install pre-commit"
	@echo "  - docker compose: https://docs.docker.com/compose/install/"

# ログ表示
logs:
	docker compose logs -f

# PostgreSQL に接続
psql:
	docker compose exec postgres psql -U effect -d effect_db

# pgAdmin を起動
pgadmin:
	docker compose --profile tools up -d pgadmin
	@echo "📊 pgAdmin: http://localhost:5050"

# サービスのステータス確認
status:
	@echo "📊 サービスステータス:"
	@docker compose ps

# ヘルスチェック
health:
	@echo "🏥 ヘルスチェック:"
	@curl -s http://localhost:8080/health || echo "API Gateway: ❌ 未起動"
	@curl -s http://localhost:8081/health || echo "Command Service: ❌ 未起動"
	@curl -s http://localhost:8082/health || echo "Query Service: ❌ 未起動"
	@curl -s http://localhost:8083/health || echo "Saga Executor: ❌ 未起動"

# 開発用ウォッチモード
watch:
	cargo watch -x "check --all" -x "test --all" -x "clippy --all"

# カバレッジレポート生成
coverage:
	@echo "📊 カバレッジレポートを生成しています..."
	@echo "⚠️  cargo-tarpaulin がインストールされていることを確認してください"
	@echo "実行: cargo install cargo-tarpaulin"
	# cargo tarpaulin --out Html --output-dir target/coverage

# ベンチマーク実行
bench:
	@echo "⚡ ベンチマークを実行しています..."
	cargo bench --all

# ドキュメント生成
doc:
	@echo "📚 ドキュメントを生成しています..."
	cargo doc --all --no-deps --open

# セキュリティ監査
audit:
	@echo "🔒 セキュリティ監査を実行しています..."
	@echo "⚠️  cargo-audit がインストールされていることを確認してください"
	@echo "実行: cargo install cargo-audit"
	cargo audit
