# ADR 002: サービス分解戦略

## ステータス

**採択** (2025-08-03)

## コンテキスト

ADR 001 で CQRS と Event Sourcing の採用を決定しました。次に、各 Bounded Context をどのようにマイクロサービスに分解するかを決定する必要があります。

考慮すべき要因：

1. **責務の分離**: Command と Query の独立性
2. **スケーラビリティ**: 読み書きの負荷特性の違い
3. **技術的制約**: 異なるストレージ要件
4. **運用複雑性**: サービス数と管理負荷のバランス
5. **学習目的**: 実践的な経験の最大化

## 決定

各 Bounded Context を以下の戦略で分解します：

### 1. Event Sourcing を採用する Context（Vocabulary, Progress）

**4サービス構成**を採用：

```
- command-service: Write Model、コマンド処理、イベント生成
- query-service: 基本的な Read Model、CRUD的なクエリ
- projection-service: イベント消費、Read Model 更新
- search-service: 特殊な検索要件（Vocabulary のみ）
```

### 2. シンプル CQRS の Context（Learning, User, Learning Algorithm, AI Integration）

**単一サービス構成**を採用：

```
- service: Command と Query を同一サービス内で分離
```

### 具体的なサービス構成

#### Vocabulary Context（4サービス）

1. **vocabulary-command-service**
   - 責務: VocabularyEntry/Item の管理、イベント発行
   - API: gRPC（Command API）
   - Storage: PostgreSQL（Event Store）

2. **vocabulary-query-service**
   - 責務: 基本的な読み取り（GetItem, GetEntry）
   - API: gRPC（Query API）
   - Storage: PostgreSQL（Read Model）

3. **vocabulary-search-service**
   - 責務: 全文検索、複雑なフィルタリング
   - API: gRPC（Search API）
   - Storage: Meilisearch

4. **vocabulary-projection-service**
   - 責務: イベント → Read Model 変換
   - Input: Pub/Sub（Event Consumer）
   - Output: PostgreSQL, Meilisearch 更新

#### Progress Context（3サービス）

1. **progress-command-service**
   - 責務: 学習記録、SM-2 計算、イベント発行
   - API: gRPC（Command API）
   - Storage: PostgreSQL（Event Store）

2. **progress-query-service**
   - 責務: 進捗表示、統計情報
   - API: gRPC（Query API）
   - Storage: PostgreSQL（Read Model）+ Redis（Cache）

3. **progress-projection-service**
   - 責務: イベント → Read Model 変換、集計
   - Input: Pub/Sub（Event Consumer）
   - Output: PostgreSQL 更新

#### その他の Context（各1サービス）

- **learning-service**: テスト生成、採点（内部で CQRS パターン）
- **user-service**: ユーザー管理（シンプル CRUD）
- **learning-algorithm-service**: SM-2 アルゴリズム計算（ステートレス処理）
- **ai-integration-service**: AI 連携（非同期処理）

### サービス間通信パターン

```
┌─────────────┐     Command      ┌──────────────┐
│   Client    │ ───────────────> │Command Service│
└─────────────┘                  └──────┬───────┘
                                        │ Event
                                        ▼
                                 ┌─────────────┐
                                 │  Event Bus  │
                                 └──────┬──────┘
                                        │
                          ┌─────────────┴──────────────┐
                          ▼                            ▼
                  ┌──────────────┐            ┌──────────────┐
                  │Projection Svc│            │Other Context │
                  └──────┬───────┘            └──────────────┘
                         │ Update
                         ▼
                  ┌──────────────┐
                  │Query Service │ <────── Query ────── Client
                  └──────────────┘
```

## 結果

### 正の結果

1. **明確な責務分離**: 各サービスが単一の責務を持つ
2. **独立したスケーリング**: 読み書きを別々にスケール可能
3. **技術の最適化**: 各サービスに最適なストレージを選択
4. **障害の分離**: 一部のサービス障害が全体に波及しない
5. **並行開発**: チームで分担して開発可能

### 負の結果

1. **サービス数の増加**: 合計約15サービス
2. **運用の複雑性**: 監視、ログ、デプロイが複雑
3. **ネットワーク遅延**: サービス間通信のオーバーヘッド
4. **開発環境**: ローカルで全サービスを起動する負荷

### 複雑性の管理

1. **Docker Compose**: 開発環境の一括管理
2. **Cloud Run**: 本番環境でのコンテナ実行
3. **Google Cloud Logging**: ログの集中管理
4. **Google Cloud Trace**: 分散トレーシング

## 代替案

### 代替案 1: Context ごとに単一サービス

各 Bounded Context を1つのサービスとして実装。

**却下理由**:

- CQRS の利点（独立スケーリング）が活かせない
- 大きなモノリシックサービスになる
- 学習機会が限定的

### 代替案 2: Command/Query の2サービス構成

Projection Service を Command または Query に統合。

**却下理由**:

- Projection のスケーラビリティが制限される
- 責務が不明確になる
- イベント処理のボトルネック

### 代替案 3: 機能ごとの細分化

さらに細かく機能ごとにサービス化（例: vocabulary-definition-service）。

**却下理由**:

- 過度な分散によるオーバーヘッド
- トランザクション境界の複雑化
- 運用負荷が現実的でない

## 移行戦略

1. **Phase 1**: Vocabulary Context の4サービス実装
2. **Phase 2**: Progress Context の3サービス実装  
3. **Phase 3**: その他の Context の実装
4. **Phase 4**: 統合層（API Gateway）の実装

## 参考資料

- [Microservices Patterns - Chris Richardson](https://microservices.io/patterns/decomposition/)
- [Domain-Driven Design and Microservices](https://www.infoq.com/articles/ddd-contextmapping/)
- [CQRS Journey - Microsoft](https://docs.microsoft.com/en-us/previous-versions/msp-n-p/jj554200(v=pandp.10))

## 更新履歴

- 2025-08-03: 初版作成
- 2025-08-03: Learning Algorithm Context 追加、技術スタック更新（Kubernetes → Cloud Run、Elasticsearch → Meilisearch）
