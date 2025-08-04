# vocabulary-search-service 設計書

## 概要

vocabulary-search-service は、Meilisearch を活用した高度な検索機能を提供する専門サービスです。全文検索、ファセット検索、検索候補の提供など、複雑な検索要件に特化しています。Meilisearch の高速性と使いやすさを活かし、優れた検索体験を提供します。

## 責務

1. **全文検索**
   - 単語、定義、例文の横断検索
   - 日本語・英語の混在検索
   - あいまい検索（typo 許容）

2. **高度なフィルタリング**
   - 品詞、CEFR レベル、ドメイン
   - 複数条件の組み合わせ
   - 範囲検索（作成日、更新日）

3. **検索候補とサジェスト**
   - オートコンプリート
   - "もしかして" 機能
   - 関連語の提案

4. **ファセット集計**
   - カテゴリ別の件数表示
   - 動的なフィルタオプション
   - 検索結果の統計情報

## アーキテクチャ

### レイヤー構造

```
vocabulary-search-service/
├── api/              # gRPC API 定義
├── application/      # 検索ハンドラー
├── domain/           # 検索モデル定義
├── infrastructure/   # Meilisearch 実装
└── main.rs          # エントリーポイント
```

### 詳細設計

#### API Layer

```rust
// api/grpc/vocabulary_search.proto
service VocabularySearchService {
    rpc SearchItems(SearchItemsRequest) returns (SearchItemsResponse);
    rpc GetSuggestions(GetSuggestionsRequest) returns (GetSuggestionsResponse);
    rpc SearchWithFacets(SearchWithFacetsRequest) returns (SearchWithFacetsResponse);
    rpc GetRelatedItems(GetRelatedItemsRequest) returns (GetRelatedItemsResponse);
}

message SearchItemsRequest {
    string query = 1;
    SearchFilters filters = 2;
    SearchOptions options = 3;
    Pagination pagination = 4;
}

message SearchFilters {
    repeated string part_of_speech = 1;
    repeated string cefr_levels = 2;
    repeated string domains = 3;
    DateRange created_range = 4;
    DateRange updated_range = 5;
    bool ai_generated_only = 6;
    repeated string tags = 7;
}

message SearchOptions {
    SearchMode mode = 1;
    repeated string search_fields = 2;
    float fuzziness = 3;  // 0.0-1.0
    string highlight_tag = 4;
    SortOrder sort_by = 5;
    bool include_synonyms = 6;
}

enum SearchMode {
    EXACT = 0;
    FUZZY = 1;
    WILDCARD = 2;
    PHRASE = 3;
    SEMANTIC = 4;  // 将来の拡張用
}

message SearchItemsResponse {
    repeated SearchResultItem items = 1;
    uint32 total_hits = 2;
    float max_score = 3;
    uint32 took_ms = 4;
    repeated SpellingSuggestion suggestions = 5;
}

message SearchResultItem {
    string item_id = 1;
    string spelling = 2;
    string disambiguation = 3;
    float score = 4;
    map<string, string> highlights = 5;  // field -> highlighted text
    MatchExplanation explanation = 6;
}
```

#### Domain Layer

```rust
// domain/search_models/search_document.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularySearchDocument {
    // 基本フィールド
    pub item_id: String,
    pub entry_id: String,
    pub spelling: String,
    pub disambiguation: String,
    
    // 検索対象フィールド
    pub search_text: String,  // 全フィールドを結合したテキスト
    pub definitions: Vec<String>,
    pub examples: Vec<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    
    // フィルタ用フィールド
    pub part_of_speech: String,
    pub cefr_level: Option<String>,
    pub domain: String,
    pub tags: Vec<String>,
    
    // 並べ替え用フィールド
    pub popularity_score: f32,
    pub quality_score: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // サジェスト用
    pub spelling_suggest: CompletionField,
    pub spelling_grams: String,  // n-gram for partial matching
}

// domain/search_models/facets.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacets {
    pub part_of_speech: HashMap<String, u64>,
    pub cefr_level: HashMap<String, u64>,
    pub domain: HashMap<String, u64>,
    pub tags: HashMap<String, u64>,
    pub year_created: HashMap<u32, u64>,
}

// domain/search_models/suggestion.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellingSuggestion {
    pub text: String,
    pub score: f32,
    pub frequency: u64,
}
```

#### Application Layer

```rust
// application/search_handlers/search_items_handler.rs
pub struct SearchItemsHandler {
    search_engine: Arc<dyn SearchEngine>,
    analyzer: Arc<dyn QueryAnalyzer>,
}

#[async_trait]
impl SearchHandler for SearchItemsHandler {
    type Request = SearchItemsRequest;
    type Response = SearchItemsResponse;
    
    async fn handle(&self, request: Self::Request) -> Result<Self::Response> {
        let start = Instant::now();
        
        // 1. クエリ分析
        let analyzed_query = self.analyzer.analyze(&request.query).await?;
        
        // 2. 検索クエリ構築
        let search_query = self.build_search_query(
            &analyzed_query,
            &request.filters,
            &request.options,
        )?;
        
        // 3. 検索実行
        let search_result = self.search_engine
            .search(
                "vocabulary_items",
                search_query,
                request.pagination,
            )
            .await?;
            
        // 4. スペルチェック
        let suggestions = if search_result.total_hits == 0 {
            self.get_spelling_suggestions(&request.query).await?
        } else {
            vec![]
        };
        
        // 5. レスポンス構築
        let response = SearchItemsResponse {
            items: self.map_to_result_items(search_result.hits),
            total_hits: search_result.total_hits,
            max_score: search_result.max_score,
            took_ms: start.elapsed().as_millis() as u32,
            suggestions,
        };
        
        Ok(response)
    }
}

impl SearchItemsHandler {
    fn build_search_query(
        &self,
        analyzed: &AnalyzedQuery,
        filters: &SearchFilters,
        options: &SearchOptions,
    ) -> Result<MeilisearchQuery> {
        // Meilisearch クエリ構築
        let query_string = match options.mode {
            SearchMode::Exact => format!("\"{}\"", analyzed.normalized_query),
            SearchMode::Fuzzy => analyzed.normalized_query.clone(),
            SearchMode::Phrase => format!("\"{}\"", analyzed.normalized_query),
            SearchMode::Wildcard => format!("{}*", analyzed.normalized_query),
            SearchMode::Semantic => analyzed.normalized_query.clone(), // 将来の拡張用
        };
        
        // フィルタ構築
        let mut filter_conditions = Vec::new();
        
        if !filters.part_of_speech.is_empty() {
            let pos_filter = filters.part_of_speech.iter()
                .map(|pos| format!("part_of_speech = {}", pos))
                .collect::<Vec<_>>()
                .join(" OR ");
            filter_conditions.push(format!("({})", pos_filter));
        }
        
        if !filters.cefr_levels.is_empty() {
            let level_filter = filters.cefr_levels.iter()
                .map(|level| format!("cefr_level = {}", level))
                .collect::<Vec<_>>()
                .join(" OR ");
            filter_conditions.push(format!("({})", level_filter));
        }
        
        if filters.ai_generated_only {
            filter_conditions.push("is_ai_generated = true".to_string());
        }
        
        let filter = if filter_conditions.is_empty() {
            None
        } else {
            Some(filter_conditions.join(" AND "))
        };
        
        Ok(MeilisearchQuery {
            query_string,
            filter,
            highlight: self.build_highlight_config(options),
            sort: self.build_sort_config(options),
        })
    }
}

// application/search_handlers/facet_search_handler.rs
pub struct FacetSearchHandler {
    search_engine: Arc<dyn SearchEngine>,
}

impl FacetSearchHandler {
    async fn handle(&self, request: SearchWithFacetsRequest) -> Result<SearchWithFacetsResponse> {
        // Meilisearch のファセット設定
        let facets = vec![
            "part_of_speech",
            "cefr_level",
            "domain",
            "tags",
        ];
            
        let index = self.search_engine.get_index("vocabulary_items");
        let mut search_query = index.search();
        
        search_query
            .with_query(&request.query)
            .with_facets(&facets)
            .with_attributes_to_retrieve(&["item_id", "spelling", "disambiguation"])
            .with_offset(request.pagination.offset)
            .with_limit(request.pagination.limit);
            
        let result = search_query.execute::<VocabularySearchDocument>().await?;
        
        let facet_distribution = result.facets.unwrap_or_default();
        
        Ok(SearchWithFacetsResponse {
            items: result.hits,
            total_hits: result.total_hits,
            facets,
        })
    }
}
```

#### Infrastructure Layer

```rust
// infrastructure/search/meilisearch_engine.rs
pub struct MeilisearchEngine {
    client: meilisearch_sdk::Client,
    index_name: String,
}

#[async_trait]
impl SearchEngine for MeilisearchEngine {
    async fn search(
        &self,
        index: &str,
        query: MeilisearchQuery,
        pagination: Pagination,
    ) -> Result<SearchResult> {
        let index = self.client.index(index);
        
        let mut search_query = index.search();
        search_query
            .with_query(&query.query_string)
            .with_offset(pagination.offset)
            .with_limit(pagination.limit)
            .with_attributes_to_retrieve(&["item_id", "spelling", "disambiguation", "_rankingScore"])
            .with_show_ranking_score(true);
            
        // フィルタ適用
        if let Some(filter) = &query.filter {
            search_query.with_filter(filter);
        }
        
        // ソート適用
        if let Some(sort) = &query.sort {
            search_query.with_sort(sort);
        }
        
        // ハイライト設定
        if let Some(highlight) = &query.highlight {
            search_query.with_attributes_to_highlight(highlight);
        }
        
        let response = search_query.execute::<VocabularySearchDocument>().await?;
        
        Ok(self.convert_meilisearch_response(response))
    }
    
    async fn index_document(
        &self,
        index: &str,
        doc_id: &str,
        document: VocabularySearchDocument,
    ) -> Result<()> {
        // n-gram フィールドの生成
        let mut doc = document;
        doc.spelling_grams = self.generate_ngrams(&doc.spelling, 2, 4);
        doc.search_text = self.build_search_text(&doc);
        
        let index = self.client.index(index);
        
        // Meilisearch ドキュメントに変換
        let meilisearch_doc = json!({
            "id": doc_id,
            "item_id": doc.item_id,
            "entry_id": doc.entry_id,
            "spelling": doc.spelling,
            "disambiguation": doc.disambiguation,
            "search_text": doc.search_text,
            "definitions": doc.definitions,
            "examples": doc.examples,
            "synonyms": doc.synonyms,
            "antonyms": doc.antonyms,
            "part_of_speech": doc.part_of_speech,
            "cefr_level": doc.cefr_level,
            "domain": doc.domain,
            "tags": doc.tags,
            "popularity_score": doc.popularity_score,
            "quality_score": doc.quality_score,
            "created_at": doc.created_at.timestamp(),
            "updated_at": doc.updated_at.timestamp(),
        });
        
        index.add_documents(&[meilisearch_doc], Some("id")).await?;
        
        Ok(())
    }
}

impl MeilisearchEngine {
    fn build_search_text(&self, doc: &VocabularySearchDocument) -> String {
        let mut parts = vec![
            doc.spelling.clone(),
            doc.disambiguation.clone(),
        ];
        
        parts.extend(doc.definitions.clone());
        parts.extend(doc.examples.clone());
        parts.extend(doc.synonyms.clone());
        
        parts.join(" ")
    }
    
    fn generate_ngrams(&self, text: &str, min: usize, max: usize) -> String {
        let mut ngrams = HashSet::new();
        let chars: Vec<char> = text.chars().collect();
        
        for n in min..=max {
            for i in 0..chars.len().saturating_sub(n - 1) {
                let ngram: String = chars[i..i + n].iter().collect();
                ngrams.insert(ngram);
            }
        }
        
        ngrams.into_iter().collect::<Vec<_>>().join(" ")
    }
}

// infrastructure/search/query_analyzer.rs
pub struct QueryAnalyzer {
    tokenizer: Arc<dyn Tokenizer>,
    synonym_dict: Arc<SynonymDictionary>,
}

#[async_trait]
impl QueryAnalyzerTrait for QueryAnalyzer {
    async fn analyze(&self, query: &str) -> Result<AnalyzedQuery> {
        // 1. トークン化
        let tokens = self.tokenizer.tokenize(query)?;
        
        // 2. 正規化
        let normalized = tokens.iter()
            .map(|t| t.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");
            
        // 3. 同義語展開
        let synonyms = self.expand_synonyms(&tokens).await?;
        
        // 4. 言語検出
        let language = self.detect_language(&tokens);
        
        Ok(AnalyzedQuery {
            original_query: query.to_string(),
            normalized_query: normalized,
            tokens,
            synonyms,
            language,
        })
    }
}
```

## Meilisearch インデックス設計

### インデックス設定

```rust
// infrastructure/search/index_config.rs
pub async fn configure_meilisearch_index(client: &Client) -> Result<()> {
    let index = client.index("vocabulary_items");
    
    // 検索可能な属性を設定
    index.set_searchable_attributes(&[
        "spelling",
        "disambiguation",
        "definitions",
        "examples",
        "synonyms",
        "search_text",
    ]).await?;
    
    // フィルタ可能な属性を設定
    index.set_filterable_attributes(&[
        "part_of_speech",
        "cefr_level",
        "domain",
        "tags",
        "created_at",
        "updated_at",
        "is_ai_generated",
    ]).await?;
    
    // ソート可能な属性を設定
    index.set_sortable_attributes(&[
        "popularity_score",
        "quality_score",
        "created_at",
        "updated_at",
    ]).await?;
    
    // ランキングルールを設定
    index.set_ranking_rules(&[
        "words",
        "typo",
        "proximity",
        "attribute",
        "sort",
        "exactness",
        "popularity_score:desc",
        "quality_score:desc",
    ]).await?;
    
    // 同義語辞書を設定
    let synonyms = HashMap::from([
        ("learn", vec!["study", "acquire", "master"]),
        ("big", vec!["large", "huge", "enormous"]),
        ("small", vec!["little", "tiny", "petite"]),
    ]);
    index.set_synonyms(&synonyms).await?;
    
    // Typo tolerance を設定
    index.set_typo_tolerance(TypoToleranceSettings {
        enabled: true,
        min_word_size_for_typos: MinWordSizeForTypos {
            one_typo: 5,
            two_typos: 9,
        },
        disable_on_words: vec![],
        disable_on_attributes: vec![],
    }).await?;
    
    Ok(())
}
```

### インデックス最適化

```rust
// Meilisearch の最適化設定
pub async fn optimize_meilisearch_settings(client: &Client) -> Result<()> {
    let index = client.index("vocabulary_items");
    
    // パフォーマンス設定
    index.update_settings(Settings {
        pagination: Some(PaginationSettings {
            max_total_hits: 10000,
        }),
        faceting: Some(FacetingSettings {
            max_values_per_facet: 100,
        }),
        search_cutoff_ms: Some(150), // 150ms でタイムアウト
        ..Default::default()
    }).await?;
    
    Ok(())
}

// インデックスの再構築（必要時のみ）
pub async fn rebuild_index(&self) -> Result<()> {
    let index = self.client.index(&self.index_name);
    
    // スワップインデックスを作成
    let swap_index_name = format!("{}_swap", self.index_name);
    let swap_index = self.client.create_index(&swap_index_name, Some("item_id")).await?;
    
    // 設定をコピー
    let settings = index.get_settings().await?;
    swap_index.set_settings(&settings).await?;
    
    // データを再インデックス
    // ... (バッチ処理)
    
    // インデックスをスワップ
    self.client.swap_indexes(vec![
        IndexSwap {
            indexes: (self.index_name.clone(), swap_index_name),
        }
    ]).await?;
    
    Ok(())
}
```

## パフォーマンスチューニング

### 検索最適化

1. **クエリキャッシュ**

   ```rust
   // 頻繁な検索パターンをキャッシュ
   pub struct CachedSearchEngine {
       engine: Arc<MeilisearchEngine>,
       cache: Arc<Cache<String, SearchResult>>,
   }
   ```

2. **検索プリセット**

   ```rust
   pub struct SearchPresets {
       pub quick_search: SearchConfig {
           attributes: vec!["spelling", "disambiguation"],
           limit: 10,
           show_ranking_score: false,
       },
       pub full_search: SearchConfig {
           attributes: vec!["spelling", "definitions", "examples", "synonyms"],
           limit: 50,
           show_ranking_score: true,
       },
       pub autocomplete: SearchConfig {
           attributes: vec!["spelling"],
           limit: 5,
           highlight_pre_tag: "<b>",
           highlight_post_tag: "</b>",
       },
   }
   ```

3. **Meilisearch クラスタ設定**
   - マルチインスタンス構成
   - ロードバランサー経由でアクセス
   - 自動フェイルオーバー

### 応答時間目標

| 操作 | 目標時間 (p95) | 最大許容時間 |
|-----|--------------|-----------|
| 単純検索 | 50ms | 200ms |
| ファセット検索 | 100ms | 500ms |
| サジェスト | 20ms | 50ms |
| 関連項目検索 | 80ms | 300ms |

## 設定とデプロイメント

### 環境変数

```yaml
MEILISEARCH_URL: http://meilisearch:7700
MEILISEARCH_API_KEY: ${MEILISEARCH_MASTER_KEY}
SERVICE_PORT: 50053
INDEX_NAME: vocabulary_items
# Meilisearch 設定
MAX_PAYLOAD_SIZE: 100MB
DUMP_DIR: /var/meilisearch/dumps
```

### Cloud Run 設定

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: vocabulary-search-service
spec:
  template:
    metadata:
      annotations:
        run.googleapis.com/vpc-connector: projects/PROJECT/locations/REGION/connectors/CONNECTOR
        autoscaling.knative.dev/minScale: "1"
        autoscaling.knative.dev/maxScale: "100"
    spec:
      serviceAccountName: vocabulary-service
      containers:
      - image: gcr.io/effect-project/vocabulary-search-service:latest
        ports:
        - containerPort: 50053
        env:
        - name: MEILISEARCH_URL
          valueFrom:
            secretKeyRef:
              name: vocabulary-secrets
              key: meilisearch-url
        - name: MEILISEARCH_API_KEY
          valueFrom:
            secretKeyRef:
              name: vocabulary-secrets
              key: meilisearch-api-key
        resources:
          limits:
            memory: "1Gi"
            cpu: "2000m"
```

## 監視とアラート

### メトリクス

```rust
lazy_static! {
    static ref SEARCH_LATENCY: HistogramVec = register_histogram_vec!(
        "vocabulary_search_latency_seconds",
        "Search query latency",
        &["query_type", "result_count"]
    ).unwrap();
    
    static ref SEARCH_HITS: Counter = register_counter!(
        "vocabulary_search_hits_total",
        "Total search queries with results"
    ).unwrap();
    
    static ref SEARCH_MISSES: Counter = register_counter!(
        "vocabulary_search_misses_total",
        "Total search queries with no results"
    ).unwrap();
}
```

### Meilisearch 監視

```yaml
# Prometheus による監視
- job_name: 'meilisearch'
  static_configs:
  - targets: ['meilisearch:7700']
  metrics_path: '/metrics'
  
# ヘルスチェック
health_check:
  endpoint: '/health'
  interval: 10s
  timeout: 5s
```

## 更新履歴

- 2025-08-03: 初版作成
