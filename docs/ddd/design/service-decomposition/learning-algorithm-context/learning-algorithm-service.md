# learning-algorithm-service 設計書

## 概要

learning-algorithm-service は、SM-2 をはじめとする各種学習アルゴリズムの計算を提供するステートレスなマイクロサービスです。
Progress Context から独立させることで、アルゴリズムの改善や新規アルゴリズムの追加を容易にし、A/B テストによる最適化を可能にします。

## 責務

1. **アルゴリズム計算**

   - SM-2 アルゴリズムの実行
   - 代替アルゴリズムの提供
   - パラメータカスタマイズ

2. **最適化サービス**

   - ユーザー別パラメータ最適化
   - 学習効率の分析
   - アルゴリズム推奨

3. **実験管理**

   - A/B テストの実行
   - メトリクス収集
   - 結果分析

4. **予測サービス**
   - 学習曲線予測
   - 負荷予測
   - 成功率推定

## アーキテクチャ

### レイヤー構造

```
learning-algorithm-service/
├── api/              # gRPC API 定義
├── application/      # サービスハンドラー
├── domain/           # アルゴリズム実装
├── infrastructure/   # 外部連携、メトリクス
└── main.rs          # エントリーポイント
```

### 詳細設計

#### API Layer

```rust
// api/grpc/learning_algorithm.proto
service LearningAlgorithmService {
    // 基本計算
    rpc CalculateSM2(SM2CalculationRequest) returns (SM2CalculationResponse);
    rpc Calculate(AlgorithmCalculationRequest) returns (AlgorithmCalculationResponse);

    // 最適化
    rpc OptimizeParameters(ParameterOptimizationRequest) returns (ParameterOptimizationResponse);
    rpc RecommendAlgorithm(AlgorithmRecommendationRequest) returns (AlgorithmRecommendationResponse);

    // 予測
    rpc PredictLearningCurve(LearningCurvePredictionRequest) returns (LearningCurvePredictionResponse);
    rpc EstimateWorkload(WorkloadEstimationRequest) returns (WorkloadEstimationResponse);

    // 分析
    rpc AnalyzePerformance(PerformanceAnalysisRequest) returns (PerformanceAnalysisResponse);
    rpc CompareAlgorithms(AlgorithmComparisonRequest) returns (AlgorithmComparisonResponse);

    // 実験
    rpc GetExperimentAssignment(ExperimentAssignmentRequest) returns (ExperimentAssignmentResponse);
    rpc RecordExperimentMetric(ExperimentMetricRequest) returns (ExperimentMetricResponse);
}

// 基本メッセージ
message SM2CalculationRequest {
    RecallQuality quality = 1;
    uint32 current_repetition = 2;
    float current_interval = 3;
    float current_easiness_factor = 4;
    optional SM2Parameters custom_parameters = 5;
    optional string user_id = 6;  // パラメータ最適化用
}

message SM2Parameters {
    repeated float initial_intervals = 1;  // [1.0, 6.0]
    float easy_bonus = 2;                  // 1.3
    float min_easiness_factor = 3;         // 1.3
    float max_easiness_factor = 4;         // 4.0
    EFModifier ef_modifier = 5;
}

message EFModifier {
    float quality_5_bonus = 1;    // 0.1
    float quality_4_bonus = 2;    // 0.05
    float quality_3_penalty = 3;  // 0.0
    float quality_2_penalty = 4;  // 0.08
    float quality_1_penalty = 5;  // 0.14
    float quality_0_penalty = 6;  // 0.8
}

message SM2CalculationResponse {
    float next_interval_days = 1;
    float easiness_factor = 2;
    uint32 repetition_number = 3;
    string next_review_date = 4;
    float retention_probability = 5;
    ConfidenceInterval confidence = 6;
    DebugInfo debug_info = 7;
}

// 汎用アルゴリズム
message AlgorithmCalculationRequest {
    string algorithm_name = 1;
    map<string, google.protobuf.Value> inputs = 2;
    map<string, google.protobuf.Value> parameters = 3;
    optional string user_id = 4;
}

// 最適化
message ParameterOptimizationRequest {
    string user_id = 1;
    string algorithm_name = 2;
    repeated LearningHistory history = 3;
    OptimizationObjectives objectives = 4;
}

message OptimizationObjectives {
    float maximize_retention_weight = 1;     // 0.0-1.0
    float minimize_workload_weight = 2;      // 0.0-1.0
    float target_success_rate = 3;           // 0.0-1.0
    float target_daily_workload_minutes = 4;
}

message ParameterOptimizationResponse {
    string algorithm_name = 1;
    map<string, google.protobuf.Value> optimized_parameters = 2;
    float expected_improvement_percentage = 3;
    float confidence_score = 4;
    OptimizationMetrics before_metrics = 5;
    OptimizationMetrics after_metrics = 6;
}

// 予測
message LearningCurvePredictionRequest {
    string user_id = 1;
    LearningState current_state = 2;
    string algorithm_name = 3;
    uint32 days_ahead = 4;
    optional SimulationConfig config = 5;
}

message LearningCurvePredictionResponse {
    repeated DailyPrediction predictions = 1;
    ConfidenceInterval confidence_interval = 2;
    repeated LearningMilestone expected_milestones = 3;
    PredictionSummary summary = 4;
}

message DailyPrediction {
    uint32 day = 1;
    uint32 expected_reviews = 2;
    uint32 expected_new_items = 3;
    float retention_rate = 4;
    uint32 workload_minutes = 5;
    float difficulty_distribution = 6;
}
```

#### Domain Layer

```rust
// domain/algorithms/mod.rs
pub trait LearningAlgorithm: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn calculate(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, AlgorithmError>;

    fn default_parameters(&self) -> Parameters;

    fn validate_parameters(&self, params: &Parameters) -> Result<(), ValidationError>;

    fn estimate_retention(
        &self,
        interval_days: f32,
        easiness_factor: f32,
    ) -> f32;
}

// domain/algorithms/sm2.rs
pub struct SM2Algorithm {
    config: SM2Config,
    retention_model: RetentionModel,
}

impl SM2Algorithm {
    pub fn new(config: SM2Config) -> Self {
        Self {
            config,
            retention_model: RetentionModel::default(),
        }
    }

    fn calculate_internal(&self, input: &SM2Input) -> SM2Output {
        let quality = input.quality;

        // 1. Easiness Factor の更新
        let new_ef = self.update_easiness_factor(
            input.current_ef,
            quality,
        );

        // 2. 間隔と繰り返し回数の計算
        let (interval, repetition) = self.calculate_interval_and_repetition(
            quality,
            input.current_repetition,
            input.current_interval,
            new_ef,
        );

        // 3. 次回復習日の計算
        let next_review = Utc::now() + Duration::days(interval.round() as i64);

        // 4. 記憶保持確率の推定
        let retention = self.retention_model.estimate(
            interval,
            new_ef,
            repetition,
        );

        // 5. 信頼区間の計算
        let confidence = self.calculate_confidence_interval(
            retention,
            input.history_size,
        );

        SM2Output {
            interval_days: interval,
            easiness_factor: new_ef,
            repetition_number: repetition,
            next_review_date: next_review,
            retention_probability: retention,
            confidence_interval: confidence,
        }
    }

    fn update_easiness_factor(&self, current_ef: f32, quality: RecallQuality) -> f32 {
        use RecallQuality::*;

        let modifier = &self.config.ef_modifier;
        let delta = match quality {
            Perfect => modifier.quality_5_bonus,
            CorrectEasy => modifier.quality_4_bonus,
            CorrectDifficult => -modifier.quality_3_penalty,
            IncorrectEasy => -modifier.quality_2_penalty,
            IncorrectDifficult => -modifier.quality_1_penalty,
            Blackout => -modifier.quality_0_penalty,
        };

        (current_ef + delta).clamp(
            self.config.min_easiness_factor,
            self.config.max_easiness_factor,
        )
    }

    fn calculate_interval_and_repetition(
        &self,
        quality: RecallQuality,
        current_repetition: u32,
        current_interval: f32,
        easiness_factor: f32,
    ) -> (f32, u32) {
        use RecallQuality::*;

        match quality {
            Blackout | IncorrectDifficult | IncorrectEasy => {
                // 失敗: リセット
                (self.config.initial_intervals[0], 0)
            }
            _ => {
                // 成功: 次の間隔を計算
                let (base_interval, new_repetition) = match current_repetition {
                    0 => (self.config.initial_intervals[0], 1),
                    1 => (self.config.initial_intervals.get(1).copied().unwrap_or(6.0), 2),
                    _ => (current_interval * easiness_factor, current_repetition + 1),
                };

                // Perfect ボーナス
                let final_interval = if quality == Perfect {
                    base_interval * self.config.easy_bonus
                } else {
                    base_interval
                };

                (final_interval, new_repetition)
            }
        }
    }
}

impl LearningAlgorithm for SM2Algorithm {
    fn name(&self) -> &str {
        "SM-2"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn calculate(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, AlgorithmError> {
        let sm2_input = SM2Input::try_from(input)?;
        let output = self.calculate_internal(&sm2_input);
        Ok(output.into())
    }

    fn default_parameters(&self) -> Parameters {
        Parameters::from(self.config.clone())
    }

    fn validate_parameters(&self, params: &Parameters) -> Result<(), ValidationError> {
        // パラメータ検証ロジック
        Ok(())
    }

    fn estimate_retention(&self, interval_days: f32, easiness_factor: f32) -> f32 {
        self.retention_model.estimate(interval_days, easiness_factor, 0)
    }
}

// domain/algorithms/retention_model.rs
pub struct RetentionModel {
    forgetting_curve: ForgettingCurve,
}

impl RetentionModel {
    pub fn estimate(
        &self,
        interval_days: f32,
        easiness_factor: f32,
        repetition_number: u32,
    ) -> f32 {
        // Ebbinghaus の忘却曲線をベースに、EF で調整
        let base_retention = self.forgetting_curve.retention_at(interval_days);

        // EF による調整（高いEF = より良い記憶）
        let ef_adjustment = (easiness_factor - 1.3) / (4.0 - 1.3) * 0.2 + 0.9;

        // 繰り返し回数による調整
        let repetition_bonus = (repetition_number as f32 * 0.05).min(0.2);

        (base_retention * ef_adjustment + repetition_bonus).min(0.99)
    }
}

// domain/optimization/parameter_optimizer.rs
pub struct ParameterOptimizer {
    optimization_engine: OptimizationEngine,
    constraint_validator: ConstraintValidator,
}

impl ParameterOptimizer {
    pub async fn optimize(
        &self,
        user_id: &UserId,
        algorithm: &dyn LearningAlgorithm,
        history: &[LearningHistory],
        objectives: &OptimizationObjectives,
    ) -> Result<OptimizedParameters, OptimizationError> {
        // 1. 現在のパフォーマンスを分析
        let current_metrics = self.analyze_current_performance(history);

        // 2. 最適化の制約を設定
        let constraints = Constraints {
            min_daily_workload: 10,
            max_daily_workload: 60,
            min_success_rate: 0.7,
            max_success_rate: 0.95,
        };

        // 3. パラメータ空間の探索
        let search_space = self.define_search_space(algorithm);

        // 4. 最適化の実行（ベイズ最適化）
        let optimal_params = self.optimization_engine.optimize(
            search_space,
            objectives,
            constraints,
            |params| self.evaluate_parameters(params, history, algorithm),
        )?;

        // 5. 結果の検証
        let expected_metrics = self.simulate_with_parameters(
            &optimal_params,
            history,
            algorithm,
        );

        Ok(OptimizedParameters {
            algorithm_name: algorithm.name().to_string(),
            parameters: optimal_params,
            current_metrics,
            expected_metrics,
            improvement: self.calculate_improvement(&current_metrics, &expected_metrics),
            confidence: self.calculate_confidence(history.len()),
        })
    }

    fn evaluate_parameters(
        &self,
        params: &Parameters,
        history: &[LearningHistory],
        algorithm: &dyn LearningAlgorithm,
    ) -> f64 {
        // パラメータを使ってhistoryを再シミュレーション
        let simulator = LearningSimulator::new(algorithm.clone(), params.clone());
        let simulated_results = simulator.simulate_history(history);

        // 目的関数の計算
        let retention_score = simulated_results.average_retention;
        let workload_score = 1.0 - (simulated_results.average_workload / 60.0).min(1.0);

        retention_score * 0.7 + workload_score * 0.3
    }
}

// domain/prediction/learning_curve_predictor.rs
pub struct LearningCurvePredictor {
    monte_carlo_simulator: MonteCarloSimulator,
    statistical_model: StatisticalModel,
}

impl LearningCurvePredictor {
    pub fn predict(
        &self,
        current_state: &LearningState,
        algorithm: &dyn LearningAlgorithm,
        config: &PredictionConfig,
    ) -> LearningCurvePrediction {
        let mut predictions = Vec::new();
        let mut confidence_bounds = Vec::new();

        // モンテカルロシミュレーション
        let simulations = self.monte_carlo_simulator.run(
            current_state,
            algorithm,
            config.days_ahead,
            config.num_simulations,
        );

        // 日ごとの予測を集計
        for day in 1..=config.days_ahead {
            let day_results = simulations.get_day_results(day);

            let prediction = DailyPrediction {
                day,
                expected_reviews: day_results.mean_reviews(),
                expected_new_items: day_results.mean_new_items(),
                retention_rate: day_results.mean_retention(),
                workload_minutes: day_results.mean_workload(),
                difficulty_distribution: day_results.difficulty_histogram(),
            };

            predictions.push(prediction);

            // 信頼区間の計算
            confidence_bounds.push(ConfidenceBound {
                lower: day_results.percentile(0.05),
                upper: day_results.percentile(0.95),
            });
        }

        // マイルストーンの予測
        let milestones = self.predict_milestones(&simulations, current_state);

        LearningCurvePrediction {
            predictions,
            confidence_interval: ConfidenceInterval { bounds: confidence_bounds },
            expected_milestones: milestones,
            summary: self.create_summary(&predictions),
        }
    }
}

// domain/experiments/ab_test_manager.rs
pub struct ABTestManager {
    experiments: Arc<RwLock<HashMap<ExperimentId, Experiment>>>,
    assignment_service: AssignmentService,
    metrics_collector: MetricsCollector,
}

impl ABTestManager {
    pub async fn get_assignment(
        &self,
        user_id: &UserId,
        context: &ExperimentContext,
    ) -> AlgorithmAssignment {
        let experiments = self.experiments.read().await;

        // アクティブな実験を探す
        for (exp_id, experiment) in experiments.iter() {
            if !experiment.is_active() {
                continue;
            }

            // ユーザーが実験対象かチェック
            if !experiment.matches_criteria(user_id, context) {
                continue;
            }

            // 既存の割り当てを確認
            if let Some(assignment) = self.assignment_service
                .get_existing_assignment(user_id, exp_id)
                .await
            {
                return assignment;
            }

            // 新規割り当て
            let variant = experiment.assign_variant(user_id);
            let assignment = AlgorithmAssignment {
                algorithm: variant.algorithm.clone(),
                parameters: variant.parameters.clone(),
                experiment_id: Some(exp_id.clone()),
                variant_id: Some(variant.id.clone()),
            };

            self.assignment_service
                .save_assignment(user_id, exp_id, &assignment)
                .await;

            return assignment;
        }

        // デフォルトアルゴリズム
        AlgorithmAssignment::default()
    }

    pub async fn record_metric(
        &self,
        metric: ExperimentMetric,
    ) -> Result<(), MetricsError> {
        // メトリクスの記録
        self.metrics_collector.record(metric).await?;

        // 実験の統計的有意性をチェック
        if let Some(experiment_id) = &metric.experiment_id {
            self.check_experiment_completion(experiment_id).await?;
        }

        Ok(())
    }

    async fn check_experiment_completion(
        &self,
        experiment_id: &ExperimentId,
    ) -> Result<(), MetricsError> {
        let metrics = self.metrics_collector
            .get_experiment_metrics(experiment_id)
            .await?;

        // 統計的有意性の計算
        let significance = calculate_statistical_significance(&metrics);

        if significance.is_conclusive() {
            let mut experiments = self.experiments.write().await;
            if let Some(experiment) = experiments.get_mut(experiment_id) {
                experiment.complete(significance);
            }
        }

        Ok(())
    }
}
```

#### Application Layer

```rust
// application/handlers/calculate_sm2_handler.rs
pub struct CalculateSM2Handler {
    algorithm_registry: Arc<AlgorithmRegistry>,
    parameter_cache: Arc<ParameterCache>,
    metrics: Arc<Metrics>,
}

impl CalculateSM2Handler {
    pub async fn handle(
        &self,
        request: SM2CalculationRequest,
    ) -> Result<SM2CalculationResponse, ServiceError> {
        let timer = self.metrics.calculation_time.start_timer();

        // 1. パラメータの取得（カスタムまたはユーザー最適化済み）
        let parameters = if let Some(custom) = request.custom_parameters {
            SM2Parameters::from(custom)
        } else if let Some(user_id) = &request.user_id {
            self.parameter_cache
                .get_user_parameters(user_id, "SM-2")
                .await?
                .unwrap_or_default()
        } else {
            SM2Parameters::default()
        };

        // 2. アルゴリズムの取得
        let algorithm = self.algorithm_registry
            .get("SM-2")
            .ok_or(ServiceError::AlgorithmNotFound)?;

        // 3. 入力の準備
        let input = AlgorithmInput::from_sm2(
            request.quality,
            request.current_repetition,
            request.current_interval,
            request.current_easiness_factor,
            parameters,
        );

        // 4. 計算の実行
        let output = algorithm.calculate(input)?;

        // 5. レスポンスの構築
        let response = SM2CalculationResponse {
            next_interval_days: output.get_f32("interval")?,
            easiness_factor: output.get_f32("easiness_factor")?,
            repetition_number: output.get_u32("repetition")?,
            next_review_date: output.get_string("next_review_date")?,
            retention_probability: output.get_f32("retention")?,
            confidence: output.get_confidence_interval()?,
            debug_info: if request.include_debug {
                Some(self.create_debug_info(&input, &output))
            } else {
                None
            },
        };

        timer.observe_duration();
        self.metrics.calculation_count.with_label_values(&["SM-2", "success"]).inc();

        Ok(response)
    }
}

// application/handlers/optimize_parameters_handler.rs
pub struct OptimizeParametersHandler {
    optimizer: Arc<ParameterOptimizer>,
    algorithm_registry: Arc<AlgorithmRegistry>,
    cache: Arc<ParameterCache>,
}

impl OptimizeParametersHandler {
    pub async fn handle(
        &self,
        request: ParameterOptimizationRequest,
    ) -> Result<ParameterOptimizationResponse, ServiceError> {
        // 1. アルゴリズムの取得
        let algorithm = self.algorithm_registry
            .get(&request.algorithm_name)
            .ok_or(ServiceError::AlgorithmNotFound)?;

        // 2. 学習履歴の変換
        let history: Vec<LearningHistory> = request.history
            .into_iter()
            .map(LearningHistory::from)
            .collect();

        // 3. 最適化の実行
        let optimized = self.optimizer
            .optimize(
                &UserId::from(request.user_id.clone()),
                algorithm,
                &history,
                &request.objectives.into(),
            )
            .await?;

        // 4. 結果をキャッシュ
        self.cache
            .set_user_parameters(
                &request.user_id,
                &request.algorithm_name,
                &optimized.parameters,
                Duration::from_secs(86400), // 24時間
            )
            .await?;

        // 5. レスポンスの構築
        Ok(ParameterOptimizationResponse {
            algorithm_name: optimized.algorithm_name,
            optimized_parameters: optimized.parameters.to_map(),
            expected_improvement_percentage: optimized.improvement.percentage,
            confidence_score: optimized.confidence,
            before_metrics: optimized.current_metrics.into(),
            after_metrics: optimized.expected_metrics.into(),
        })
    }
}

// application/handlers/predict_learning_curve_handler.rs
pub struct PredictLearningCurveHandler {
    predictor: Arc<LearningCurvePredictor>,
    algorithm_registry: Arc<AlgorithmRegistry>,
}

impl PredictLearningCurveHandler {
    pub async fn handle(
        &self,
        request: LearningCurvePredictionRequest,
    ) -> Result<LearningCurvePredictionResponse, ServiceError> {
        // 1. アルゴリズムの取得
        let algorithm = self.algorithm_registry
            .get(&request.algorithm_name)
            .ok_or(ServiceError::AlgorithmNotFound)?;

        // 2. 現在の状態を準備
        let current_state = LearningState::from(request.current_state);

        // 3. 予測設定
        let config = request.config
            .map(PredictionConfig::from)
            .unwrap_or_default();

        // 4. 予測の実行
        let prediction = self.predictor.predict(
            &current_state,
            algorithm,
            &config,
        );

        // 5. レスポンスの構築
        Ok(LearningCurvePredictionResponse {
            predictions: prediction.predictions.into_iter()
                .map(Into::into)
                .collect(),
            confidence_interval: prediction.confidence_interval.into(),
            expected_milestones: prediction.expected_milestones.into_iter()
                .map(Into::into)
                .collect(),
            summary: prediction.summary.into(),
        })
    }
}
```

#### Infrastructure Layer

```rust
// infrastructure/cache/parameter_cache.rs
pub struct RedisParameterCache {
    client: redis::Client,
    serializer: Arc<dyn Serializer>,
}

#[async_trait]
impl ParameterCache for RedisParameterCache {
    async fn get_user_parameters(
        &self,
        user_id: &str,
        algorithm: &str,
    ) -> Result<Option<Parameters>, CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("params:{}:{}", user_id, algorithm);

        let data: Option<Vec<u8>> = conn.get(&key).await?;

        match data {
            Some(bytes) => {
                let params = self.serializer.deserialize(&bytes)?;
                Ok(Some(params))
            }
            None => Ok(None),
        }
    }

    async fn set_user_parameters(
        &self,
        user_id: &str,
        algorithm: &str,
        parameters: &Parameters,
        ttl: Duration,
    ) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("params:{}:{}", user_id, algorithm);

        let bytes = self.serializer.serialize(parameters)?;

        conn.set_ex(&key, bytes, ttl.as_secs() as usize).await?;

        Ok(())
    }
}

// infrastructure/metrics/prometheus_metrics.rs
pub struct PrometheusMetrics {
    pub calculation_time: HistogramVec,
    pub calculation_count: CounterVec,
    pub optimization_time: Histogram,
    pub cache_hit_rate: GaugeVec,
    pub active_experiments: Gauge,
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        Self {
            calculation_time: register_histogram_vec!(
                "learning_algorithm_calculation_duration_seconds",
                "Time taken to calculate algorithm",
                &["algorithm", "version"]
            ).unwrap(),

            calculation_count: register_counter_vec!(
                "learning_algorithm_calculations_total",
                "Total number of calculations",
                &["algorithm", "status"]
            ).unwrap(),

            optimization_time: register_histogram!(
                "learning_algorithm_optimization_duration_seconds",
                "Time taken to optimize parameters"
            ).unwrap(),

            cache_hit_rate: register_gauge_vec!(
                "learning_algorithm_cache_hit_rate",
                "Cache hit rate for parameters",
                &["cache_type"]
            ).unwrap(),

            active_experiments: register_gauge!(
                "learning_algorithm_active_experiments",
                "Number of active A/B test experiments"
            ).unwrap(),
        }
    }
}
```

## 設定とデプロイメント

### 環境変数

```yaml
# サービス設定
SERVICE_PORT: 50070
GRPC_MAX_MESSAGE_SIZE: 4194304 # 4MB

# アルゴリズム設定
ALGORITHMS_CONFIG_PATH: /config/algorithms.yaml
ENABLE_EXPERIMENTAL_ALGORITHMS: false

# キャッシュ設定
REDIS_URL: redis://redis:6379
PARAMETER_CACHE_TTL_SECONDS: 86400 # 24時間
CACHE_POOL_SIZE: 10

# 実験設定
EXPERIMENTS_CONFIG_PATH: /config/experiments.yaml
EXPERIMENT_ASSIGNMENT_SALT: "effect-learning-2025"

# 監視
METRICS_PORT: 9090
LOG_LEVEL: info
TRACE_ENDPOINT: https://cloudtrace.googleapis.com
```

### アルゴリズム設定 (algorithms.yaml)

```yaml
algorithms:
  - name: SM-2
    version: "1.0.0"
    enabled: true
    parameters:
      initial_intervals: [1.0, 6.0]
      easy_bonus: 1.3
      min_easiness_factor: 1.3
      max_easiness_factor: 4.0
      ef_modifier:
        quality_5_bonus: 0.1
        quality_4_bonus: 0.05
        quality_3_penalty: 0.0
        quality_2_penalty: 0.08
        quality_1_penalty: 0.14
        quality_0_penalty: 0.8

  - name: SM-2-modified
    version: "1.1.0"
    enabled: true
    parameters:
      initial_intervals: [1.0, 4.0, 10.0]
      easy_bonus: 1.5
      min_easiness_factor: 1.3
      max_easiness_factor: 3.5

  - name: SM-15
    version: "0.9.0"
    enabled: false # 実験的
    parameters:
      initial_stability: 4.0
      stability_increase_factor: 1.8
      difficulty_weight: 0.2

  - name: Leitner
    version: "1.0.0"
    enabled: true
    parameters:
      box_intervals: [1, 3, 7, 14, 30, 60]
      failure_penalty_boxes: 2
```

### 実験設定 (experiments.yaml)

```yaml
experiments:
  - id: "ef-range-optimization-2025-08"
    name: "Easiness Factor 範囲最適化"
    status: active
    start_date: "2025-08-01"
    end_date: "2025-08-31"

    targeting:
      user_percentage: 10
      criteria:
        min_items_learned: 50
        active_days: 7

    variants:
      - id: control
        name: "Control"
        allocation: 50
        algorithm: SM-2
        parameters: default

      - id: narrow-ef
        name: "Narrow EF Range"
        allocation: 25
        algorithm: SM-2
        parameters:
          min_easiness_factor: 1.5
          max_easiness_factor: 3.0

      - id: wide-ef
        name: "Wide EF Range"
        allocation: 25
        algorithm: SM-2
        parameters:
          min_easiness_factor: 1.1
          max_easiness_factor: 5.0

    metrics:
      - retention_rate
      - workload_minutes
      - user_satisfaction

    success_criteria:
      min_sample_size: 1000
      confidence_level: 0.95
      min_effect_size: 0.05
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

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/learning-algorithm-service /usr/local/bin/
COPY config /config

EXPOSE 50070 9090

ENTRYPOINT ["learning-algorithm-service"]
```

### Cloud Run デプロイメント

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: learning-algorithm-service
  annotations:
    run.googleapis.com/launch-stage: GA
spec:
  template:
    metadata:
      annotations:
        run.googleapis.com/cpu-throttling: "false"
        run.googleapis.com/execution-environment: gen2
        autoscaling.knative.dev/minScale: "2"
        autoscaling.knative.dev/maxScale: "100"
        autoscaling.knative.dev/target: "80"
    spec:
      serviceAccountName: learning-algorithm
      containers:
        - image: gcr.io/effect-project/learning-algorithm-service:latest
          ports:
            - name: grpc
              containerPort: 50070
            - name: metrics
              containerPort: 9090
          env:
            - name: SERVICE_PORT
              value: "50070"
            - name: REDIS_URL
              valueFrom:
                secretKeyRef:
                  name: learning-algorithm-secrets
                  key: redis-url
          resources:
            requests:
              memory: "256Mi"
              cpu: "500m"
            limits:
              memory: "512Mi"
              cpu: "1000m"
          livenessProbe:
            grpc:
              port: 50070
              service: learning.LearningAlgorithmService
            periodSeconds: 10
            timeoutSeconds: 1
          readinessProbe:
            grpc:
              port: 50070
              service: learning.LearningAlgorithmService
            periodSeconds: 5
            timeoutSeconds: 1
          startupProbe:
            grpc:
              port: 50070
              service: learning.LearningAlgorithmService
            initialDelaySeconds: 5
            periodSeconds: 5
            failureThreshold: 3
```

## パフォーマンス最適化

### レスポンスタイム最適化

```rust
// アルゴリズムのプリコンパイル
lazy_static! {
    static ref ALGORITHM_CACHE: HashMap<String, Box<dyn LearningAlgorithm>> = {
        let mut cache = HashMap::new();
        cache.insert("SM-2".to_string(), Box::new(SM2Algorithm::default()));
        cache.insert("SM-2-modified".to_string(), Box::new(ModifiedSM2Algorithm::default()));
        cache.insert("Leitner".to_string(), Box::new(LeitnerAlgorithm::default()));
        cache
    };
}

// パラメータキャッシュのウォームアップ
pub async fn warm_cache(cache: &dyn ParameterCache) -> Result<()> {
    let popular_users = get_popular_users().await?;

    for user_id in popular_users {
        for algorithm in &["SM-2", "SM-2-modified"] {
            if let Some(params) = load_user_parameters(&user_id, algorithm).await? {
                cache.set_user_parameters(
                    &user_id,
                    algorithm,
                    &params,
                    Duration::from_secs(86400),
                ).await?;
            }
        }
    }

    Ok(())
}
```

### メモリ最適化

```rust
// オブジェクトプール
pub struct AlgorithmPool {
    sm2_pool: Pool<SM2Algorithm>,
    leitner_pool: Pool<LeitnerAlgorithm>,
}

impl AlgorithmPool {
    pub fn get_sm2(&self) -> PooledObject<SM2Algorithm> {
        self.sm2_pool.get()
    }

    pub fn get_leitner(&self) -> PooledObject<LeitnerAlgorithm> {
        self.leitner_pool.get()
    }
}
```

## 監視とアラート

### ヘルスチェック

```rust
#[derive(Serialize)]
struct HealthStatus {
    status: String,
    version: String,
    algorithms: Vec<AlgorithmStatus>,
    cache_connected: bool,
    active_experiments: u32,
}

async fn health_check(deps: &ServiceDependencies) -> HealthStatus {
    let cache_connected = deps.cache.ping().await.is_ok();

    let algorithm_statuses = deps.registry
        .list_algorithms()
        .iter()
        .map(|algo| AlgorithmStatus {
            name: algo.name().to_string(),
            version: algo.version().to_string(),
            enabled: true,
            health: "healthy".to_string(),
        })
        .collect();

    let active_experiments = deps.ab_test_manager
        .count_active_experiments()
        .await;

    HealthStatus {
        status: if cache_connected { "healthy" } else { "degraded" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        algorithms: algorithm_statuses,
        cache_connected,
        active_experiments,
    }
}
```

### メトリクス

```yaml
dashboards:
  - name: "Learning Algorithm Service"
    panels:
      - title: "Algorithm Usage"
        query: sum(rate(learning_algorithm_calculations_total[5m])) by (algorithm)

      - title: "Response Time (p95)"
        query: histogram_quantile(0.95, rate(learning_algorithm_calculation_duration_seconds_bucket[5m]))

      - title: "Cache Hit Rate"
        query: learning_algorithm_cache_hit_rate{cache_type="parameters"}

      - title: "Active Experiments"
        query: learning_algorithm_active_experiments

      - title: "Error Rate"
        query: sum(rate(learning_algorithm_calculations_total{status="error"}[5m])) / sum(rate(learning_algorithm_calculations_total[5m]))

alerts:
  - name: HighResponseTime
    expr: histogram_quantile(0.95, rate(learning_algorithm_calculation_duration_seconds_bucket[5m])) > 0.01
    severity: warning

  - name: LowCacheHitRate
    expr: learning_algorithm_cache_hit_rate{cache_type="parameters"} < 0.8
    severity: warning

  - name: HighErrorRate
    expr: |
      sum(rate(learning_algorithm_calculations_total{status="error"}[5m])) 
      / sum(rate(learning_algorithm_calculations_total[5m])) > 0.01
    severity: critical
```

## 更新履歴

- 2025-08-03: 初版作成
