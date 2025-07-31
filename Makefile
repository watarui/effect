# Effect Makefile - 開発用コマンド集

.PHONY: help
help: ## ヘルプを表示
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

# ===========================================
# Docker 関連
# ===========================================

.PHONY: up
up: ## すべてのサービスを起動
	docker compose up -d

.PHONY: down
down: ## すべてのサービスを停止
	docker compose down

.PHONY: up-infra
up-infra: ## インフラのみ起動（PostgreSQL × 8, Redis）
	docker compose up -d \
		postgres-event-store \
		postgres-learning \
		postgres-vocabulary \
		postgres-user \
		postgres-progress \
		postgres-algorithm \
		postgres-ai \
		postgres-saga \
		redis

.PHONY: up-services
up-services: ## マイクロサービスを起動
	docker compose up -d

.PHONY: up-tools
up-tools: ## 開発ツールを起動
	docker compose --profile tools up -d

.PHONY: up-monitoring
up-monitoring: ## モニタリングツールを起動
	docker compose --profile monitoring up -d

.PHONY: logs
logs: ## すべてのログを表示
	docker compose logs -f

.PHONY: logs-service
logs-service: ## 特定のサービスのログを表示（例: make logs-service SERVICE=learning-service）
	docker compose logs -f $(SERVICE)

.PHONY: ps
ps: ## コンテナの状態を表示
	docker compose ps

.PHONY: clean
clean: ## すべてのコンテナとボリュームを削除
	docker compose down -v

# ===========================================
# ビルド関連
# ===========================================

.PHONY: build
build: ## すべてのサービスをビルド
	cargo build --workspace

.PHONY: build-release
build-release: ## リリースビルド
	cargo build --workspace --release

.PHONY: build-service
build-service: ## 特定のサービスをビルド（例: make build-service SERVICE=learning-service）
	cargo build -p $(SERVICE)

.PHONY: test
test: ## すべてのテストを実行
	cargo test --workspace

.PHONY: test-service
test-service: ## 特定のサービスのテストを実行（例: make test-service SERVICE=learning-service）
	cargo test -p $(SERVICE)

.PHONY: lint
lint: ## Clippy でリントチェック
	cargo clippy --workspace -- -D warnings

.PHONY: fmt
fmt: ## コードフォーマット
	cargo fmt --all

.PHONY: fmt-check
fmt-check: ## フォーマットチェック
	cargo fmt --all -- --check

.PHONY: check
check: ## 型チェック
	cargo check --workspace

.PHONY: audit
audit: ## セキュリティ監査
	cargo audit

# ===========================================
# データベース関連
# ===========================================

.PHONY: db-migrate
db-migrate: ## データベースマイグレーション実行
	@echo "マイグレーションは未実装です"

.PHONY: db-reset
db-reset: ## データベースをリセット（各サービスのデータベースを個別にリセット）
	@echo "各データベースをリセットします..."
	@echo "注意: 現在は各サービスが独立したデータベースを持っています"

.PHONY: db-shell
db-shell: ## PostgreSQL シェルに接続（サービスを指定: make db-shell SERVICE=learning）
	@if [ -z "$(SERVICE)" ]; then \
		echo "使用法: make db-shell SERVICE=learning"; \
		echo "利用可能なサービス: event-store, learning, vocabulary, user, progress, algorithm, ai, saga"; \
	else \
		docker compose exec postgres-$(SERVICE) psql -U effect -d $$(echo $(SERVICE) | sed 's/-/_/g')_db; \
	fi

.PHONY: redis-cli
redis-cli: ## Redis CLI に接続
	docker compose exec redis redis-cli

# ===========================================
# 開発環境セットアップ
# ===========================================

.PHONY: setup
setup: ## 開発環境の初期セットアップ
	@echo "=== 開発環境セットアップ ==="
	@echo "1. .env ファイルを作成..."
	@if [ ! -f .env ]; then cp .env.example .env; echo ".env ファイルを作成しました"; else echo ".env ファイルは既に存在します"; fi
	@echo "2. Rust ツールチェインの確認..."
	@rustc --version
	@cargo --version
	@echo "3. 必要なツールのインストール..."
	@cargo install cargo-watch cargo-audit || true
	@echo "4. Git フックの設定..."
	@echo "セットアップ完了！"

.PHONY: dev
dev: ## 開発環境を起動（インフラ + ホットリロード）
	@make up-infra
	@echo "開発サーバーを起動するには、各サービスディレクトリで 'cargo watch -x run' を実行してください"

# ===========================================
# プロトコルバッファ関連
# ===========================================

.PHONY: proto-gen
proto-gen: ## Protocol Buffers からコードを生成
	@echo "Protocol Buffers のコード生成は未実装です"

# ===========================================
# ドキュメント関連
# ===========================================

.PHONY: doc
doc: ## ドキュメントを生成して開く
	cargo doc --workspace --no-deps --open

.PHONY: doc-deps
doc-deps: ## 依存関係を含むドキュメントを生成
	cargo doc --workspace --open

# ===========================================
# CI/CD 関連
# ===========================================

.PHONY: ci
ci: fmt-check lint test ## CI で実行するチェック

.PHONY: pre-commit
pre-commit: fmt lint test ## コミット前のチェック

# ===========================================
# ユーティリティ
# ===========================================

.PHONY: clean-cargo
clean-cargo: ## Cargo のキャッシュをクリーン
	cargo clean

.PHONY: update
update: ## 依存関係を更新
	cargo update

.PHONY: tree
tree: ## 依存関係ツリーを表示
	cargo tree

.PHONY: bloat
bloat: ## バイナリサイズの分析
	cargo bloat --release

.PHONY: bench
bench: ## ベンチマークを実行
	cargo bench --workspace

# デフォルトターゲット
.DEFAULT_GOAL := help
