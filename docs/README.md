# effect - Event-Driven英単語学習アプリケーション

## 概要

effect は、CQRS/Event Sourcing パターンを活用した英単語学習アプリケーションです。
Rust で実装され、マイクロサービスアーキテクチャを採用しています。

## アーキテクチャ

- **パターン**: ヘキサゴナルアーキテクチャ + CQRS/Event Sourcing
- **言語**: Rust
- **マイクロサービス構成**:
  - API Gateway (GraphQL)
  - Command Service
  - Query Service
  - Saga Executor

## 主要機能

### Phase 1 (MVP)

- 単語登録・管理
- 基本的な学習セッション
- SM-2 アルゴリズムによる間隔反復
- 学習履歴のイベント記録

### Phase 2

- 学習パターン分析
- AI による単語推薦
- 進捗の可視化

## ドキュメント構成

- [architecture/](./architecture/) - アーキテクチャ設計
- [features/](./features/) - 機能仕様
- [api/](./api/) - API 仕様
- [development/](./development/) - 開発ガイド
- [deployment/](./deployment/) - デプロイメント

## Quick Start

```bash
# 開発環境のセットアップ
cd /Users/w/w/effect
cargo build

# テストの実行
cargo test

# 開発サーバーの起動
cargo run --bin api-gateway
```

## 技術スタック

- **言語**: Rust
- **Web Framework**: Axum
- **GraphQL**: async-graphql
- **Database**: PostgreSQL (Event Store)
- **Message Queue**: Google Cloud Pub/Sub
- **AI**: Gemini API
