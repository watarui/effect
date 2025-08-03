# vocabulary-query-service 設計書

## 概要

vocabulary-query-service は、Vocabulary Context の Read Model に対する基本的な読み取り操作を提供します。
項目の詳細取得、エントリー一覧、シンプルなフィルタリングなど、CRUD 的な読み取り要求を高速に処理します。
複雑な検索は vocabulary-search-service に委譲します。

## 責務

1. **基本的な読み取り操作**

   - 項目詳細の取得
   - エントリー詳細の取得
   - 項目一覧の取得（ページネーション付き）

2. **シンプルなフィルタリング**

   - 品詞によるフィルタ
   - 難易度によるフィルタ
   - カテゴリによるフィルタ

3. **関連データの取得**

   - エントリーに紐づく全項目
   - 項目の例文一覧
   - 関連語の取得

4. **キャッシュ管理**
   - 頻繁にアクセスされるデータのキャッシュ
   - キャッシュの一貫性保証

## アーキテクチャ

### レイヤー構造

```
vocabulary-query-service/
├── api/              # gRPC API 定義
├── application/      # クエリハンドラー
├── domain/           # 読み取りモデル
├── infrastructure/   # データアクセス、キャッシュ
└── main.rs          # エントリーポイント
```

### 詳細設計

#### API Layer

```rust
// api/grpc/vocabulary_query.proto
service VocabularyQueryService {
    // 単一取得
    rpc GetEntry(GetEntryQuery) returns (EntryResponse);
    rpc GetItem(GetItemQuery) returns (ItemResponse);
    rpc GetItemsByEntry(GetItemsByEntryQuery) returns (ItemsResponse);

    // 一覧取得
    rpc ListEntries(ListEntriesQuery) returns (EntriesResponse);
    rpc ListItems(ListItemsQuery) returns (ItemsResponse);
    rpc ListRecentItems(ListRecentItemsQuery) returns (ItemsResponse);

    // 関連データ取得
    rpc GetExamples(GetExamplesQuery) returns (ExamplesResponse);
    rpc GetRelatedWords(GetRelatedWordsQuery) returns (RelatedWordsResponse);
    rpc GetItemHistory(GetItemHistoryQuery) returns (ItemHistoryResponse);

    // 統計情報
    rpc GetVocabularyStats(GetVocabularyStatsQuery) returns (VocabularyStatsResponse);
    rpc GetUserContributions(GetUserContributionsQuery) returns (UserContributionsResponse);
}

message GetEntryQuery {
    string entry_id = 1;
    bool include_items = 2;
}

message EntryResponse {
    Entry entry = 1;
    repeated ItemSummary items = 2;
}

message Entry {
    string entry_id = 1;
    string word = 2;
    string reading = 3;
    PartOfSpeech part_of_speech = 4;
    uint32 item_count = 5;
    string created_by = 6;
    string created_at = 7;
    string updated_at = 8;
    int64 version = 9;
}

message GetItemQuery {
    string item_id = 1;
    bool include_examples = 2;
    bool include_related = 3;
}

message ItemResponse {
    Item item = 1;
    repeated Example examples = 2;
    repeated RelatedWord related_words = 3;
}

message Item {
    string item_id = 1;
    string entry_id = 2;
    string word = 3;
    string definition = 4;
    Difficulty difficulty = 5;
    repeated string categories = 6;
    repeated string tags = 7;
    float quality_score = 8;
    uint32 example_count = 9;
    string created_by = 10;
    string created_at = 11;
    string updated_at = 12;
    int64 version = 13;
}

message ListItemsQuery {
    ListFilter filter = 1;
    Pagination pagination = 2;
    SortOption sort = 3;
}

message ListFilter {
    repeated PartOfSpeech part_of_speech = 1;
    repeated Difficulty difficulty = 2;
    repeated string categories = 3;
    repeated string tags = 4;
    DateRange created_range = 5;
    DateRange updated_range = 6;
}

message Pagination {
    uint32 offset = 1;
    uint32 limit = 2;
}

message SortOption {
    SortField field = 1;
    SortDirection direction = 2;
}

enum SortField {
    CREATED_AT = 0;
    UPDATED_AT = 1;
    WORD = 2;
    DIFFICULTY = 3;
    QUALITY_SCORE = 4;
}

enum SortDirection {
    ASC = 0;
    DESC = 1;
}
```

#### Domain Layer

```rust
// domain/read_models/entry_view.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryView {
    pub entry_id: EntryId,
    pub word: String,
    pub reading: Option<String>,
    pub part_of_speech: PartOfSpeech,
    pub item_count: u32,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: Version,

    // 集計情報
    pub total_examples: u32,
    pub difficulty_distribution: HashMap<Difficulty, u32>,
    pub category_distribution: HashMap<String, u32>,
}

impl EntryView {
    pub fn from_projection(projection: EntryProjection) -> Self {
        Self {
            entry_id: projection.entry_id,
            word: projection.word,
            reading: projection.reading,
            part_of_speech: projection.part_of_speech,
            item_count: projection.items.len() as u32,
            created_by: projection.created_by,
            created_at: projection.created_at,
            updated_at: projection.updated_at,
            version: projection.version,
            total_examples: 0, // 別途集計
            difficulty_distribution: HashMap::new(),
            category_distribution: HashMap::new(),
        }
    }
}

// domain/read_models/item_view.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemView {
    pub item_id: ItemId,
    pub entry_id: EntryId,
    pub word: String,
    pub definition: String,
    pub difficulty: Difficulty,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub quality_score: f32,
    pub example_count: u32,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: Version,

    // 追加の表示情報
    pub contributor_name: String,
    pub is_ai_generated: bool,
    pub review_status: ReviewStatus,
}

// domain/read_models/example_view.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleView {
    pub example_id: ExampleId,
    pub item_id: ItemId,
    pub sentence: String,
    pub translation: String,
    pub source: String,
    pub tags: Vec<String>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,

    // 使用統計
    pub usage_count: u32,
    pub helpfulness_score: f32,
}

// domain/read_models/vocabulary_stats.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyStats {
    pub total_entries: u64,
    pub total_items: u64,
    pub total_examples: u64,
    pub total_contributors: u64,
    pub last_updated: DateTime<Utc>,

    pub growth_stats: GrowthStats,
    pub quality_stats: QualityStats,
    pub contribution_stats: ContributionStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthStats {
    pub daily_new_items: Vec<(NaiveDate, u32)>,
    pub monthly_growth_rate: f32,
    pub trending_categories: Vec<(String, u32)>,
}
```

#### Application Layer

```rust
// application/query_handlers/get_item_handler.rs
pub struct GetItemHandler {
    item_repository: Arc<dyn ItemReadRepository>,
    example_repository: Arc<dyn ExampleReadRepository>,
    cache_service: Arc<dyn CacheService>,
}

impl GetItemHandler {
    const CACHE_KEY_PREFIX: &'static str = "item:detail:";
    const CACHE_TTL: Duration = Duration::from_secs(300); // 5分

    pub async fn handle(
        &self,
        query: GetItemQuery,
    ) -> Result<ItemResponse, QueryError> {
        // 1. キャッシュチェック
        let cache_key = format!("{}{}", Self::CACHE_KEY_PREFIX, query.item_id);

        if let Some(cached) = self.cache_service
            .get::<ItemView>(&cache_key)
            .await?
        {
            return self.build_response(cached, &query).await;
        }

        // 2. データベースから取得
        let item = self.item_repository
            .find_by_id(&query.item_id)
            .await?
            .ok_or(QueryError::NotFound)?;

        // 3. キャッシュに保存
        self.cache_service
            .set(&cache_key, &item, Self::CACHE_TTL)
            .await?;

        // 4. レスポンス構築
        self.build_response(item, &query).await
    }

    async fn build_response(
        &self,
        item: ItemView,
        query: &GetItemQuery,
    ) -> Result<ItemResponse, QueryError> {
        let mut response = ItemResponse {
            item: item.into(),
            examples: vec![],
            related_words: vec![],
        };

        // 例文の取得
        if query.include_examples {
            let examples = self.example_repository
                .find_by_item_id(&query.item_id)
                .await?;
            response.examples = examples.into_iter()
                .map(Into::into)
                .collect();
        }

        // 関連語の取得
        if query.include_related {
            // 関連語の取得ロジック
        }

        Ok(response)
    }
}

// application/query_handlers/list_items_handler.rs
pub struct ListItemsHandler {
    item_repository: Arc<dyn ItemReadRepository>,
    cache_service: Arc<dyn CacheService>,
}

impl ListItemsHandler {
    pub async fn handle(
        &self,
        query: ListItemsQuery,
    ) -> Result<ItemsResponse, QueryError> {
        // 1. フィルタをキャッシュキーに変換
        let cache_key = self.build_cache_key(&query);

        // 2. キャッシュチェック（リスト系は短めのTTL）
        if let Some(cached) = self.cache_service
            .get::<Vec<ItemView>>(&cache_key)
            .await?
        {
            return Ok(self.build_items_response(cached));
        }

        // 3. リポジトリから取得
        let filter = self.build_repository_filter(&query.filter);
        let sort = self.build_repository_sort(&query.sort);

        let (items, total) = self.item_repository
            .find_with_filter(filter, sort, query.pagination)
            .await?;

        // 4. キャッシュに保存（1分）
        self.cache_service
            .set(&cache_key, &items, Duration::from_secs(60))
            .await?;

        // 5. レスポンス構築
        Ok(ItemsResponse {
            items: items.into_iter().map(Into::into).collect(),
            total_count: total,
            has_more: total > (query.pagination.offset + query.pagination.limit) as u64,
        })
    }

    fn build_repository_filter(&self, filter: &ListFilter) -> RepositoryFilter {
        let mut repo_filter = RepositoryFilter::new();

        if !filter.part_of_speech.is_empty() {
            repo_filter.add_in("part_of_speech", filter.part_of_speech.clone());
        }

        if !filter.difficulty.is_empty() {
            repo_filter.add_in("difficulty", filter.difficulty.clone());
        }

        if !filter.categories.is_empty() {
            repo_filter.add_array_contains("categories", filter.categories.clone());
        }

        if let Some(created_range) = &filter.created_range {
            repo_filter.add_range("created_at", created_range.start, created_range.end);
        }

        repo_filter
    }
}

// application/query_handlers/get_vocabulary_stats_handler.rs
pub struct GetVocabularyStatsHandler {
    stats_repository: Arc<dyn StatsReadRepository>,
    cache_service: Arc<dyn CacheService>,
}

impl GetVocabularyStatsHandler {
    const CACHE_KEY: &'static str = "vocabulary:stats:global";
    const CACHE_TTL: Duration = Duration::from_secs(3600); // 1時間

    pub async fn handle(
        &self,
        _query: GetVocabularyStatsQuery,
    ) -> Result<VocabularyStatsResponse, QueryError> {
        // 統計情報は更新頻度が低いので長めにキャッシュ
        if let Some(cached) = self.cache_service
            .get::<VocabularyStats>(Self::CACHE_KEY)
            .await?
        {
            return Ok(VocabularyStatsResponse { stats: cached });
        }

        // 各種統計を並列で取得
        let (total_entries, total_items, total_examples, total_contributors) = tokio::join!(
            self.stats_repository.count_entries(),
            self.stats_repository.count_items(),
            self.stats_repository.count_examples(),
            self.stats_repository.count_contributors(),
        );

        let growth_stats = self.stats_repository
            .get_growth_stats(30)
            .await?;

        let stats = VocabularyStats {
            total_entries: total_entries?,
            total_items: total_items?,
            total_examples: total_examples?,
            total_contributors: total_contributors?,
            last_updated: Utc::now(),
            growth_stats,
            quality_stats: self.stats_repository.get_quality_stats().await?,
            contribution_stats: self.stats_repository.get_contribution_stats().await?,
        };

        self.cache_service
            .set(Self::CACHE_KEY, &stats, Self::CACHE_TTL)
            .await?;

        Ok(VocabularyStatsResponse { stats })
    }
}
```

#### Infrastructure Layer

```rust
// infrastructure/repositories/postgres_item_read_repository.rs
pub struct PostgresItemReadRepository {
    pool: PgPool,
}

#[async_trait]
impl ItemReadRepository for PostgresItemReadRepository {
    async fn find_by_id(&self, item_id: &str) -> Result<Option<ItemView>, RepositoryError> {
        let row = sqlx::query_as!(
            ItemRow,
            r#"
            SELECT
                i.item_id,
                i.entry_id,
                e.word,
                i.definition,
                i.difficulty as "difficulty: _",
                i.categories,
                i.tags,
                i.quality_score,
                i.example_count,
                i.created_by,
                u.username as contributor_name,
                i.created_at,
                i.updated_at,
                i.version,
                i.is_ai_generated,
                i.review_status as "review_status: _"
            FROM vocabulary_items i
            JOIN vocabulary_entries e ON i.entry_id = e.entry_id
            JOIN users u ON i.created_by = u.user_id
            WHERE i.item_id = $1
            "#,
            item_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(ItemView::from))
    }

    async fn find_with_filter(
        &self,
        filter: RepositoryFilter,
        sort: RepositorySort,
        pagination: Pagination,
    ) -> Result<(Vec<ItemView>, u64), RepositoryError> {
        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT
                i.*,
                e.word,
                u.username as contributor_name
            FROM vocabulary_items i
            JOIN vocabulary_entries e ON i.entry_id = e.entry_id
            JOIN users u ON i.created_by = u.user_id
            WHERE 1=1
            "#
        );

        // フィルタ条件の追加
        for (field, condition) in filter.conditions {
            match condition {
                FilterCondition::In(values) => {
                    query_builder.push(format!(" AND {} = ANY(${})", field, query_builder.argnum()));
                    query_builder.push_bind(values);
                }
                FilterCondition::ArrayContains(values) => {
                    query_builder.push(format!(" AND {} && ${}::text[]", field, query_builder.argnum()));
                    query_builder.push_bind(values);
                }
                FilterCondition::Range(start, end) => {
                    query_builder.push(format!(" AND {} BETWEEN ${} AND ${}",
                        field, query_builder.argnum(), query_builder.argnum() + 1));
                    query_builder.push_bind(start);
                    query_builder.push_bind(end);
                }
            }
        }

        // ソート条件の追加
        query_builder.push(format!(" ORDER BY {} {}", sort.field, sort.direction));

        // ページネーション
        query_builder.push(format!(" LIMIT ${} OFFSET ${}",
            query_builder.argnum(), query_builder.argnum() + 1));
        query_builder.push_bind(pagination.limit as i64);
        query_builder.push_bind(pagination.offset as i64);

        let query = query_builder.build_query_as::<ItemRow>();
        let items = query.fetch_all(&self.pool).await?;

        // 総件数の取得
        let count_query = format!(
            "SELECT COUNT(*) FROM vocabulary_items i WHERE 1=1 {}",
            self.build_filter_clause(&filter)
        );
        let total: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&self.pool)
            .await?;

        Ok((
            items.into_iter().map(ItemView::from).collect(),
            total.0 as u64,
        ))
    }
}

// infrastructure/cache/redis_cache_service.rs
pub struct RedisCacheService {
    client: redis::Client,
    serializer: Arc<dyn Serializer>,
}

#[async_trait]
impl CacheService for RedisCacheService {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        let mut conn = self.client.get_async_connection().await?;

        let data: Option<Vec<u8>> = conn.get(key).await?;

        match data {
            Some(bytes) => {
                let value = self.serializer.deserialize(&bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;

        let bytes = self.serializer.serialize(value)?;

        conn.set_ex(key, bytes, ttl.as_secs() as usize).await?;

        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        conn.del(key).await?;
        Ok(())
    }

    async fn invalidate_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;

        let keys: Vec<String> = conn.keys(pattern).await?;

        if !keys.is_empty() {
            conn.del(keys).await?;
        }

        Ok(())
    }
}
```

## データベース設計

### Read Model テーブル

```sql
-- エントリービュー
CREATE TABLE vocabulary_entries (
    entry_id UUID PRIMARY KEY,
    word VARCHAR(255) NOT NULL,
    reading VARCHAR(255),
    part_of_speech VARCHAR(50) NOT NULL,
    item_count INTEGER NOT NULL DEFAULT 0,
    created_by VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    version BIGINT NOT NULL,

    -- インデックス
    INDEX idx_word (word),
    INDEX idx_part_of_speech (part_of_speech),
    INDEX idx_created_at (created_at DESC)
);

-- 項目ビュー
CREATE TABLE vocabulary_items (
    item_id UUID PRIMARY KEY,
    entry_id UUID NOT NULL REFERENCES vocabulary_entries(entry_id),
    definition TEXT NOT NULL,
    difficulty VARCHAR(20) NOT NULL,
    categories TEXT[] NOT NULL DEFAULT '{}',
    tags TEXT[] NOT NULL DEFAULT '{}',
    quality_score REAL NOT NULL DEFAULT 0.0,
    example_count INTEGER NOT NULL DEFAULT 0,
    created_by VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    version BIGINT NOT NULL,
    is_ai_generated BOOLEAN NOT NULL DEFAULT FALSE,
    review_status VARCHAR(20) NOT NULL DEFAULT 'pending',

    -- インデックス
    INDEX idx_entry_id (entry_id),
    INDEX idx_difficulty (difficulty),
    INDEX idx_categories (categories) USING GIN,
    INDEX idx_tags (tags) USING GIN,
    INDEX idx_created_at (created_at DESC),
    INDEX idx_quality_score (quality_score DESC)
);

-- 例文ビュー
CREATE TABLE vocabulary_examples (
    example_id UUID PRIMARY KEY,
    item_id UUID NOT NULL REFERENCES vocabulary_items(item_id),
    sentence TEXT NOT NULL,
    translation TEXT NOT NULL,
    source VARCHAR(255),
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_by VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    helpfulness_score REAL NOT NULL DEFAULT 0.0,

    -- インデックス
    INDEX idx_item_id (item_id),
    INDEX idx_created_at (created_at DESC)
);

-- 統計ビュー（マテリアライズドビュー）
CREATE MATERIALIZED VIEW vocabulary_stats_view AS
SELECT
    COUNT(DISTINCT e.entry_id) as total_entries,
    COUNT(DISTINCT i.item_id) as total_items,
    COUNT(DISTINCT ex.example_id) as total_examples,
    COUNT(DISTINCT i.created_by) as total_contributors,
    NOW() as last_updated
FROM vocabulary_entries e
LEFT JOIN vocabulary_items i ON e.entry_id = i.entry_id
LEFT JOIN vocabulary_examples ex ON i.item_id = ex.item_id;

-- 定期的なリフレッシュ
CREATE INDEX idx_stats_view ON vocabulary_stats_view(last_updated);
```

## パフォーマンス最適化

### キャッシュ戦略

```yaml
cache_policies:
  item_detail:
    ttl: 300 # 5分
    invalidation:
      - on_event: ItemUpdated
      - on_event: ItemDeleted

  entry_detail:
    ttl: 300 # 5分
    invalidation:
      - on_event: EntryUpdated
      - on_event: ItemAddedToEntry

  list_queries:
    ttl: 60 # 1分
    invalidation:
      - on_event: ItemCreated
      - on_event: ItemUpdated

  statistics:
    ttl: 3600 # 1時間
    invalidation:
      - scheduled: "0 * * * *" # 毎時0分
```

### クエリ最適化

1. **インデックス活用**

   - 複合インデックスの適切な設計
   - 部分インデックスによるメモリ効率化

2. **バッチ取得**

   - N+1 問題の回避
   - IN 句を使った効率的な取得

3. **プロジェクション最適化**
   - 必要なカラムのみ取得
   - JOIN の最小化

## 設定とデプロイメント

### 環境変数

```yaml
# データベース
DATABASE_URL: postgres://user:pass@postgres:5432/vocabulary_read
DATABASE_MAX_CONNECTIONS: 25
DATABASE_MIN_CONNECTIONS: 5

# Redis
REDIS_URL: redis://redis:6379
REDIS_MAX_CONNECTIONS: 10

# サービス設定
SERVICE_PORT: 50052
GRPC_MAX_MESSAGE_SIZE: 4194304 # 4MB

# 監視
METRICS_PORT: 9090
LOG_LEVEL: info
```

### Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY proto ./proto
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/vocabulary-query-service /usr/local/bin/
EXPOSE 50052 9090
CMD ["vocabulary-query-service"]
```

### Cloud Run デプロイメント

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: vocabulary-query-service
spec:
  template:
    metadata:
      annotations:
        run.googleapis.com/cloudsql-instances: project:region:instance
        run.googleapis.com/vpc-connector: projects/PROJECT/locations/REGION/connectors/CONNECTOR
    spec:
      serviceAccountName: vocabulary-service
      containers:
        - image: gcr.io/effect-project/vocabulary-query-service:latest
          ports:
            - containerPort: 50052
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: vocabulary-secrets
                  key: read-database-url
            - name: REDIS_URL
              valueFrom:
                secretKeyRef:
                  name: vocabulary-secrets
                  key: redis-url
          resources:
            limits:
              memory: "1Gi"
              cpu: "2000m"
```

## 監視とメトリクス

### 主要メトリクス

```rust
lazy_static! {
    static ref QUERY_DURATION: HistogramVec = register_histogram_vec!(
        "vocabulary_query_duration_seconds",
        "Query execution time",
        &["query_type", "status"]
    ).unwrap();

    static ref CACHE_HIT_RATE: GaugeVec = register_gauge_vec!(
        "vocabulary_cache_hit_rate",
        "Cache hit rate by query type",
        &["query_type"]
    ).unwrap();

    static ref QUERY_COUNT: IntCounterVec = register_int_counter_vec!(
        "vocabulary_query_total",
        "Total number of queries",
        &["query_type", "status"]
    ).unwrap();
}
```

### アラート設定

```yaml
alerts:
  - name: HighQueryLatency
    condition: vocabulary_query_duration_seconds_p95 > 0.1
    severity: warning

  - name: LowCacheHitRate
    condition: vocabulary_cache_hit_rate < 0.8
    severity: warning

  - name: HighErrorRate
    condition: rate(vocabulary_query_total{status="error"}[5m]) > 0.05
    severity: critical
```

## 更新履歴

- 2025-08-03: 初版作成
