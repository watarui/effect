# 開発標準

## 命名規則

### 基本方針

すべての名前は `snake_case`（アンダースコア区切り）で統一します。
（Rust Edition 2024 準拠）

### 規則一覧

| 対象 | 規則 | 例 |
|------|------|-----|
| ディレクトリ名 | `snake_case` | `services/ai_service` |
| パッケージ名（Cargo.toml） | `snake_case` | `name = "ai_service"` |
| クレート名 | `snake_case` | `extern crate ai_service;` |
| モジュール名・ファイル名 | `snake_case` | `event_store.rs` |
| 型（構造体、トレイト、列挙型） | `PascalCase` | `struct VocabularyItem` |
| 関数・メソッド | `snake_case` | `fn calculate_difficulty()` |
| 定数 | `SCREAMING_SNAKE_CASE` | `const MAX_RETRY_COUNT: u32 = 3;` |
| 変数 | `snake_case` | `let item_count = 10;` |

### 参考資料

- [RFC 430: Rust Naming Conventions](https://rust-lang.github.io/rfcs/0430-finalizing-naming-conventions.html)
- [Rust API Guidelines - Naming](https://rust-lang.github.io/api-guidelines/naming.html)

## プロジェクト構造

### 基本レイアウト

```
effect/
├── services/                    # マイクロサービス
│   ├── vocabulary_service/      # 各サービスは独立したクレート
│   ├── learning_service/
│   └── ...
├── shared/                      # 共有ライブラリ
│   ├── domain_events/          # ドメインイベント定義
│   └── infrastructure/         # 共通インフラ
└── docs/                       # ドキュメント
```

### サービス内部構造（ヘキサゴナルアーキテクチャ）

```
service_name/
├── src/
│   ├── main.rs                 # エントリーポイント
│   ├── config.rs               # 設定
│   ├── domain/                 # ドメイン層（ビジネスロジック）
│   │   ├── mod.rs
│   │   ├── aggregates/         # 集約
│   │   ├── events/            # ドメインイベント
│   │   └── value_objects/     # 値オブジェクト
│   ├── application/           # アプリケーション層（ユースケース）
│   │   ├── mod.rs
│   │   ├── commands/          # コマンドハンドラ
│   │   └── queries/           # クエリハンドラ
│   ├── infrastructure/        # インフラ層（技術的実装）
│   │   ├── mod.rs
│   │   ├── repositories/      # リポジトリ実装
│   │   └── grpc/             # gRPC サービス
│   └── ports/                # ポート（インターフェース）
│       ├── mod.rs
│       └── repositories.rs    # リポジトリトレイト
└── tests/                    # 統合テスト
```

## コーディング規約

### 基本原則

1. **公式スタイルガイドに従う**: `rustfmt` と `clippy` を使用
2. **明示的なエラーハンドリング**: `Result` 型と `thiserror` を活用
3. **テスト駆動開発（TDD）**: Red-Green-Refactor サイクルの遵守
4. **依存性注入**: トレイトを使用した疎結合設計

### エラーハンドリング

```rust
// thiserror を使用したドメインエラー定義
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Item not found: {id}")]
    ItemNotFound { id: ItemId },
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
}
```

### テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_create_item() {
        // Given
        let mut mock_repo = MockItemRepository::new();
        mock_repo.expect_save()
            .times(1)
            .returning(|_| Ok(()));

        // When
        let use_case = CreateItemUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(/* ... */).await;

        // Then
        assert!(result.is_ok());
    }
}
```

### 非同期プログラミング

- `tokio` ランタイムを使用
- `async`/`await` を適切に活用
- 並行処理には `tokio::spawn` を使用

### ログとトレース

- `tracing` クレートを使用
- 構造化ログで情報を記録
- OpenTelemetry 対応

## 開発ツール

### 必須ツール

- `rustfmt`: コードフォーマッタ
- `clippy`: リンター
- `cargo-watch`: ファイル変更監視
- `cargo-nextest`: 高速テストランナー

### 推奨ツール

- `cargo-machete`: 未使用の依存関係検出
- `cargo-audit`: セキュリティ脆弱性チェック
- `cargo-outdated`: 依存関係の更新確認
