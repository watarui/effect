# Vocabulary Context - アーキテクチャ

## 概要

Vocabulary Context は CQRS（Command Query Responsibility Segregation）+ Event Sourcing パターンを採用し、4 つのマイクロサービスに分解されています。これにより、書き込みと読み取りの責務を分離し、それぞれ最適化できます。

## マイクロサービス構成

### 1. vocabulary_command_service

**責務**: コマンド処理とドメインロジックの実行

- 語彙項目の作成・更新・削除コマンドを処理
- ビジネスルールの検証（重複チェック、ステータス遷移など）
- ドメインイベントの生成
- Event Store への永続化

**主要コンポーネント**:

- CommandHandler: コマンドの受付と処理
- VocabularyRepository: Event Store への保存
- DomainService: ビジネスロジック

### 2. vocabulary_query_service

**責務**: 高速な読み取り処理

- Read Model からの語彙情報取得
- キャッシュ層（Redis）による応答速度の最適化
- GraphQL への応答

**主要コンポーネント**:

- QueryHandler: クエリの処理
- ReadRepository: Read Model からの取得
- CacheManager: Redis キャッシュ管理

### 3. vocabulary_projection_service

**責務**: Event Store から Read Model への投影

- ドメインイベントの購読
- Read Model（非正規化ビュー）の更新
- 投影状態の管理

**主要コンポーネント**:

- EventHandler: イベントの処理
- ProjectionManager: 投影の管理
- ReadModelRepository: Read Model の更新

### 4. vocabulary_search_service

**責務**: シンプルな検索機能の提供

- Meilisearch SDK の標準機能を活用
- 基本的な全文検索（typo許容、部分一致）
- シンプルな自動補完
- 最小限の設定で実用的な検索を実現

**主要コンポーネント**:

- SearchHandler: 検索リクエストの処理
- MeilisearchClient: SDK を使った検索実行
- IndexManager: インデックスの同期管理

## データフロー

### 書き込みフロー（Command）

```
1. Client → API Gateway → vocabulary_command_service
2. CommandHandler がコマンドを検証
3. ドメインモデルがビジネスルールを適用
4. イベントを生成して Event Store に保存
5. Event Bus にイベントを発行
```

### 投影フロー（Projection）

```
1. vocabulary_projection_service が Event Bus からイベントを受信
2. イベントに基づいて Read Model を更新
3. 非正規化されたビューを構築（JSONB 形式）
4. vocabulary_search_service の検索インデックスも更新
```

### 読み取りフロー（Query）

```
1. Client → API Gateway → vocabulary_query_service
2. Redis キャッシュをチェック
3. キャッシュミスの場合、Read Model から取得
4. 非正規化されたデータを返却（JOIN 不要）
```

## データモデルの対比

### Write Model（Event Store）

```sql
-- イベントの履歴を保存
events (
  event_id UUID,
  aggregate_id UUID,
  event_type VARCHAR,    -- "VocabularyItemCreated" など
  event_data JSONB,      -- イベントの詳細
  event_version INTEGER
)
```

### Read Model（投影）

```sql
-- 現在の状態を非正規化して保存
vocabulary_items_view (
  item_id UUID,
  spelling VARCHAR,
  definitions JSONB,     -- すべての定義と例文を含む
  synonyms JSONB,
  antonyms JSONB,
  -- パフォーマンスのため一部フィールドは別カラム
  status VARCHAR,
  cefr_level VARCHAR
)
```

## 主要な設計判断

### 1. イベントソーシングの採用理由

- **完全な監査証跡**: すべての変更履歴が残る
- **時系列での状態再現**: 任意の時点の状態を再構築可能
- **並行編集の解決**: イベントレベルでの競合解決

### 2. CQRS の採用理由

- **パフォーマンス最適化**: 読み取りは非正規化、書き込みは正規化
- **スケーラビリティ**: Read/Write を独立してスケール可能
- **複雑性の分離**: コマンドとクエリのロジックを分離

### 3. 非正規化の範囲

**JSONB として格納**:

- definitions（定義、例文、ドメインを含む）
- synonyms（類義語）
- antonyms（対義語）
- collocations（コロケーション）

**別カラムとして格納**:

- status（フィルタリング用）
- cefr_level（フィルタリング用）
- spelling（検索用）

### 4. 最終的一貫性

- Write → Read の反映には若干の遅延がある（通常数百ミリ秒）
- ユーザー体験を損なわない範囲での非同期処理
- 重要な操作後は明示的な同期オプション

### 5. Read Model バージョニング

Event Sourcing と API バージョニングの統合により、柔軟な Read Model 管理が可能：

**バージョン別 Read Model**:

- `/api/v1/` → `vocabulary_items_v1` テーブル
- `/api/v2/` → `vocabulary_items_v2` テーブル（新しいスキーマ）

**利点**:

- 同一のイベントストリームから複数のビューを生成
- クライアントの段階的移行が可能
- 破壊的変更なしに新機能を追加

**実装例**:

```
Event Store (不変)
    ↓
Projection Service
    ├→ Read Model V1 (既存クライアント用)
    └→ Read Model V2 (新機能を含む)
```

**移行戦略**:

1. 新バージョンの Read Model を並行稼働
2. クライアントを段階的に移行
3. 旧バージョンの利用が0になったら削除

## 運用上の考慮事項

### モニタリング

- イベント処理の遅延監視
- 投影の同期状態チェック
- キャッシュヒット率の監視

### エラーハンドリング

- イベント処理の失敗時はリトライ
- Dead Letter Queue での失敗イベント管理
- 投影の再構築機能

### スケーリング

- Command Service: 垂直スケーリング（整合性重視）
- Query Service: 水平スケーリング（読み取り負荷分散）
- Projection Service: パーティショニングによる並列処理
