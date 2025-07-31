# Effect 開発ワークフロー

## 概要

このドキュメントでは、Effect プロジェクトでの開発作業の進め方、ローカル環境での開発フロー、テスト実行方法などを説明します。

## 開発環境のセットアップ

### 前提条件

- Rust (最新安定版)
- Docker Desktop
- Git
- Make コマンド

### 初期セットアップ

```bash
# リポジトリのクローン
git clone <repository-url>
cd effect

# 環境変数ファイルの作成
cp .env.example .env

# 開発環境の初期セットアップ
make setup
```

## 開発環境の起動

### インフラストラクチャのみ起動

```bash
# PostgreSQL × 8 + Redis を起動
make up-infra

# 起動状態の確認
make ps
```

### 開発ツールの起動

```bash
# pgAdmin, RedisInsight を起動
make up-tools
```

### モニタリングツールの起動

```bash
# Prometheus, Grafana, Jaeger を起動
make up-monitoring
```

## サービス開発フロー

### 1. サービスのビルド

```bash
# すべてのサービスをビルド
make build

# 特定のサービスのみビルド
make build-service SERVICE=learning-service
```

### 2. ホットリロード開発

各サービスディレクトリで cargo-watch を使用：

```bash
cd services/learning-service
cargo watch -x run
```

### 3. テスト実行

```bash
# すべてのテストを実行
make test

# 特定のサービスのテスト
make test-service SERVICE=learning-service

# 単体テストのみ
cargo test --lib

# 統合テストのみ
cargo test --test '*'
```

### 4. コード品質チェック

```bash
# フォーマット
make fmt

# フォーマットチェック（CI 用）
make fmt-check

# Clippy による静的解析
make lint

# 型チェックのみ
make check
```

## データベース操作

### データベースへの接続

```bash
# 特定サービスの DB に接続
make db-shell SERVICE=learning

# 利用可能なサービス:
# - event-store
# - learning
# - vocabulary
# - user
# - progress
# - algorithm
# - ai
# - saga
```

### Redis CLI への接続

```bash
make redis-cli
```

## Docker 操作

### ログの確認

```bash
# すべてのログを表示
make logs

# 特定サービスのログ
make logs-service SERVICE=learning-service
```

### コンテナの管理

```bash
# すべて停止
make down

# すべて削除（ボリューム含む）
make clean
```

## TDD（テスト駆動開発）の実践

### 1. 失敗するテストを書く

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_create_learning_session() {
        // Given
        let user_id = UserId::new();
        let item_count = 50;
        
        // When
        let session = LearningSession::create(user_id, item_count);
        
        // Then
        assert_eq!(session.item_count(), item_count);
        assert_eq!(session.status(), SessionStatus::NotStarted);
    }
}
```

### 2. 最小限の実装

```rust
impl LearningSession {
    pub fn create(user_id: UserId, item_count: u32) -> Self {
        Self {
            id: SessionId::new(),
            user_id,
            item_count,
            status: SessionStatus::NotStarted,
            // ...
        }
    }
}
```

### 3. リファクタリング

テストが通ったら、コードを改善します。

## デバッグ

### VS Code でのデバッグ設定

`.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Learning Service",
            "cargo": {
                "args": [
                    "build",
                    "--package=learning-service",
                    "--bin=learning-service"
                ],
                "filter": {
                    "name": "learning-service",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### ログレベルの設定

```bash
# 環境変数で設定
RUST_LOG=debug cargo run

# 特定のモジュールのみ
RUST_LOG=learning_service=debug,tower_http=trace cargo run
```

## CI/CD 連携

### ローカルでの CI チェック

```bash
# CI で実行されるすべてのチェック
make ci
```

### コミット前チェック

```bash
# フォーマット、リント、テストを実行
make pre-commit
```

## トラブルシューティング

### ビルドエラー

```bash
# キャッシュをクリア
make clean-cargo

# 依存関係を更新
make update
```

### ポート競合

`.env` ファイルでポートを変更：

```bash
POSTGRES_LEARNING_PORT=15433
```

### メモリ不足

Docker Desktop の設定でメモリを増やす：

- 推奨: 8GB 以上

## 開発のベストプラクティス

### 1. 小さな変更を頻繁にコミット

```bash
git add .
git commit -m "feat(learning): add session creation logic"
```

### 2. ブランチ戦略

```bash
# 機能開発
git checkout -b feature/learning-session-impl

# バグ修正
git checkout -b fix/session-validation-error
```

### 3. コードレビューの準備

- テストが通ることを確認
- フォーマットとリントをパス
- ドキュメントコメントを追加

### 4. パフォーマンステスト

```bash
# ベンチマークの実行
make bench
```

## 便利なコマンド

### 依存関係の確認

```bash
# 依存関係ツリーを表示
make tree

# 未使用の依存関係を検出
cargo machete
```

### セキュリティ監査

```bash
make audit
```

### バイナリサイズ分析

```bash
make bloat
```

## 次のステップ

1. 共通ライブラリ（shared/）の実装から開始
2. TDD でドメインモデルを実装
3. リポジトリ層の実装
4. gRPC サービスの実装
5. GraphQL スキーマの定義

詳細は [サービス構造ドキュメント](./service-structure.md) を参照してください。
