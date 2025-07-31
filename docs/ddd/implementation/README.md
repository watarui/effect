# Effect 実装フェーズドキュメント

## 概要

このディレクトリには、Effect プロジェクトの実装フェーズに関するドキュメントが含まれています。DDD 設計フェーズで定義したアーキテクチャとドメインモデルを、実際のコードに落とし込むための詳細な実装ガイドです。

## 実装状況（2025-07-31）

### ✅ 完了した実装

1. **マイクロサービスインフラストラクチャ**
   - 9 つのマイクロサービスの基本構造
   - Docker Compose による統合開発環境
   - 完全分離型データベース構成（8 PostgreSQL + Redis）
   - 開発支援ツールの統合

2. **プロジェクト構造**
   - Cargo workspace による monorepo 構成
   - 各サービスの基本的なディレクトリ構造
   - Makefile による開発コマンド整備

### 🚧 進行中の実装

1. **共有ライブラリ（shared/）**
   - common-types: 共通型定義
   - domain-events: ドメインイベント
   - infrastructure: インフラ実装

2. **ビジネスロジック**
   - 各サービスのドメイン層実装
   - ヘキサゴナルアーキテクチャの適用

## マイクロサービス構成

### ビジネスサービス（6 Bounded Contexts）

1. **learning-service** - Learning Context
   - 学習セッション管理
   - 解答処理とフィードバック

2. **algorithm-service** - Learning Algorithm Context
   - SM-2 アルゴリズム実装
   - 学習効率の最適化

3. **vocabulary-service** - Vocabulary Context
   - 語彙エントリ管理
   - Wikipedia スタイルのデータモデル

4. **user-service** - User Context
   - 認証・認可
   - ユーザープロファイル管理

5. **progress-service** - Progress Context
   - イベントソーシング
   - 学習進捗の集計

6. **ai-service** - AI Integration Context
   - Gemini API 統合
   - AI 生成タスク管理

### インフラストラクチャサービス

7. **api-gateway**
   - GraphQL API エンドポイント
   - 認証統合
   - サービス間通信の調整

8. **event-processor**
   - ドメインイベント処理
   - イベントストアへの永続化
   - イベントバス（Redis Streams）との統合

9. **saga-orchestrator**
   - 分散トランザクション管理
   - 補償処理の実装
   - Saga パターンの実行

## アーキテクチャの特徴

### ヘキサゴナルアーキテクチャ

各マイクロサービスは、ヘキサゴナルアーキテクチャ（ポート＆アダプターパターン）に従って実装されています：

- **Domain 層**: ビジネスロジックの中核
- **Application 層**: ユースケースの実装
- **Ports 層**: インターフェース定義
- **Adapters 層**: 外部システムとの統合

### イベント駆動アーキテクチャ

- **イベントストア**: PostgreSQL によるイベントソーシング
- **イベントバス**: Redis Streams による非同期通信
- **CQRS**: コマンドとクエリの責務分離

### 技術スタック

- **言語**: Rust (edition 2024)
- **Web フレームワーク**: Axum
- **GraphQL**: async-graphql
- **gRPC**: Tonic
- **データベース**: PostgreSQL 18
- **キャッシュ/イベントバス**: Redis 8.2
- **認証**: Firebase Auth + Google OAuth
- **AI**: Gemini API

## ドキュメント構成

- [infrastructure.md](./infrastructure.md) - インフラストラクチャの詳細
- [development-workflow.md](./development-workflow.md) - 開発ワークフロー
- [service-structure.md](./service-structure.md) - サービス内部構造

## 次のステップ

1. 共有ライブラリの実装
   - 共通型（UserId, ItemId など）の定義
   - ドメインイベントの基本実装
   - インフラストラクチャの抽象化

2. Core Domain の実装
   - Learning Context のビジネスロジック
   - Vocabulary Context の集約実装

3. API Gateway の実装
   - GraphQL スキーマ定義
   - 各サービスとの gRPC 通信

## 参考リンク

- [DDD 設計ドキュメント](../README.md)
- [進捗サマリー](../progress-summary.md)
- [集約設計](../design/aggregate-identification.md)
