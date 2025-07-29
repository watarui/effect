# AI Integration Context - EventStorming Design Level

## 概要

AI Integration Context は、Effect プロジェクトにおける外部 AI サービスへのゲートウェイとして機能します。
OpenAI、Gemini などの AI サービスを抽象化し、Anti-Corruption Layer パターンを実装して、内部ドメインを外部 API の変更から保護します。

### 主要な責務

- **項目情報生成**: 語彙項目の意味、例文、発音などを AI で生成
- **テストカスタマイズ**: 自然言語による指示を解釈してテスト内容を調整
- **深掘りチャット**: 特定の項目について詳細な説明を提供
- **画像生成/検索**: 項目に関連するイメージ画像の取得（AI 生成または画像素材サービス）

### 設計方針

- **機能別インターフェース**: 各機能に最適なプロバイダーを選択可能
- **同期的処理**: タイムアウト付きの同期処理を基本とする（将来的に非同期化可能）
- **エラー回復**: Circuit Breaker とリトライによる安定性確保
- **Anti-Corruption Layer**: 外部 API の詳細を内部ドメインから隠蔽

## 集約の設計

### 1. AIGenerationTask（AI 生成タスク）- 集約ルート

AI による各種生成タスクを管理します。

```rust
pub struct AIGenerationTask {
    task_id: TaskId,
    task_type: TaskType,
    status: TaskStatus,

    // リクエスト情報
    requested_by: UserId,
    requested_at: DateTime<Utc>,
    request_content: RequestContent,

    // レスポンス情報
    response: Option<GenerationResponse>,
    completed_at: Option<DateTime<Utc>>,

    // エラー情報
    error: Option<TaskError>,
    retry_count: u32,
}

pub enum TaskType {
    ItemInfoGeneration { item_id: ItemId },
    TestCustomization { session_id: SessionId },
    ImageGeneration { item_id: ItemId },
}

pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

pub enum RequestContent {
    ItemInfo {
        spelling: String,
        context: Option<String>,
    },
    TestCustomization {
        instruction: String,  // "Speaking項目多めで"
        base_items: Vec<ItemId>,
    },
    ImageGeneration {
        description: String,
        style: ImageStyle,
    },
}
```

### 2. ChatSession（チャットセッション）- 集約ルート

深掘りチャット機能のセッションを管理します。

```rust
pub struct ChatSession {
    session_id: ChatSessionId,
    user_id: UserId,
    item_id: ItemId,

    // 会話履歴
    messages: Vec<ChatMessage>,

    // セッション状態
    started_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    status: SessionStatus,

    // コンテキスト
    context: ChatContext,
}

pub struct ChatMessage {
    message_id: MessageId,
    role: MessageRole,
    content: String,
    timestamp: DateTime<Utc>,
}

pub enum MessageRole {
    User,
    Assistant,
    System,
}

pub struct ChatContext {
    item_details: ItemSummary,
    user_level: Option<CefrLevel>,
    focus_areas: Vec<String>,  // ["usage", "collocations", "synonyms"]
}
```

## 機能別インターフェース

### 1. ItemInfoGenerator（項目情報生成）

```rust
pub trait ItemInfoGenerator: Send + Sync {
    /// 項目情報を生成
    async fn generate(
        &self,
        request: ItemInfoRequest,
        config: GenerationConfig,
    ) -> Result<ItemInfoResponse, AIServiceError>;

    /// プロバイダー名を取得
    fn provider_name(&self) -> &str;
}

pub struct ItemInfoRequest {
    spelling: String,
    part_of_speech: Option<String>,
    context: Option<String>,
    target_language: Language,
}

pub struct ItemInfoResponse {
    pronunciation: String,
    phonetic_respelling: String,
    definitions: Vec<Definition>,
    examples: Vec<Example>,
    synonyms: Vec<String>,
    antonyms: Vec<String>,
    collocations: Vec<String>,
    register: Option<Register>,
    cefr_level: Option<CefrLevel>,

    // メタ情報
    provider: String,
    model: String,
    generation_time_ms: u64,
}

pub struct GenerationConfig {
    timeout: Duration,
    temperature: f32,
    max_tokens: Option<u32>,
}
```

### 2. TestCustomizer（テストカスタマイズ）

```rust
pub trait TestCustomizer: Send + Sync {
    /// 自然言語の指示からテスト項目を選定
    async fn customize(
        &self,
        request: CustomizationRequest,
        config: GenerationConfig,
    ) -> Result<CustomizationResponse, AIServiceError>;
}

pub struct CustomizationRequest {
    instruction: String,  // "Speaking項目多めで、難易度は中級程度"
    available_items: Vec<ItemSummary>,
    user_context: UserContext,
    desired_count: usize,
}

pub struct CustomizationResponse {
    selected_items: Vec<SelectedItem>,
    rationale: String,  // 選定理由の説明

    // メタ情報
    provider: String,
    processing_time_ms: u64,
}

pub struct SelectedItem {
    item_id: ItemId,
    priority: f32,
    reason: String,
}
```

### 3. DeepDiveChatProvider（深掘りチャット）

```rust
pub trait DeepDiveChatProvider: Send + Sync {
    /// チャットメッセージに応答
    async fn chat(
        &self,
        session: &ChatSession,
        message: String,
        config: ChatConfig,
    ) -> Result<ChatResponse, AIServiceError>;

    /// 新しいセッションを開始
    async fn start_session(
        &self,
        context: ChatContext,
    ) -> Result<ChatSession, AIServiceError>;
}

pub struct ChatConfig {
    timeout: Duration,
    max_response_length: usize,
    system_prompt: Option<String>,
}

pub struct ChatResponse {
    content: String,
    suggested_followups: Vec<String>,

    // メタ情報
    tokens_used: TokenUsage,
    provider: String,
}
```

### 4. ImageProvider（画像プロバイダー）

```rust
pub trait ImageProvider: Send + Sync {
    /// 画像を生成または検索
    async fn get_image(
        &self,
        request: ImageRequest,
        config: ImageConfig,
    ) -> Result<ImageResponse, AIServiceError>;
}

pub struct ImageRequest {
    description: String,
    style: ImageStyle,
    item_context: ItemSummary,
}

pub enum ImageStyle {
    Realistic,
    Illustration,
    Minimalist,
    Educational,
}

pub struct ImageResponse {
    images: Vec<GeneratedImage>,
    provider: String,
}

pub struct GeneratedImage {
    url: String,
    alt_text: String,
    source: ImageSource,
    license: Option<String>,
}

pub enum ImageSource {
    AIGenerated { model: String },
    StockPhoto { service: String, photo_id: String },
    PublicDomain { attribution: String },
}
```

## プロバイダー実装

### プロバイダー選択戦略

```rust
pub struct AIServiceRouter {
    item_info_providers: Vec<Box<dyn ItemInfoGenerator>>,
    test_customizers: Vec<Box<dyn TestCustomizer>>,
    chat_providers: Vec<Box<dyn DeepDiveChatProvider>>,
    image_providers: Vec<Box<dyn ImageProvider>>,

    selection_strategy: SelectionStrategy,
}

pub enum SelectionStrategy {
    // 優先順位に基づく選択
    Priority,

    // コストベース（最も安いプロバイダー）
    CostOptimized,

    // 品質優先（最も高品質なプロバイダー）
    QualityFirst,

    // ラウンドロビン（負荷分散）
    RoundRobin,

    // 動的選択（エラー率やレスポンス時間に基づく）
    Dynamic,
}

impl AIServiceRouter {
    pub async fn select_provider<T>(&self, providers: &[Box<T>]) -> Result<&Box<T>> {
        match self.selection_strategy {
            SelectionStrategy::Priority => {
                // Circuit Breakerが開いていない最初のプロバイダー
                providers.iter()
                    .find(|p| self.is_available(p))
                    .ok_or(AIServiceError::NoAvailableProvider)
            }
            SelectionStrategy::CostOptimized => {
                // コスト計算して最安を選択
                self.select_cheapest_available(providers)
            }
            // 他の戦略...
        }
    }
}
```

### 具体的なプロバイダー実装例

```rust
// OpenAI実装
pub struct OpenAIProvider {
    client: OpenAIClient,
    api_key: String,
    rate_limiter: RateLimiter,
    circuit_breaker: CircuitBreaker,
}

impl ItemInfoGenerator for OpenAIProvider {
    async fn generate(
        &self,
        request: ItemInfoRequest,
        config: GenerationConfig,
    ) -> Result<ItemInfoResponse, AIServiceError> {
        // レート制限チェック
        self.rate_limiter.acquire().await?;

        // Circuit Breaker チェック
        self.circuit_breaker.call(async {
            // OpenAI API用のプロンプト構築
            let prompt = self.build_prompt(&request);

            // API呼び出し（タイムアウト付き）
            let response = timeout(
                config.timeout,
                self.client.completions().create(prompt)
            ).await??;

            // レスポンスを内部モデルに変換
            self.transform_response(response)
        }).await
    }
}

// Gemini実装
pub struct GeminiProvider {
    client: GeminiClient,
    api_key: String,
    rate_limiter: RateLimiter,
}

impl ItemInfoGenerator for GeminiProvider {
    // 同様の実装...
}
```

## エラーハンドリング

### エラー分類と処理

```rust
#[derive(Debug, Clone)]
pub enum AIServiceError {
    // リトライ可能なエラー
    RateLimit {
        provider: String,
        retry_after: Option<Duration>,
    },
    Timeout {
        provider: String,
        elapsed: Duration,
    },
    NetworkError {
        provider: String,
        details: String,
    },

    // リトライ不可能なエラー
    InvalidRequest {
        provider: String,
        reason: String,
    },
    InsufficientCredits {
        provider: String,
    },
    ContentFiltered {
        provider: String,
        reason: String,
    },

    // システムエラー
    CircuitBreakerOpen {
        provider: String,
        recovery_at: DateTime<Utc>,
    },
    NoAvailableProvider,
}

impl AIServiceError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AIServiceError::RateLimit { .. } |
            AIServiceError::Timeout { .. } |
            AIServiceError::NetworkError { .. }
        )
    }
}
```

### リトライ機構

```rust
pub struct RetryConfig {
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
    exponential_base: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            exponential_base: 2.0,
        }
    }
}

pub async fn with_retry<F, T>(
    operation: F,
    config: RetryConfig,
) -> Result<T, AIServiceError>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, AIServiceError>>>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) if !err.is_retryable() => return Err(err),
            Err(err) if attempt >= config.max_attempts - 1 => return Err(err),
            Err(err) => {
                attempt += 1;

                // レート制限の場合は指定された時間待つ
                if let AIServiceError::RateLimit { retry_after: Some(wait), .. } = &err {
                    tokio::time::sleep(*wait).await;
                } else {
                    tokio::time::sleep(delay).await;
                    delay = (delay.as_secs_f32() * config.exponential_base)
                        .min(config.max_delay.as_secs_f32());
                }
            }
        }
    }
}
```

### Circuit Breaker 実装

```rust
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: RwLock<Option<Instant>>,
    state: RwLock<CircuitState>,
    config: CircuitBreakerConfig,
}

pub struct CircuitBreakerConfig {
    failure_threshold: u32,      // 5回失敗で開く
    recovery_timeout: Duration,  // 1分後に半開状態へ
    success_threshold: u32,      // 3回成功で閉じる
}

#[derive(Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,     // 正常動作
    Open,       // 遮断中
    HalfOpen,   // 回復試行中
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, operation: F) -> Result<T, AIServiceError>
    where
        F: Future<Output = Result<T, AIServiceError>>,
    {
        let state = self.state.read().await;

        match *state {
            CircuitState::Open => {
                if self.should_attempt_reset().await {
                    drop(state);
                    *self.state.write().await = CircuitState::HalfOpen;
                } else {
                    return Err(AIServiceError::CircuitBreakerOpen {
                        provider: "".to_string(),
                        recovery_at: self.recovery_time().await,
                    });
                }
            }
            _ => {}
        }

        match operation.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure().await;
                Err(err)
            }
        }
    }
}
```

### レート制限

```rust
pub struct RateLimiter {
    tokens: Arc<Mutex<f64>>,
    max_tokens: f64,
    refill_rate: f64,  // トークン/秒
    last_refill: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub async fn acquire(&self) -> Result<(), AIServiceError> {
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        // トークンを補充
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        *tokens = (*tokens + elapsed * self.refill_rate).min(self.max_tokens);
        *last_refill = now;

        // トークンを消費
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            Ok(())
        } else {
            // 次のトークンが利用可能になるまでの時間を計算
            let wait_time = Duration::from_secs_f64((1.0 - *tokens) / self.refill_rate);
            Err(AIServiceError::RateLimit {
                provider: "".to_string(),
                retry_after: Some(wait_time),
            })
        }
    }
}
```

## Anti-Corruption Layer

### 外部モデルから内部モデルへの変換

```rust
// OpenAI固有のレスポンス
#[derive(Deserialize)]
struct OpenAICompletion {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

// 内部モデルへの変換
impl OpenAIProvider {
    fn transform_response(&self, response: OpenAICompletion) -> Result<ItemInfoResponse> {
        let content = response.choices
            .first()
            .ok_or(AIServiceError::InvalidResponse)?
            .message.content;

        // JSON形式の応答をパース
        let parsed: OpenAIItemInfo = serde_json::from_str(&content)?;

        // 内部モデルに変換
        Ok(ItemInfoResponse {
            pronunciation: parsed.pronunciation,
            phonetic_respelling: self.convert_phonetic(parsed.ipa),
            definitions: parsed.meanings.into_iter()
                .map(|m| self.convert_definition(m))
                .collect(),
            examples: self.extract_examples(parsed.examples),
            synonyms: parsed.synonyms.unwrap_or_default(),
            antonyms: parsed.antonyms.unwrap_or_default(),
            collocations: self.extract_collocations(parsed.usage_notes),
            register: self.map_register(parsed.formality),
            cefr_level: self.estimate_cefr_level(&parsed),

            provider: "OpenAI".to_string(),
            model: "gpt-4".to_string(),
            generation_time_ms: response.usage.total_time_ms,
        })
    }
}

// Gemini固有のレスポンス
#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    metadata: GeminiMetadata,
}

// 同様に内部モデルへ変換
impl GeminiProvider {
    fn transform_response(&self, response: GeminiResponse) -> Result<ItemInfoResponse> {
        // Gemini特有の形式から共通形式へ
        // ...
    }
}
```

## コマンドとイベント

### コマンド（青い付箋 🟦）

```rust
pub enum AIIntegrationCommand {
    // 項目情報生成
    GenerateItemInfo {
        item_id: ItemId,
        spelling: String,
        context: Option<String>,
    },

    // テストカスタマイズ
    CustomizeTest {
        session_id: SessionId,
        instruction: String,
        available_items: Vec<ItemId>,
    },

    // 深掘りチャット
    StartChatSession {
        user_id: UserId,
        item_id: ItemId,
    },

    SendChatMessage {
        session_id: ChatSessionId,
        message: String,
    },

    // 画像生成
    GenerateImage {
        item_id: ItemId,
        description: String,
        style: ImageStyle,
    },

    // タスク管理
    CancelTask {
        task_id: TaskId,
    },

    RetryTask {
        task_id: TaskId,
    },
}
```

### ドメインイベント（オレンジの付箋 🟠）

```rust
pub enum AIIntegrationEvent {
    // タスク関連
    TaskCreated {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        task_id: TaskId,
        task_type: TaskType,
    },

    TaskStarted {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        task_id: TaskId,
        provider: String,
    },

    TaskCompleted {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        task_id: TaskId,
        result: TaskResult,
        duration_ms: u64,
    },

    TaskFailed {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        task_id: TaskId,
        error: String,
        retry_count: u32,
    },

    // 項目情報生成
    ItemInfoGenerated {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        item_id: ItemId,
        info: GeneratedItemInfo,
        provider: String,
    },

    // テストカスタマイズ
    TestCustomized {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        session_id: SessionId,
        selected_items: Vec<SelectedItem>,
        rationale: String,
    },

    // チャット関連
    ChatSessionStarted {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        session_id: ChatSessionId,
        user_id: UserId,
        item_id: ItemId,
    },

    ChatMessageSent {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        session_id: ChatSessionId,
        message: ChatMessage,
    },

    ChatResponseReceived {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        session_id: ChatSessionId,
        response: ChatMessage,
        tokens_used: TokenUsage,
    },

    // 画像生成
    ImageGenerated {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        item_id: ItemId,
        images: Vec<GeneratedImage>,
        provider: String,
    },

    // エラーイベント
    ProviderUnavailable {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        provider: String,
        reason: String,
        recovery_time: Option<DateTime<Utc>>,
    },
}
```

## ビジネスポリシー（紫の付箋 🟪）

### プロバイダー選択ポリシー

```rust
// 機能別に最適なプロバイダーを選択
when GenerateItemInfoCommand {
    // 1. 利用可能なプロバイダーをフィルタ
    let available = providers.filter(|p| !circuit_breaker.is_open(p))

    // 2. コスト最適化モードの場合
    if mode == CostOptimized {
        select_cheapest(available)
    }
    // 3. 品質優先モードの場合
    else if mode == QualityFirst {
        select_highest_quality(available)
    }
    // 4. デフォルトは優先順位
    else {
        select_by_priority(available)
    }
}
```

### リトライポリシー

```rust
// エラー時のリトライ判定
when TaskFailed {
    if error.is_retryable() && retry_count < max_retries {
        schedule_retry_with_backoff()
        emit TaskRetryScheduled
    } else {
        mark_task_as_failed()
        emit TaskPermanentlyFailed
    }
}
```

### レート制限ポリシー

```rust
// API呼び出し前のレート制限チェック
when before_api_call {
    if !rate_limiter.can_proceed() {
        if queue_enabled {
            queue_for_later()
        } else {
            reject_with_rate_limit_error()
        }
    }
}
```

### コスト管理ポリシー

```rust
// 月間コスト上限チェック
when before_expensive_operation {
    if monthly_cost > budget_limit * 0.8 {
        // 80%到達で警告
        emit CostWarning

        if monthly_cost > budget_limit {
            // 上限到達で停止
            reject_with_budget_exceeded()
        }
    }
}
```

## リードモデル（緑の付箋 🟩）

### TaskStatusView（タスク状態表示）

```rust
pub struct TaskStatusView {
    task_id: TaskId,
    task_type: String,
    status: String,

    // 進捗情報
    started_at: Option<String>,
    completed_at: Option<String>,
    duration_seconds: Option<u64>,

    // エラー情報
    error_message: Option<String>,
    retry_count: u32,
    can_retry: bool,
}
```

### ProviderHealthView（プロバイダー健全性）

```rust
pub struct ProviderHealthView {
    provider_name: String,
    status: ProviderStatus,

    // パフォーマンス統計
    average_response_time_ms: f64,
    success_rate: f64,

    // エラー情報
    recent_errors: Vec<ErrorSummary>,
    circuit_breaker_state: String,

    // コスト情報
    monthly_cost: f64,
    request_count: u64,
}

pub enum ProviderStatus {
    Healthy,
    Degraded,
    Unavailable,
}
```

### UsageStatisticsView（利用統計）

```rust
pub struct UsageStatisticsView {
    period: String,  // "2024-01", "2024-01-20", etc

    // タスク別統計
    item_info_generated: u64,
    tests_customized: u64,
    chat_messages: u64,
    images_generated: u64,

    // プロバイダー別統計
    provider_usage: HashMap<String, ProviderUsage>,

    // コスト
    total_cost: f64,
    cost_by_provider: HashMap<String, f64>,
}
```

## 他コンテキストとの連携

### Vocabulary Context との連携

```rust
// Vocabulary → AI Integration
impl EventHandler for AIIntegrationContext {
    async fn handle(&self, event: VocabularyEvent) -> Result<()> {
        match event {
            VocabularyEvent::AIGenerationRequested { item_id, spelling, .. } => {
                // 項目情報生成タスクを作成
                let task = self.create_item_info_task(item_id, spelling).await?;
                self.process_task(task).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

// AI Integration → Vocabulary (コールバック)
impl AIIntegrationContext {
    async fn on_item_info_generated(&self, result: ItemInfoResponse) -> Result<()> {
        // Vocabulary Context に結果を送信
        self.vocabulary_callback.on_info_generated(
            result.item_id,
            result.into()
        ).await?;
        Ok(())
    }
}
```

### Learning Context との連携

```rust
// Learning → AI Integration
when TestCustomizationRequested {
    create_customization_task()
    process_with_ai()
    return_selected_items()
}

// 深掘りチャット要求
when DeepDiveChatRequested {
    create_or_resume_chat_session()
    provide_chat_interface()
}
```

## 実装の考慮事項

### セキュリティ

```rust
pub struct SecurityConfig {
    // APIキーの管理
    api_key_encryption: bool,
    key_rotation_interval: Duration,

    // 入力検証
    max_input_length: usize,
    content_filters: Vec<ContentFilter>,

    // 出力検証
    pii_detection: bool,
    output_sanitization: bool,
}
```

### モニタリング

```rust
pub struct AIMetrics {
    // レスポンスタイム
    response_time_histogram: Histogram,

    // 成功率
    success_counter: Counter,
    failure_counter: Counter,

    // コスト追跡
    token_usage_counter: Counter,
    cost_gauge: Gauge,

    // プロバイダー別メトリクス
    provider_metrics: HashMap<String, ProviderMetrics>,
}
```

### 非同期処理の将来拡張

```rust
// 将来的な非同期処理への拡張準備
pub enum ProcessingMode {
    // 現在の実装
    Synchronous {
        timeout: Duration,
    },

    // 将来の拡張
    Asynchronous {
        callback_url: Option<String>,
        webhook_secret: Option<String>,
    },

    // ハイブリッド
    Hybrid {
        sync_timeout: Duration,
        fallback_to_async: bool,
    },
}
```

## CQRS 適用方針

### 適用状況: ❌ 通常の DDD（CQRS なし）

AI Integration Context では、従来の DDD パターンを採用し、CQRS は適用していません。

### 理由

1. **シンプルなCRUD操作**
   - AIGenerationTask: タスクの作成、状態更新、取得
   - ChatSession: チャットメッセージの追加と履歴取得
   - 複雑な表示変換が不要

2. **内部サービスとしての性質**
   - 他のコンテキストからの要求を処理
   - AI プロバイダーへの仲介役
   - UI への直接的な表示要件が少ない

3. **リアルタイム性の要求**
   - 同期的な AI 呼び出しが中心
   - 結果整合性より強い整合性が必要
   - タスクの状態は即座に反映される必要がある

### アーキテクチャ設計

- **集約**:
  - AIGenerationTask（AI生成タスク管理）
  - ChatSession（チャットセッション管理）
- **リポジトリ**: 標準的な CRUD 操作
- **ドメインサービス**: AIServiceAdapter（プロバイダー抽象化）
- **Anti-Corruption Layer**: 外部 AI サービスとの境界

### 将来の拡張可能性

現在は CQRS 不要だが、以下の場合は検討：

- AI 利用統計の複雑な分析要件が発生
- 大量のタスク履歴の表示最適化が必要
- コスト分析などの集計処理が増加

### アーキテクチャ学習の観点

AI Integration Context を通じて以下を学習：

- Anti-Corruption Layer パターンの実装
- 外部サービスとの統合における DDD
- Circuit Breaker などの耐障害性パターン
- CQRS が不要な統合レイヤーの設計

## 更新履歴

- 2025-07-27: 初版作成（機能別インターフェース設計、同期処理実装、エラーハンドリング）
- 2025-07-28: CQRS 適用方針セクションを追加（通常の DDD パターン採用、Anti-Corruption Layer の重要性を明記）
