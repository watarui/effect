# Learning Service 用 Dockerfile
# 各サービスが完全に独立したビルド環境を持つ

# ===============================
# ベースイメージの定義
# ===============================
ARG RUST_VERSION=slim
ARG DEBIAN_VERSION=bookworm

# ===============================
# ビルドステージ
# ===============================
FROM rust:1.89-slim-bookworm AS builder

# ビルドに必要なパッケージをインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# ワークディレクトリを設定
WORKDIR /app

# Cargo.toml と Cargo.lock をコピー（依存関係のキャッシュ用）
COPY Cargo.toml Cargo.lock ./
COPY services/api-gateway/Cargo.toml ./services/api-gateway/
COPY services/learning-service/Cargo.toml ./services/learning-service/
COPY services/vocabulary-service/Cargo.toml ./services/vocabulary-service/
COPY services/user-service/Cargo.toml ./services/user-service/
COPY services/progress-service/Cargo.toml ./services/progress-service/
COPY services/algorithm-service/Cargo.toml ./services/algorithm-service/
COPY services/ai-service/Cargo.toml ./services/ai-service/
COPY services/event-processor/Cargo.toml ./services/event-processor/
COPY services/saga-orchestrator/Cargo.toml ./services/saga-orchestrator/
COPY shared/common-types/Cargo.toml ./shared/common-types/
COPY shared/domain-events/Cargo.toml ./shared/domain-events/

# ダミーファイルを作成して依存関係をビルド
RUN mkdir -p services/api-gateway/src && echo "fn main() {}" > services/api-gateway/src/main.rs
RUN mkdir -p services/learning-service/src && echo "fn main() {}" > services/learning-service/src/main.rs
RUN mkdir -p services/vocabulary-service/src && echo "" > services/vocabulary-service/src/lib.rs
RUN mkdir -p services/user-service/src && echo "" > services/user-service/src/lib.rs
RUN mkdir -p services/progress-service/src && echo "" > services/progress-service/src/lib.rs
RUN mkdir -p services/algorithm-service/src && echo "" > services/algorithm-service/src/lib.rs
RUN mkdir -p services/ai-service/src && echo "" > services/ai-service/src/lib.rs
RUN mkdir -p services/event-processor/src && echo "fn main() {}" > services/event-processor/src/main.rs
RUN mkdir -p services/saga-orchestrator/src && echo "fn main() {}" > services/saga-orchestrator/src/main.rs
RUN mkdir -p shared/common-types/src && echo "" > shared/common-types/src/lib.rs
RUN mkdir -p shared/domain-events/src && echo "" > shared/domain-events/src/lib.rs

# 依存関係をビルド
RUN cargo build --release -p learning-service

# 実際のソースコードをコピー
COPY protos ./protos
COPY shared ./shared
COPY services/learning-service ./services/learning-service

# ダミーファイルを削除してリビルド
RUN rm -rf services/*/src shared/*/src && \
    touch services/learning-service/src/main.rs && \
    cargo build --release -p learning-service

# ===============================
# 開発用ステージ
# ===============================
FROM rust:1.89-slim-bookworm AS development

# 開発に必要なツールをインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# cargo-watch をインストール
RUN cargo install cargo-watch

WORKDIR /app

# 開発時はボリュームマウントを使用するため、COPYは不要

# ポートを公開
EXPOSE 50051

# 開発時はホットリロードを有効化
CMD ["cargo", "watch", "-x", "run", "-p", "learning-service"]

# ===============================
# 本番用ステージ
# ===============================
FROM debian:${DEBIAN_VERSION}-slim AS production

# 実行に必要な最小限のパッケージをインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 非rootユーザーを作成
RUN useradd -m -u 1001 -s /bin/bash appuser

# ビルド成果物をコピー
COPY --from=builder /app/target/release/learning-service /usr/local/bin/learning-service

# 所有権を変更
RUN chown appuser:appuser /usr/local/bin/learning-service

# 非rootユーザーに切り替え
USER appuser

# ポートを公開
EXPOSE 50051

# ヘルスチェック（gRPC health check）
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD grpc_health_probe -addr=:50051 || exit 1

# アプリケーションを実行
CMD ["learning-service"]