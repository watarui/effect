//! gRPC service implementation for Algorithm Service

use chrono::Utc;
use tonic::{Request, Response, Status};

use crate::{
    domain::{
        services::sm2_calculator::Sm2Calculator,
        value_objects::{
            difficulty::Difficulty,
            easy_factor::EasyFactor,
            interval::Interval,
            repetition::Repetition,
        },
    },
    proto::effect::services::algorithm::{
        AdjustDifficultyRequest,
        AdjustDifficultyResponse,
        AdjustStrategyRequest,
        AdjustStrategyResponse,
        AnalyzeLearningTrendsRequest,
        AnalyzeLearningTrendsResponse,
        AnalyzePerformanceRequest,
        AnalyzePerformanceResponse,
        DifficultyAdjustment,
        GetDueItemsRequest,
        GetDueItemsResponse,
        GetItemHistoryRequest,
        GetItemHistoryResponse,
        GetLearningStatisticsRequest,
        GetLearningStatisticsResponse,
        GetLearningStrategyRequest,
        GetLearningStrategyResponse,
        IntervalAdjustment,
        LearningItemState,
        LearningStrategy,
        LearningTrends,
        PerformanceAnalysis,
        RescheduleItemRequest,
        RescheduleItemResponse,
        ScheduleNewItemRequest,
        ScheduleNewItemResponse,
        UserLearningStatistics,
        algorithm_service_server::AlgorithmService,
    },
};

/// gRPC サービス実装
#[derive(Debug, Clone)]
pub struct AlgorithmServiceImpl {
    // TODO: Add repository dependencies
}

impl AlgorithmServiceImpl {
    /// 新しいサービスインスタンスを作成
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// SM-2 アルゴリズムで難易度から初期値を計算
    const fn calculate_initial_difficulty(user_level: i32, item_level: i32) -> u8 {
        // ユーザーレベルと項目レベルの差から難易度を計算
        let diff = item_level - user_level;
        match diff {
            d if d <= -2 => 5, // Very Easy
            -1 => 4,           // Easy
            0 => 3,            // Normal
            1 => 2,            // Hard
            _ => 1,            // Very Hard
        }
    }

    /// `CorrectnessJudgment` から SM-2 の difficulty に変換
    const fn judgment_to_difficulty(judgment: i32) -> u8 {
        match judgment {
            1 => 0, // Incorrect
            2 => 2, // PartiallyCorrect
            3 => 4, // Correct
            4 => 5, // Perfect
            _ => 3, // Default to Normal
        }
    }
}

impl Default for AlgorithmServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl AlgorithmService for AlgorithmServiceImpl {
    /// 新規項目のスケジューリング
    async fn schedule_new_item(
        &self,
        request: Request<ScheduleNewItemRequest>,
    ) -> Result<Response<ScheduleNewItemResponse>, Status> {
        let req = request.into_inner();

        // 初期パラメータを設定
        let initial_difficulty =
            Self::calculate_initial_difficulty(req.user_level, req.difficulty_level);

        let difficulty = Difficulty::new(initial_difficulty)
            .map_err(|e| Status::invalid_argument(format!("Invalid difficulty: {e}")))?;

        let easy_factor = EasyFactor::initial();
        let repetition = Repetition::initial();
        let interval = Interval::first();
        let current_time = Utc::now();

        // SM-2 計算
        let result =
            Sm2Calculator::calculate(difficulty, repetition, interval, easy_factor, current_time);

        // LearningItemState を構築
        let state = LearningItemState {
            item_id:                  req.item_id.clone(),
            user_id:                  req.user_id.clone(),
            easiness_factor:          {
                #[allow(clippy::cast_possible_truncation)]
                let value = result.easy_factor.value() as f32;
                value
            },
            repetition_number:        result.repetition.count(),
            interval_days:            result.interval.days(),
            mastery_level:            1, // Beginner
            retention_rate:           0.0,
            next_review_date:         Some(prost_types::Timestamp {
                seconds: result.next_review_date.timestamp(),
                // timestamp_subsec_nanos() returns u32 (0..1_000_000_000), safe to cast to i32
                nanos:   i32::try_from(result.next_review_date.timestamp_subsec_nanos())
                    .unwrap_or(i32::MAX),
            }),
            last_reviewed_at:         Some(prost_types::Timestamp {
                seconds: current_time.timestamp(),
                // timestamp_subsec_nanos() returns u32 (0..1_000_000_000), safe to cast to i32
                nanos:   i32::try_from(current_time.timestamp_subsec_nanos()).unwrap_or(i32::MAX),
            }),
            total_reviews:            0,
            correct_count:            0,
            incorrect_count:          0,
            average_response_time_ms: 0.0,
            difficulty_level:         req.difficulty_level,
            is_problematic:           false,
        };

        // TODO: Repository に保存
        // TODO: イベント発行

        let response = ScheduleNewItemResponse {
            state:   Some(state),
            message: "Item scheduled successfully".to_string(),
        };

        Ok(Response::new(response))
    }

    /// 学習結果に基づくリスケジューリング
    async fn reschedule_item(
        &self,
        request: Request<RescheduleItemRequest>,
    ) -> Result<Response<RescheduleItemResponse>, Status> {
        let req = request.into_inner();

        // TODO: Repository から現在の状態を取得
        // 仮の実装
        let current_easy_factor = 2.5;
        let current_repetition = 1;
        let current_interval = 1;

        // CorrectnessJudgment を difficulty に変換
        let difficulty = Self::judgment_to_difficulty(req.judgment);
        let difficulty = Difficulty::new(difficulty)
            .map_err(|e| Status::invalid_argument(format!("Invalid difficulty: {e}")))?;

        let easy_factor = EasyFactor::new(current_easy_factor)
            .map_err(|e| Status::invalid_argument(format!("Invalid easy factor: {e}")))?;
        let repetition = Repetition::new(current_repetition);
        let interval = Interval::new(current_interval)
            .map_err(|e| Status::invalid_argument(format!("Invalid interval: {e}")))?;

        let current_time = Utc::now();

        // SM-2 計算
        let result =
            Sm2Calculator::calculate(difficulty, repetition, interval, easy_factor, current_time);

        // 更新された状態を構築
        let state = LearningItemState {
            item_id:                  req.item_id.clone(),
            user_id:                  req.user_id.clone(),
            easiness_factor:          {
                #[allow(clippy::cast_possible_truncation)]
                let value = result.easy_factor.value() as f32;
                value
            },
            repetition_number:        result.repetition.count(),
            interval_days:            result.interval.days(),
            mastery_level:            1,   // Beginner // TODO: 計算
            retention_rate:           0.0, // TODO: 計算
            next_review_date:         Some(prost_types::Timestamp {
                seconds: result.next_review_date.timestamp(),
                // timestamp_subsec_nanos() returns u32 (0..1_000_000_000), safe to cast to i32
                nanos:   i32::try_from(result.next_review_date.timestamp_subsec_nanos())
                    .unwrap_or(i32::MAX),
            }),
            last_reviewed_at:         Some(prost_types::Timestamp {
                seconds: current_time.timestamp(),
                // timestamp_subsec_nanos() returns u32 (0..1_000_000_000), safe to cast to i32
                nanos:   i32::try_from(current_time.timestamp_subsec_nanos()).unwrap_or(i32::MAX),
            }),
            total_reviews:            1, // TODO: インクリメント
            correct_count:            u32::from(req.judgment >= 3),
            incorrect_count:          u32::from(req.judgment <= 1),
            average_response_time_ms: {
                #[allow(clippy::cast_precision_loss)]
                let value = req.response_time_ms as f32;
                value
            },
            difficulty_level:         1,     // A1 // TODO: 取得
            is_problematic:           false, // TODO: 判定
        };

        let adjustment = IntervalAdjustment {
            old_interval_days:   current_interval,
            new_interval_days:   result.interval.days(),
            old_easiness_factor: {
                #[allow(clippy::cast_possible_truncation)]
                let value = current_easy_factor as f32;
                value
            },
            new_easiness_factor: {
                #[allow(clippy::cast_possible_truncation)]
                let value = result.easy_factor.value() as f32;
                value
            },
            reason:              if result.is_successful() {
                "Review successful".to_string()
            } else {
                "Review failed, resetting".to_string()
            },
        };

        // TODO: Repository に保存
        // TODO: イベント発行

        let response = RescheduleItemResponse {
            state:      Some(state),
            adjustment: Some(adjustment),
        };

        Ok(Response::new(response))
    }

    /// 復習対象項目の取得
    async fn get_due_items(
        &self,
        request: Request<GetDueItemsRequest>,
    ) -> Result<Response<GetDueItemsResponse>, Status> {
        let req = request.into_inner();
        let _limit = if req.limit == 0 { 20 } else { req.limit };

        // TODO: Repository から due items を取得
        // 仮の実装
        let due_items = vec![];

        let response = GetDueItemsResponse {
            due_items,
            total_due_count: 0,
        };

        Ok(Response::new(response))
    }

    /// 学習統計の取得
    async fn get_learning_statistics(
        &self,
        request: Request<GetLearningStatisticsRequest>,
    ) -> Result<Response<GetLearningStatisticsResponse>, Status> {
        let req = request.into_inner();

        // TODO: Repository から統計を取得
        // 仮の実装
        let statistics = UserLearningStatistics {
            user_id: req.user_id,
            total_items: 0,
            mastered_items: 0,
            learning_items: 0,
            new_items: 0,
            total_sessions: 0,
            total_reviews: 0,
            overall_accuracy: 0.0,
            average_session_duration_seconds: 0,
            daily_review_average: 0.0,
            current_streak_days: 0,
            longest_streak_days: 0,
            level_distribution: vec![],
        };

        let response = GetLearningStatisticsResponse {
            statistics: Some(statistics),
        };

        Ok(Response::new(response))
    }

    /// 難易度の調整
    async fn adjust_difficulty(
        &self,
        request: Request<AdjustDifficultyRequest>,
    ) -> Result<Response<AdjustDifficultyResponse>, Status> {
        let req = request.into_inner();

        // TODO: Repository から現在の状態を取得
        // TODO: 難易度調整ロジックの実装

        // 仮の実装
        let state = LearningItemState {
            item_id:                  req.item_id,
            user_id:                  req.user_id,
            easiness_factor:          2.5,
            repetition_number:        0,
            interval_days:            1,
            mastery_level:            1, // Beginner
            retention_rate:           0.0,
            next_review_date:         None,
            last_reviewed_at:         None,
            total_reviews:            0,
            correct_count:            0,
            incorrect_count:          0,
            average_response_time_ms: 0.0,
            difficulty_level:         1, // A1
            is_problematic:           false,
        };

        let adjustment = DifficultyAdjustment {
            old_factor:  2.5,
            new_factor:  req.suggested_factor.unwrap_or(2.5),
            explanation: "Difficulty adjusted based on performance".to_string(),
        };

        let response = AdjustDifficultyResponse {
            state:      Some(state),
            adjustment: Some(adjustment),
        };

        Ok(Response::new(response))
    }

    /// パフォーマンス分析
    async fn analyze_performance(
        &self,
        request: Request<AnalyzePerformanceRequest>,
    ) -> Result<Response<AnalyzePerformanceResponse>, Status> {
        let _req = request.into_inner();

        // TODO: Repository から履歴データを取得して分析
        // 仮の実装
        let analysis = PerformanceAnalysis {
            accuracy_trend:         0.0,
            speed_trend:            0.0,
            retention_trend:        0.0,
            problematic_categories: vec![],
            strong_categories:      vec![],
            active_hours:           vec![],
            consistency_score:      0.0,
        };

        let recommendations = vec![];

        let response = AnalyzePerformanceResponse {
            analysis: Some(analysis),
            recommendations,
        };

        Ok(Response::new(response))
    }

    /// 学習戦略の取得
    async fn get_learning_strategy(
        &self,
        request: Request<GetLearningStrategyRequest>,
    ) -> Result<Response<GetLearningStrategyResponse>, Status> {
        let req = request.into_inner();

        // TODO: Repository から戦略を取得
        // 仮の実装
        let strategy = LearningStrategy {
            user_id:               req.user_id,
            strategy_type:         0, // TODO: enum
            daily_target_items:    20,
            new_items_per_day:     5,
            difficulty_threshold:  0.7,
            learning_speed_factor: 1.0,
            retention_priority:    0.8,
            adaptive_scheduling:   true,
            last_adjusted_at:      None,
        };

        let response = GetLearningStrategyResponse {
            strategy: Some(strategy),
        };

        Ok(Response::new(response))
    }

    /// 学習戦略の調整
    async fn adjust_strategy(
        &self,
        request: Request<AdjustStrategyRequest>,
    ) -> Result<Response<AdjustStrategyResponse>, Status> {
        let req = request.into_inner();

        // TODO: Repository から現在の戦略を取得して更新
        // 仮の実装
        let strategy = LearningStrategy {
            user_id:               req.user_id,
            strategy_type:         req.new_strategy_type.unwrap_or(0),
            daily_target_items:    req.daily_target_items.unwrap_or(20),
            new_items_per_day:     req.new_items_per_day.unwrap_or(5),
            difficulty_threshold:  0.7,
            learning_speed_factor: req.learning_speed_factor.unwrap_or(1.0),
            retention_priority:    0.8,
            adaptive_scheduling:   true,
            last_adjusted_at:      Some(prost_types::Timestamp {
                seconds: Utc::now().timestamp(),
                nanos:   0,
            }),
        };

        let response = AdjustStrategyResponse {
            strategy:          Some(strategy),
            adjustment_reason: "Strategy adjusted based on user preferences".to_string(),
        };

        Ok(Response::new(response))
    }

    /// 項目の学習履歴取得
    async fn get_item_history(
        &self,
        request: Request<GetItemHistoryRequest>,
    ) -> Result<Response<GetItemHistoryResponse>, Status> {
        let req = request.into_inner();

        // TODO: Repository から履歴を取得
        // 仮の実装
        let history = vec![];

        let current_state = LearningItemState {
            item_id:                  req.item_id,
            user_id:                  req.user_id,
            easiness_factor:          2.5,
            repetition_number:        0,
            interval_days:            1,
            mastery_level:            1, // Beginner
            retention_rate:           0.0,
            next_review_date:         None,
            last_reviewed_at:         None,
            total_reviews:            0,
            correct_count:            0,
            incorrect_count:          0,
            average_response_time_ms: 0.0,
            difficulty_level:         1, // A1
            is_problematic:           false,
        };

        let response = GetItemHistoryResponse {
            history,
            current_state: Some(current_state),
        };

        Ok(Response::new(response))
    }

    /// ユーザーの学習傾向分析
    async fn analyze_learning_trends(
        &self,
        request: Request<AnalyzeLearningTrendsRequest>,
    ) -> Result<Response<AnalyzeLearningTrendsResponse>, Status> {
        let _req = request.into_inner();

        // TODO: Repository から履歴データを取得して傾向分析
        // 仮の実装
        let trends = LearningTrends {
            hourly_performance:     vec![],
            daily_performance:      vec![],
            category_progress:      vec![],
            predicted_mastery_days: 0,
            burnout_risk:           0.0,
        };

        let response = AnalyzeLearningTrendsResponse {
            trends: Some(trends),
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schedule_new_item() {
        let service = AlgorithmServiceImpl::new();

        let request = ScheduleNewItemRequest {
            user_id:          "user123".to_string(),
            item_id:          "item456".to_string(),
            difficulty_level: 3, // B1
            user_level:       2, // A2
        };

        let response = service
            .schedule_new_item(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.state.is_some());
        let state = response.state.unwrap();
        assert_eq!(state.interval_days, 1); // 初回は1日後
        assert_eq!(state.repetition_number, 0);
    }

    #[tokio::test]
    async fn test_reschedule_item_correct() {
        let service = AlgorithmServiceImpl::new();

        let request = RescheduleItemRequest {
            user_id:          "user123".to_string(),
            item_id:          "item456".to_string(),
            judgment:         3, // Correct
            response_time_ms: 2000,
            session_id:       Some("session789".to_string()),
        };

        let response = service
            .reschedule_item(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.state.is_some());
        assert!(response.adjustment.is_some());
    }

    #[tokio::test]
    async fn test_get_due_items() {
        let service = AlgorithmServiceImpl::new();

        let request = GetDueItemsRequest {
            user_id:  "user123".to_string(),
            limit:    10,
            as_of:    None,
            strategy: None,
        };

        let response = service
            .get_due_items(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.due_items.len(), 0); // TODO: 実装後に更新
    }
}
