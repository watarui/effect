# アーキテクチャ概要

## 設計思想

effect は、ヘキサゴナルアーキテクチャをベースに、CQRS/Event Sourcing パターンを組み合わせたハイブリッドアーキテクチャを採用しています。

## アーキテクチャパターン

### ヘキサゴナルアーキテクチャ（Ports & Adapters）

```
     ┌─────────────────────────────────┐
     │      Primary Adapters           │
     │    (GraphQL, gRPC, CLI)        │
     └────────────┬────────────────────┘
                  │
    ┌─────────────▼─────────────────────┐
    │         Application Core          │
    │  ┌─────────────────────────────┐ │
    │  │      Domain Model           │ │
    │  │   (Business Logic)          │ │
    │  └─────────────────────────────┘ │
    │           Ports                   │
    └─────────────┬─────────────────────┘
                  │
     ┌────────────▼────────────────────┐
     │      Secondary Adapters         │
     │  (PostgreSQL, Pub/Sub, AI API)  │
     └─────────────────────────────────┘
```

### CQRS (Command Query Responsibility Segregation)

```
┌──────────┐     ┌─────────────┐     ┌─────────────┐
│  Client  │────▶│ API Gateway │────▶│   Command   │
└──────────┘     │  (GraphQL)  │     │   Service   │
                 └──────┬──────┘     └──────┬──────┘
                        │                    │
                        │                    ▼
                        │            ┌─────────────┐
                        │            │ Event Store │
                        │            │(PostgreSQL) │
                        │            └──────┬──────┘
                        │                    │
                        ▼                    ▼
                 ┌─────────────┐     ┌─────────────┐
                 │    Query    │◀────│ Projection  │
                 │   Service   │     │   Manager   │
                 └─────────────┘     └─────────────┘
```

## マイクロサービス構成

### 1. API Gateway

- **役割**: クライアントとの単一エンドポイント
- **技術**: GraphQL (async-graphql)
- **責務**:
  - 認証・認可
  - リクエストルーティング
  - レスポンス集約

### 2. Command Service

- **役割**: 書き込み操作の処理
- **責務**:
  - コマンドの検証
  - ビジネスロジックの実行
  - イベントの生成と保存
  - Saga の開始

### 3. Query Service

- **役割**: 読み取り操作の処理
- **責務**:
  - Read Model の管理
  - クエリの最適化
  - キャッシュ管理

### 4. Saga Executor

- **役割**: 分散トランザクションの管理
- **責務**:
  - Saga の実行
  - 補償トランザクション
  - エラーハンドリング

## 通信パターン

### 同期通信 (gRPC)

- マイクロサービス間の直接通信
- コマンド実行時の即座の応答

### 非同期通信 (Pub/Sub)

- イベントの配信
- サービス間の疎結合
- Saga のステップ実行

## データフロー

1. **コマンドフロー**:

   ```
   Client → API Gateway → Command Service → Event Store → Pub/Sub
   ```

2. **クエリフロー**:

   ```
   Client → API Gateway → Query Service → Read Model
   ```

3. **イベント処理フロー**:

   ```
   Event Store → Pub/Sub → [Query Service, Saga Executor]
   ```

## 利点

1. **関心の分離**: 読み取りと書き込みの最適化
2. **スケーラビリティ**: サービス単位での水平スケーリング
3. **イベントソーシング**: 完全な監査証跡
4. **疎結合**: サービス間の独立性
5. **テスタビリティ**: ポートとアダプターの分離
