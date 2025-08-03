# Rust プロジェクト命名規則ガイドライン

## 概要

このドキュメントは、Effect プロジェクトにおける Rust Edition 2024 準拠の命名規則を定義します。
2025年2月リリース予定の Rust 1.85（Edition 2024）の推奨事項に基づいています。

## 基本方針

すべての名前は `snake_case`（アンダースコア区切り）で統一します。

## 命名規則詳細

### 1. ディレクトリ名

**規則**: `snake_case` を使用

```
✅ 正しい例:
services/ai_service
services/api_gateway
shared/common_types
shared/cross_cutting

❌ 誤った例:
services/ai-service
services/api-gateway
shared/common-types
```

### 2. パッケージ名（Cargo.toml）

**規則**: `snake_case` を使用

```toml
# ✅ 正しい例
[package]
name = "ai_service"

# ❌ 誤った例
[package]
name = "ai-service"
```

### 3. クレート名

**規則**: `snake_case` を使用（RFC 430 準拠）

```rust
// ✅ 正しい例
extern crate ai_service;
use shared_kernel::UserId;

// ❌ 誤った例
extern crate ai-service;  // コンパイルエラー
```

### 4. モジュール名・ファイル名

**規則**: `snake_case` を使用

```
✅ 正しい例:
src/user_service.rs
src/value_objects.rs
src/event_store.rs

❌ 誤った例:
src/user-service.rs
src/value-objects.rs
src/event-store.rs
```

## 移行計画

### 現状

#### 完了した移行

```
services/
├── ai_service/          ✓
├── api_gateway/         ✓
├── algorithm_service/   ✓
├── saga_orchestrator/   ✓
├── event_processor/     ✓
├── learning_service/    ✓
├── user_service/        ✓
├── progress_service/    ✓
└── vocabulary_service/  ✓

shared/
├── common-types/        削除済み
├── domain_events/       ✓ (移行済み、Proto 分散は保留)
├── cross_cutting/       ✓ (既に正しい)
│   ├── cache/          ✓ (新規作成)
│   └── config/         ✓ (新規作成)
└── infrastructure/      (サブディレクトリのみ残存)
```

#### 未完了の移行

```
# パッケージ名の更新が必要
shared-kernel         → shared_kernel
shared-database       → shared_database
shared-repository     → shared_repository
shared-event-bus      → shared_event_bus
shared-event-store    → shared_event_store
```

### 移行手順

1. **ディレクトリ名の変更**

   ```bash
   # 例
   mv services/ai-service services/ai_service
   ```

2. **Cargo.toml の更新**

   ```toml
   # Before
   name = "ai-service"
   
   # After
   name = "ai_service"
   ```

3. **ワークスペース設定の更新**

   ```toml
   [workspace]
   members = [
     "services/ai_service",
     "services/api_gateway",
     # ...
   ]
   ```

4. **依存関係の更新**

   ```toml
   [dependencies]
   shared_kernel = { path = "../shared/kernel" }
   shared_event_store = { path = "../shared/infrastructure/event_store" }
   ```

5. **Docker 関連ファイルの更新**
   - Dockerfile のパス
   - docker-compose.yml のビルドコンテキスト

6. **CI/CD パイプラインの更新**
   - GitHub Actions のパス
   - ビルドスクリプトのパス

## 根拠

### RFC 430（Finalizing Rust Naming Conventions）

- クレート名は `snake_case` を使用（単一単語が望ましい）
- モジュール名は `snake_case` を使用

### Rust のモジュールシステムとの整合性

- ファイルシステムのパスがモジュールパスに対応
- `mod user_service;` は `user_service.rs` または `user_service/mod.rs` を探す
- ハイフンを含むファイル名はモジュール名として使用不可

### Cargo の自動変換機能の問題

- Cargo はパッケージ名のハイフンを自動的にアンダースコアに変換
- この暗黙的な変換は混乱を招く可能性がある
- 最初から `snake_case` を使用することで一貫性を保つ

## 例外事項

### crates.io での公開

- crates.io では歴史的に `kebab-case` が一般的
- 公開クレートの場合は、エコシステムの慣習に従うことも検討

### 既存の外部依存関係

- 外部クレートの命名規則は変更不可
- `async-trait`、`sqlx-core` などはそのまま使用

## 参考資料

- [RFC 430: Finalizing Rust Naming Conventions](https://rust-lang.github.io/rfcs/0430-finalizing-naming-conventions.html)
- [Rust API Guidelines - Naming](https://rust-lang.github.io/api-guidelines/naming.html)
- [The Rust Programming Language - Module System](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)

## 更新履歴

- 2025-08-03: 初版作成（Rust Edition 2024 対応）
- 2025-08-03: ディレクトリ名とパッケージ名の一部を snake_case に移行
