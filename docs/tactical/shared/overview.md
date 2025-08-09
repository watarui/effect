# 共有コンポーネント概要

## 概要

このディレクトリには、Effect プロジェクトの全マイクロサービスで共有されるコンポーネントのドキュメントが含まれています。
DDD の原則に従い、ドメイン層の共有（Shared Kernel）と技術的基盤（Infrastructure、Cross-cutting Concerns）を明確に分離しています。

## アーキテクチャ概要

```
┌──────────────────────────────────────────────────────────────┐
│                     Application Services                      │
│  (Learning, Vocabulary, Learning Algorithm, AI, User)        │
│  ※ Progress は Read Model のためイベント受信専用              │
└────────────────────────┬─────────────────────────────────────┘
                         │ 依存
┌────────────────────────┴─────────────────────────────────────┐
│                      Shared Components                        │
├───────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                   Shared Kernel                         │ │
│  │  • IDs (UserId, ItemId, SessionId, etc.)              │ │
│  │  • Value Objects (CefrLevel, CourseType, etc.)        │ │
│  │  • Timestamps                                          │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Shared Infrastructure                      │ │
│  │  • Event Store (PostgreSQL)                            │ │
│  │  • Event Bus (Google Pub/Sub)                          │ │
│  │  • Repository Base                                     │ │
│  │  • Database Connection                                 │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │            Cross-cutting Concerns                       │ │
│  │  • Error Handling                                      │ │
│  │  • Security (Auth/Authz)                               │ │
│  │  • Observability (Logging, Metrics, Tracing)           │ │
│  │  • Caching                                             │ │
│  │  • Configuration                                       │ │
│  └─────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────┘
```

## ドキュメント構成

### 1. [Shared Kernel](kernel.md)

**DDD の共有カーネル** - 境界づけられたコンテキスト間で共有される、小さく安定したモデルの集合

- **識別子（IDs）**: UserId、ItemId、SessionId など
- **値オブジェクト**: CefrLevel、CourseType、LanguageCode など
- **タイムスタンプ**: 統一的な時刻管理
- **実装場所**: `shared/kernel/`

### 2. [Shared Infrastructure](infrastructure.md)

**技術的基盤** - すべてのコンテキストが利用する技術的なコンポーネント

- **Event Store**: イベントソーシングの永続化層
- **Event Bus**: 非同期メッセージング（Google Pub/Sub）
- **Repository**: CRUD 操作の基底実装
- **Database**: 接続プールとトランザクション管理
- **実装場所**: `shared/infrastructure/`

### 3. [Cross-cutting Concerns](cross-cutting-concerns.md)

**横断的関心事** - 複数のコンテキストにまたがる共通の技術的課題

- **エラー処理**: 統一的なエラーハンドリング
- **セキュリティ**: 認証・認可、データ保護
- **観測性**: ロギング、メトリクス、トレーシング
- **キャッシング**: 分散キャッシュ戦略
- **設定管理**: 環境別設定の管理
- **実装場所**: `shared/cross_cutting/`

## 設計原則

### 1. レイヤー分離

```
Domain Layer (ビジネスロジック)
    ↑
    | implements
    |
Infrastructure Layer (技術的実装)
    ↑
    | uses
    |
External Systems (データベース、メッセージング等)
```

### 2. 依存関係の方向

- ドメイン層はインフラストラクチャ層に依存しない
- インフラストラクチャ層はドメイン層のインターフェースを実装
- 依存性逆転の原則（DIP）を適用

### 3. 最小限の共有

- 本当に共有が必要なものだけを含める
- コンテキスト固有のロジックは含めない
- ビジネスルールではなく、基盤となる要素のみを共有

## 実装ディレクトリ構造

```
shared/
├── kernel/                  # Shared Kernel
│   ├── src/
│   │   ├── ids.rs          # 共通識別子
│   │   ├── value_objects.rs # 共通値オブジェクト
│   │   ├── events.rs       # イベント基底型
│   │   └── timestamp.rs    # タイムスタンプ
│   └── Cargo.toml
│
├── contexts/               # 各コンテキストの共有型
│   ├── vocabulary/
│   ├── learning/
│   ├── learning_algorithm/  # SM-2アルゴリズム専用
│   ├── progress/           # Read Model専用（集約なし）
│   ├── ai_integration/
│   └── user/
│
├── infrastructure/         # インフラストラクチャ実装
│   ├── event_store/       # Event Store
│   ├── event_bus/         # Event Bus
│   ├── repository/        # Repository 基底
│   └── database/          # DB 接続管理
│
└── cross_cutting/         # 横断的関心事
    ├── error/            # エラーハンドリング
    ├── telemetry/        # ログ・メトリクス
    ├── security/         # セキュリティ
    ├── cache/            # キャッシュ
    └── config/           # 設定管理
```

## 使用ガイドライン

### Do's ✅

1. **明確な責務分離**

   - 各コンポーネントの責務を明確に定義
   - 単一責任原則の遵守

2. **適切な抽象化**

   - トレイトによるインターフェース定義
   - 実装の詳細を隠蔽

3. **テスタビリティ**
   - モック可能な設計
   - 依存性注入の活用

### Don'ts ❌

1. **過度な共有**

   - コンテキスト固有のロジックを共有しない
   - 頻繁に変更される要素は含めない

2. **密結合**

   - 直接的な依存を避ける
   - インターフェースを介した疎結合

3. **レイヤー違反**
   - ドメイン層に技術的詳細を含めない
   - インフラ層にビジネスロジックを含めない

## 関連ドキュメント

- [Bounded Contexts](../contexts/) - 各コンテキストの詳細設計
- [Event Sourcing Guidelines](../event-sourcing-guidelines.md) - イベントソーシングの実装指針
- [Integration Patterns](../integration/patterns.md) - コンテキスト間の統合パターン

## 更新履歴

- 2025-08-06: 初版作成（ディレクトリ再編成に伴い）
