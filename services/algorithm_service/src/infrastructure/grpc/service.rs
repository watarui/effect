//! gRPC service implementation for Algorithm Service

use std::sync::Arc;

use chrono::{Datelike, Timelike, Utc};
use tonic::{Request, Response, Status};

use super::conversion::i32_to_u32;
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
#[derive(Clone)]
pub struct AlgorithmServiceImpl {
    /// データベース接続プール
    #[allow(dead_code)]
    db_pool:             sqlx::PgPool,
    /// 学習項目リポジトリ
    learning_item_repo:  Arc<dyn super::super::repository::learning_item::LearningItemRepository>,
    /// 復習履歴リポジトリ
    review_history_repo: Arc<dyn super::super::repository::review_history::ReviewHistoryRepository>,
    /// 統計リポジトリ
    statistics_repo:     Arc<dyn super::super::repository::statistics::StatisticsRepository>,
    /// 戦略リポジトリ
    strategy_repo:       Arc<dyn super::super::repository::strategy::LearningStrategyRepository>,
}

impl AlgorithmServiceImpl {
    /// データベースプールを使用してサービスインスタンスを作成
    #[must_use]
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        use super::super::repository::{
            learning_item::PostgresRepository as LearningItemRepo,
            review_history::PostgresRepository as ReviewHistoryRepo,
            statistics::PostgresRepository as StatisticsRepo,
            strategy::PostgresRepository as StrategyRepo,
        };

        Self {
            learning_item_repo: Arc::new(LearningItemRepo::new(db_pool.clone())),
            review_history_repo: Arc::new(ReviewHistoryRepo::new(db_pool.clone())),
            statistics_repo: Arc::new(StatisticsRepo::new(db_pool.clone())),
            strategy_repo: Arc::new(StrategyRepo::new(db_pool.clone())),
            db_pool,
        }
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
                #[allow(clippy::cast_possible_wrap)]
                nanos: result.next_review_date.timestamp_subsec_nanos() as i32,
            }),
            last_reviewed_at:         Some(prost_types::Timestamp {
                seconds: current_time.timestamp(),
                // timestamp_subsec_nanos() returns u32 (0..1_000_000_000), safe to cast to i32
                #[allow(clippy::cast_possible_wrap)]
                nanos: current_time.timestamp_subsec_nanos() as i32,
            }),
            total_reviews:            0,
            correct_count:            0,
            incorrect_count:          0,
            average_response_time_ms: 0.0,
            difficulty_level:         req.difficulty_level,
            is_problematic:           false,
        };

        // UUIDの解析
        let user_uuid = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {e}")))?;
        let item_uuid = uuid::Uuid::parse_str(&req.item_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid item_id: {e}")))?;

        // データベースに保存するためのエンティティを作成
        let db_state = super::super::repository::learning_item::LearningItemState {
            id: uuid::Uuid::new_v4(),
            user_id: user_uuid,
            item_id: item_uuid,
            #[allow(clippy::cast_possible_truncation)]
            easiness_factor: result.easy_factor.value() as f32,
            #[allow(clippy::cast_possible_wrap)]
            repetition_number: result.repetition.count() as i32,
            #[allow(clippy::cast_possible_wrap)]
            interval_days: result.interval.days() as i32,
            mastery_level: 1,
            retention_rate: 0.0,
            next_review_date: Some(result.next_review_date),
            last_reviewed_at: Some(current_time),
            total_reviews: 0,
            correct_count: 0,
            incorrect_count: 0,
            average_response_time_ms: 0.0,
            difficulty_level: req.difficulty_level,
            is_problematic: false,
            version: 1,
            created_at: current_time,
            updated_at: current_time,
        };

        // Repository に保存
        self.learning_item_repo
            .create(&db_state)
            .await
            .map_err(|e| Status::internal(format!("Failed to save learning item: {e}")))?;

        // TODO: イベント発行

        let response = ScheduleNewItemResponse {
            state:   Some(state),
            message: "Item scheduled successfully".to_string(),
        };

        Ok(Response::new(response))
    }

    /// 学習結果に基づくリスケジューリング
    #[allow(clippy::too_many_lines)]
    async fn reschedule_item(
        &self,
        request: Request<RescheduleItemRequest>,
    ) -> Result<Response<RescheduleItemResponse>, Status> {
        let req = request.into_inner();

        // UUIDの解析
        let user_uuid = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {e}")))?;
        let item_uuid = uuid::Uuid::parse_str(&req.item_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid item_id: {e}")))?;

        // Repository から現在の状態を取得
        let current_state = self
            .learning_item_repo
            .find_by_user_and_item(user_uuid, item_uuid)
            .await
            .map_err(|e| Status::internal(format!("Failed to get learning item: {e}")))?
            .ok_or_else(|| Status::not_found("Learning item not found"))?;

        let current_easy_factor = f64::from(current_state.easiness_factor);
        let current_repetition = current_state.repetition_number;
        let current_interval = current_state.interval_days;

        // CorrectnessJudgment を difficulty に変換
        let difficulty = Self::judgment_to_difficulty(req.judgment);
        let difficulty = Difficulty::new(difficulty)
            .map_err(|e| Status::invalid_argument(format!("Invalid difficulty: {e}")))?;

        let easy_factor = EasyFactor::new(current_easy_factor)
            .map_err(|e| Status::invalid_argument(format!("Invalid easy factor: {e}")))?;
        let repetition =
            Repetition::new(u32::try_from(current_repetition).map_err(|e| {
                Status::invalid_argument(format!("Invalid repetition number: {e}"))
            })?);
        let interval = Interval::new(
            u32::try_from(current_interval)
                .map_err(|e| Status::invalid_argument(format!("Invalid interval: {e}")))?,
        )
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
                #[allow(clippy::cast_possible_wrap)]
                nanos: result.next_review_date.timestamp_subsec_nanos() as i32,
            }),
            last_reviewed_at:         Some(prost_types::Timestamp {
                seconds: current_time.timestamp(),
                // timestamp_subsec_nanos() returns u32 (0..1_000_000_000), safe to cast to i32
                #[allow(clippy::cast_possible_wrap)]
                nanos: current_time.timestamp_subsec_nanos() as i32,
            }),
            total_reviews:            i32_to_u32(current_state.total_reviews + 1)?,
            correct_count:            i32_to_u32(current_state.correct_count)?
                + u32::from(req.judgment >= 3),
            incorrect_count:          i32_to_u32(current_state.incorrect_count)?
                + u32::from(req.judgment <= 1),
            average_response_time_ms: {
                #[allow(clippy::cast_precision_loss)]
                let value = req.response_time_ms as f32;
                value
            },
            difficulty_level:         current_state.difficulty_level,
            is_problematic:           req.judgment <= 1 || current_state.is_problematic,
        };

        let adjustment = IntervalAdjustment {
            old_interval_days:   i32_to_u32(current_interval)?,
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

        // データベースの状態を更新
        let mut updated_state = current_state;
        #[allow(clippy::cast_possible_truncation)]
        {
            updated_state.easiness_factor = result.easy_factor.value() as f32;
        }
        updated_state.repetition_number = {
            #[allow(clippy::cast_possible_wrap)]
            let value = result.repetition.count() as i32;
            value
        };
        updated_state.interval_days = {
            #[allow(clippy::cast_possible_wrap)]
            let value = result.interval.days() as i32;
            value
        };
        updated_state.next_review_date = Some(result.next_review_date);
        updated_state.last_reviewed_at = Some(current_time);
        updated_state.total_reviews += 1;
        updated_state.correct_count += i32::from(req.judgment >= 3);
        updated_state.incorrect_count += i32::from(req.judgment <= 1);

        // 平均応答時間を更新
        #[allow(clippy::cast_precision_loss)]
        {
            let total = updated_state.total_reviews as f32;
            let prev_avg = updated_state.average_response_time_ms;
            updated_state.average_response_time_ms =
                prev_avg.mul_add(total - 1.0, req.response_time_ms as f32) / total;
        }

        updated_state.is_problematic = req.judgment <= 1 || updated_state.is_problematic;
        updated_state.updated_at = current_time;
        updated_state.version += 1;

        // 習熟レベルを更新
        updated_state.mastery_level = match result.repetition.count() {
            0..=2 => 1,   // Beginner
            3..=5 => 2,   // Learning
            6..=10 => 3,  // Familiar
            11..=20 => 4, // Proficient
            _ => 5,       // Mastered
        };

        // Repository に保存
        self.learning_item_repo
            .update(&updated_state)
            .await
            .map_err(|e| Status::internal(format!("Failed to update learning item: {e}")))?;

        // 復習履歴を保存
        let session_uuid = req
            .session_id
            .as_ref()
            .map(|s| uuid::Uuid::parse_str(s))
            .transpose()
            .map_err(|e| Status::invalid_argument(format!("Invalid session_id: {e}")))?;

        let review_history = super::super::repository::review_history::ReviewHistory {
            id: uuid::Uuid::new_v4(),
            user_id: user_uuid,
            item_id: item_uuid,
            reviewed_at: current_time,
            judgment: req.judgment,
            response_time_ms: {
                #[allow(clippy::cast_possible_wrap)]
                let value = req.response_time_ms as i32;
                value
            },
            interval_days: current_interval,
            #[allow(clippy::cast_possible_truncation)]
            easiness_factor: current_easy_factor as f32,
            session_id: session_uuid,
            created_at: current_time,
        };

        self.review_history_repo
            .create(&review_history)
            .await
            .map_err(|e| Status::internal(format!("Failed to save review history: {e}")))?;

        // TODO: イベント発行

        let response = RescheduleItemResponse {
            state:      Some(state),
            adjustment: Some(adjustment),
        };

        Ok(Response::new(response))
    }

    /// 復習対象項目の取得
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    async fn get_due_items(
        &self,
        request: Request<GetDueItemsRequest>,
    ) -> Result<Response<GetDueItemsResponse>, Status> {
        let req = request.into_inner();
        let limit = if req.limit == 0 { 20 } else { req.limit };

        // UUIDの解析
        let user_uuid = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {e}")))?;

        // 基準時刻を決定（指定がなければ現在時刻）
        let as_of = req
            .as_of
            .and_then(|ts| chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32))
            .unwrap_or_else(Utc::now);

        // Repository から due items を取得
        let db_items = self
            .learning_item_repo
            .get_due_items(user_uuid, as_of, i64::from(limit))
            .await
            .map_err(|e| Status::internal(format!("Failed to get due items: {e}")))?;

        // データベースの状態をプロトコルバッファ形式に変換
        let due_items = db_items
            .into_iter()
            .map(
                |item| -> Result<crate::proto::effect::services::algorithm::DueItem, Status> {
                    // 期限超過日数を計算
                    let overdue_days = item.next_review_date.map_or(0, |next_review| {
                        let days_diff = (as_of - next_review).num_days();
                        if days_diff > 0 { days_diff as u32 } else { 0 }
                    });

                    // 優先度スコアを計算（期限超過日数と習熟レベルから）
                    let priority_score =
                        (overdue_days as f32 + 1.0) * (6 - item.mastery_level) as f32 / 5.0;

                    let state = LearningItemState {
                        item_id:                  item.item_id.to_string(),
                        user_id:                  item.user_id.to_string(),
                        easiness_factor:          item.easiness_factor,
                        repetition_number:        i32_to_u32(item.repetition_number)?,
                        interval_days:            i32_to_u32(item.interval_days)?,
                        mastery_level:            item.mastery_level,
                        retention_rate:           item.retention_rate,
                        next_review_date:         item.next_review_date.map(|dt| {
                            prost_types::Timestamp {
                                seconds: dt.timestamp(),
                                nanos:   {
                                    #[allow(clippy::cast_possible_wrap)]
                                    let value = dt.timestamp_subsec_nanos() as i32;
                                    value
                                },
                            }
                        }),
                        last_reviewed_at:         item.last_reviewed_at.map(|dt| {
                            prost_types::Timestamp {
                                seconds: dt.timestamp(),
                                nanos:   {
                                    #[allow(clippy::cast_possible_wrap)]
                                    let value = dt.timestamp_subsec_nanos() as i32;
                                    value
                                },
                            }
                        }),
                        total_reviews:            i32_to_u32(item.total_reviews)?,
                        correct_count:            i32_to_u32(item.correct_count)?,
                        incorrect_count:          i32_to_u32(item.incorrect_count)?,
                        average_response_time_ms: item.average_response_time_ms,
                        difficulty_level:         item.difficulty_level,
                        is_problematic:           item.is_problematic,
                    };

                    Ok(crate::proto::effect::services::algorithm::DueItem {
                        state: Some(state),
                        overdue_days,
                        priority_score,
                    })
                },
            )
            .collect::<Result<Vec<_>, _>>()?;

        let total_due_count = due_items.len() as u32;

        let response = GetDueItemsResponse {
            due_items,
            total_due_count,
        };

        Ok(Response::new(response))
    }

    /// 学習統計の取得
    async fn get_learning_statistics(
        &self,
        request: Request<GetLearningStatisticsRequest>,
    ) -> Result<Response<GetLearningStatisticsResponse>, Status> {
        let req = request.into_inner();

        // UUIDの解析
        let user_uuid = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {e}")))?;

        // Repository から統計を取得
        let db_statistics = self
            .statistics_repo
            .get_user_statistics(user_uuid)
            .await
            .map_err(|e| Status::internal(format!("Failed to get statistics: {e}")))?;

        let statistics = if let Some(stats) = db_statistics {
            // 既存の統計データを使用
            UserLearningStatistics {
                user_id: req.user_id,
                total_items: i32_to_u32(stats.total_items)?,
                mastered_items: i32_to_u32(stats.mastered_items)?,
                learning_items: i32_to_u32(stats.learning_items)?,
                new_items: i32_to_u32(stats.new_items)?,
                total_sessions: i32_to_u32(stats.total_sessions)?,
                total_reviews: i32_to_u32(stats.total_reviews)?,
                overall_accuracy: stats.overall_accuracy,
                average_session_duration_seconds: i32_to_u32(
                    stats.average_session_duration_seconds,
                )?,
                daily_review_average: stats.daily_review_average,
                current_streak_days: i32_to_u32(stats.current_streak_days)?,
                longest_streak_days: i32_to_u32(stats.longest_streak_days)?,
                level_distribution: vec![], // TODO: レベル分布の実装
            }
        } else {
            // 統計がない場合は、learning_item_states から集計
            let item_counts = self
                .learning_item_repo
                .count_by_user(user_uuid)
                .await
                .map_err(|e| Status::internal(format!("Failed to count items: {e}")))?;

            UserLearningStatistics {
                user_id: req.user_id,
                total_items: i32_to_u32(item_counts.total)?,
                mastered_items: i32_to_u32(item_counts.mastered)?,
                learning_items: i32_to_u32(item_counts.learning)?,
                new_items: i32_to_u32(item_counts.new)?,
                total_sessions: 0, // セッション数は履歴から計算が必要
                total_reviews: 0,  // レビュー数も履歴から計算が必要
                overall_accuracy: 0.0,
                average_session_duration_seconds: 0,
                daily_review_average: 0.0,
                current_streak_days: 0,
                longest_streak_days: 0,
                level_distribution: vec![],
            }
        };

        let response = GetLearningStatisticsResponse {
            statistics: Some(statistics),
        };

        Ok(Response::new(response))
    }

    /// 難易度の調整
    #[allow(clippy::too_many_lines)]
    async fn adjust_difficulty(
        &self,
        request: Request<AdjustDifficultyRequest>,
    ) -> Result<Response<AdjustDifficultyResponse>, Status> {
        let req = request.into_inner();

        // UUIDの解析
        let user_uuid = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {e}")))?;
        let item_uuid = uuid::Uuid::parse_str(&req.item_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid item_id: {e}")))?;

        // Repository から現在の状態を取得
        let mut current_state = self
            .learning_item_repo
            .find_by_user_and_item(user_uuid, item_uuid)
            .await
            .map_err(|e| Status::internal(format!("Failed to get learning item: {e}")))?
            .ok_or_else(|| Status::not_found("Learning item not found"))?;

        let old_factor = current_state.easiness_factor;

        // 新しい easiness_factor を決定
        let new_factor = if let Some(suggested) = req.suggested_factor {
            // 提案された値を検証（SM-2 の範囲: 1.3 - 2.5）
            suggested.clamp(1.3, 2.5)
        } else {
            // 調整理由に基づいて自動調整
            match req.reason {
                1 => {
                    // TooEasy
                    // 難易度を上げる（係数を下げる）
                    (old_factor - 0.2).max(1.3)
                },
                2 => {
                    // TooHard
                    // 難易度を下げる（係数を上げる）
                    (old_factor + 0.2).min(2.5)
                },
                3 => {
                    // RepeatedFailure
                    // 大幅に難易度を下げる
                    (old_factor + 0.4).min(2.5)
                },
                4 => {
                    // RapidMastery
                    // 大幅に難易度を上げる
                    (old_factor - 0.4).max(1.3)
                },
                5 => {
                    // UserFeedback
                    // ユーザーフィードバックの場合は suggested_factor が必須
                    old_factor
                },
                _ => old_factor,
            }
        };

        // 説明文を生成
        let explanation = match req.reason {
            1 => format!(
                "項目が簡単すぎるため、難易度を上げました（係数: {old_factor:.2} → \
                 {new_factor:.2}）"
            ),
            2 => format!(
                "項目が難しすぎるため、難易度を下げました（係数: {old_factor:.2} → \
                 {new_factor:.2}）"
            ),
            3 => format!(
                "繰り返し失敗しているため、難易度を大幅に下げました（係数: {old_factor:.2} → \
                 {new_factor:.2}）"
            ),
            4 => format!(
                "急速に習得されたため、難易度を大幅に上げました（係数: {old_factor:.2} → \
                 {new_factor:.2}）"
            ),
            5 => format!(
                "ユーザーフィードバックに基づき調整しました（係数: {old_factor:.2} → \
                 {new_factor:.2}）"
            ),
            _ => format!("難易度を調整しました（係数: {old_factor:.2} → {new_factor:.2}）"),
        };

        // 状態を更新
        current_state.easiness_factor = new_factor;
        current_state.updated_at = Utc::now();
        current_state.version += 1;

        // 新しい係数で次回復習日を再計算
        let difficulty = Difficulty::new(3) // Normal difficulty as baseline
            .map_err(|e| Status::internal(format!("Failed to create difficulty: {e}")))?;
        let easy_factor = EasyFactor::new(f64::from(new_factor))
            .map_err(|e| Status::internal(format!("Failed to create easy factor: {e}")))?;
        let repetition = Repetition::new(
            u32::try_from(current_state.repetition_number)
                .map_err(|e| Status::internal(format!("Invalid repetition number: {e}")))?,
        );
        let interval = Interval::new(
            u32::try_from(current_state.interval_days)
                .map_err(|e| Status::internal(format!("Invalid interval days: {e}")))?,
        )
        .map_err(|e| Status::internal(format!("Failed to create interval: {e}")))?;

        let result = Sm2Calculator::calculate(
            difficulty,
            repetition,
            interval,
            easy_factor,
            current_state.last_reviewed_at.unwrap_or_else(Utc::now),
        );

        current_state.next_review_date = Some(result.next_review_date);
        current_state.interval_days = {
            #[allow(clippy::cast_possible_wrap)]
            let value = result.interval.days() as i32;
            value
        };

        // Repository に保存
        self.learning_item_repo
            .update(&current_state)
            .await
            .map_err(|e| Status::internal(format!("Failed to update learning item: {e}")))?;

        // レスポンス用に変換
        let state = LearningItemState {
            item_id:                  req.item_id.clone(),
            user_id:                  req.user_id.clone(),
            easiness_factor:          current_state.easiness_factor,
            repetition_number:        i32_to_u32(current_state.repetition_number)?,
            interval_days:            i32_to_u32(current_state.interval_days)?,
            mastery_level:            current_state.mastery_level,
            retention_rate:           current_state.retention_rate,
            next_review_date:         current_state.next_review_date.map(|dt| {
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos:   {
                        #[allow(clippy::cast_possible_wrap)]
                        let value = dt.timestamp_subsec_nanos() as i32;
                        value
                    },
                }
            }),
            last_reviewed_at:         current_state.last_reviewed_at.map(|dt| {
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos:   {
                        #[allow(clippy::cast_possible_wrap)]
                        let value = dt.timestamp_subsec_nanos() as i32;
                        value
                    },
                }
            }),
            total_reviews:            i32_to_u32(current_state.total_reviews + 1)?,
            correct_count:            i32_to_u32(current_state.correct_count)?,
            incorrect_count:          i32_to_u32(current_state.incorrect_count)?,
            average_response_time_ms: current_state.average_response_time_ms,
            difficulty_level:         current_state.difficulty_level,
            is_problematic:           current_state.is_problematic,
        };

        let adjustment = DifficultyAdjustment {
            old_factor,
            new_factor,
            explanation,
        };

        let response = AdjustDifficultyResponse {
            state:      Some(state),
            adjustment: Some(adjustment),
        };

        Ok(Response::new(response))
    }

    /// パフォーマンス分析
    #[allow(
        clippy::too_many_lines,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    async fn analyze_performance(
        &self,
        request: Request<AnalyzePerformanceRequest>,
    ) -> Result<Response<AnalyzePerformanceResponse>, Status> {
        let req = request.into_inner();
        let recent_sessions = if req.recent_sessions == 0 {
            10
        } else {
            req.recent_sessions
        };

        // UUIDの解析
        let user_uuid = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {e}")))?;

        // 最近の復習履歴を取得
        let recent_reviews = self.review_history_repo
            .get_recent_reviews_for_user(user_uuid, i64::from(recent_sessions) * 20) // 1セッション約20項目と仮定
            .await
            .map_err(|e| Status::internal(format!("Failed to get review history: {e}")))?;

        if recent_reviews.is_empty() {
            // 履歴がない場合はデフォルト値を返す
            let analysis = PerformanceAnalysis {
                accuracy_trend:         0.0,
                speed_trend:            0.0,
                retention_trend:        0.0,
                problematic_categories: vec![],
                strong_categories:      vec![],
                active_hours:           vec![],
                consistency_score:      0.0,
            };

            let response = AnalyzePerformanceResponse {
                analysis:        Some(analysis),
                recommendations: vec![],
            };

            return Ok(Response::new(response));
        }

        // パフォーマンス指標を計算
        let total_reviews = recent_reviews.len() as f32;
        let correct_reviews = recent_reviews.iter().filter(|r| r.judgment >= 3).count() as f32;
        let current_accuracy = correct_reviews / total_reviews;

        // 時系列で前半と後半に分けて傾向を分析
        let mid_point = recent_reviews.len() / 2;
        let first_half = &recent_reviews[..mid_point];
        let second_half = &recent_reviews[mid_point..];

        // 正答率の傾向
        let first_half_correct = first_half.iter().filter(|r| r.judgment >= 3).count() as f32;
        let second_half_correct = second_half.iter().filter(|r| r.judgment >= 3).count() as f32;
        let first_half_accuracy = if first_half.is_empty() {
            0.0
        } else {
            first_half_correct / first_half.len() as f32
        };
        let second_half_accuracy = if second_half.is_empty() {
            0.0
        } else {
            second_half_correct / second_half.len() as f32
        };
        let accuracy_trend = second_half_accuracy - first_half_accuracy;

        // 応答速度の傾向
        let first_half_avg_time: f32 = if first_half.is_empty() {
            0.0
        } else {
            first_half
                .iter()
                .map(|r| r.response_time_ms as f32)
                .sum::<f32>()
                / first_half.len() as f32
        };
        let second_half_avg_time: f32 = if second_half.is_empty() {
            0.0
        } else {
            second_half
                .iter()
                .map(|r| r.response_time_ms as f32)
                .sum::<f32>()
                / second_half.len() as f32
        };
        // 速度が改善（短くなる）していれば正の値
        let speed_trend = if first_half_avg_time > 0.0 {
            (first_half_avg_time - second_half_avg_time) / first_half_avg_time
        } else {
            0.0
        };

        // 定着率の傾向（間隔が長くなっているかどうか）
        let first_half_avg_interval: f32 = if first_half.is_empty() {
            0.0
        } else {
            first_half
                .iter()
                .map(|r| r.interval_days as f32)
                .sum::<f32>()
                / first_half.len() as f32
        };
        let second_half_avg_interval: f32 = if second_half.is_empty() {
            0.0
        } else {
            second_half
                .iter()
                .map(|r| r.interval_days as f32)
                .sum::<f32>()
                / second_half.len() as f32
        };
        let retention_trend = if first_half_avg_interval > 0.0 {
            (second_half_avg_interval - first_half_avg_interval) / first_half_avg_interval
        } else {
            0.0
        };

        // アクティブな時間帯を分析
        let mut hour_counts = [0u32; 24];
        for review in &recent_reviews {
            let hour = review.reviewed_at.hour();
            hour_counts[hour as usize] += 1;
        }
        let active_hours: Vec<u32> = hour_counts
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            .map(|(hour, _)| hour as u32)
            .collect();

        // 一貫性スコア（日ごとの学習の安定性）
        let mut daily_reviews = std::collections::HashMap::new();
        for review in &recent_reviews {
            let date = review.reviewed_at.date_naive();
            *daily_reviews.entry(date).or_insert(0) += 1;
        }
        let avg_daily = total_reviews / daily_reviews.len().max(1) as f32;
        let variance: f32 = daily_reviews
            .values()
            .map(|&count| {
                let diff = count as f32 - avg_daily;
                diff * diff
            })
            .sum::<f32>()
            / daily_reviews.len().max(1) as f32;
        let std_dev = variance.sqrt();
        let consistency_score = if avg_daily > 0.0 {
            1.0 - (std_dev / avg_daily).min(1.0)
        } else {
            0.0
        };

        // 推奨事項を生成
        let mut recommendations = vec![];

        if accuracy_trend < -0.1 {
            recommendations.push(
                crate::proto::effect::services::algorithm::PerformanceRecommendation {
                    recommendation_type: "accuracy_decline".to_string(),
                    description:         "正答率が低下傾向にあります。難易度を調整するか、\
                                          復習間隔を短くすることを検討してください。"
                        .to_string(),
                    impact_score:        0.8,
                },
            );
        }

        if speed_trend < -0.2 {
            recommendations.push(
                crate::proto::effect::services::algorithm::PerformanceRecommendation {
                    recommendation_type: "speed_decline".to_string(),
                    description:         "回答速度が遅くなっています。疲労の可能性があるため、\
                                          休憩を取ることをお勧めします。"
                        .to_string(),
                    impact_score:        0.6,
                },
            );
        }

        if consistency_score < 0.5 {
            recommendations.push(
                crate::proto::effect::services::algorithm::PerformanceRecommendation {
                    recommendation_type: "inconsistent_learning".to_string(),
                    description:         "学習の一貫性が低いです。\
                                          毎日決まった時間に学習することを心がけましょう。"
                        .to_string(),
                    impact_score:        0.7,
                },
            );
        }

        if current_accuracy > 0.9 && retention_trend > 0.2 {
            recommendations.push(
                crate::proto::effect::services::algorithm::PerformanceRecommendation {
                    recommendation_type: "increase_difficulty".to_string(),
                    description:         "優れたパフォーマンスを示しています。\
                                          より難しい項目に挑戦してみましょう。"
                        .to_string(),
                    impact_score:        0.9,
                },
            );
        }

        let analysis = PerformanceAnalysis {
            accuracy_trend,
            speed_trend,
            retention_trend,
            problematic_categories: vec![], // TODO: カテゴリ別分析の実装
            strong_categories: vec![],      // TODO: カテゴリ別分析の実装
            active_hours,
            consistency_score,
        };

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

        // UUIDの解析
        let user_uuid = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {e}")))?;

        // Repository から戦略を取得、なければデフォルトを作成
        let db_strategy = self
            .strategy_repo
            .get_or_create_default(user_uuid)
            .await
            .map_err(|e| Status::internal(format!("Failed to get learning strategy: {e}")))?;

        let strategy = LearningStrategy {
            user_id:               req.user_id,
            strategy_type:         db_strategy.strategy_type,
            daily_target_items:    i32_to_u32(db_strategy.daily_target_items)?,
            new_items_per_day:     i32_to_u32(db_strategy.new_items_per_day)?,
            difficulty_threshold:  db_strategy.difficulty_threshold,
            learning_speed_factor: db_strategy.learning_speed_factor,
            retention_priority:    db_strategy.retention_priority,
            adaptive_scheduling:   db_strategy.adaptive_scheduling,
            last_adjusted_at:      db_strategy
                .last_adjusted_at
                .map(|dt| prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos:   {
                        #[allow(clippy::cast_possible_wrap)]
                        let value = dt.timestamp_subsec_nanos() as i32;
                        value
                    },
                }),
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

        // UUIDの解析
        let user_uuid = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {e}")))?;

        // Repository から現在の戦略を取得
        let mut db_strategy = self
            .strategy_repo
            .find_by_user(user_uuid)
            .await
            .map_err(|e| Status::internal(format!("Failed to get strategy: {e}")))?
            .ok_or_else(|| Status::not_found("Learning strategy not found"))?;

        // 戦略を更新
        if let Some(strategy_type) = req.new_strategy_type {
            db_strategy.strategy_type = strategy_type;
        }
        if let Some(daily_target) = req.daily_target_items {
            db_strategy.daily_target_items = {
                #[allow(clippy::cast_possible_wrap)]
                let value = daily_target as i32;
                value
            };
        }
        if let Some(new_items) = req.new_items_per_day {
            db_strategy.new_items_per_day = {
                #[allow(clippy::cast_possible_wrap)]
                let value = new_items as i32;
                value
            };
        }
        if let Some(speed_factor) = req.learning_speed_factor {
            db_strategy.learning_speed_factor = speed_factor;
        }

        let current_time = Utc::now();
        db_strategy.last_adjusted_at = Some(current_time);
        db_strategy.updated_at = current_time;
        db_strategy.version += 1;

        // Repository に保存
        self.strategy_repo
            .update(&db_strategy)
            .await
            .map_err(|e| Status::internal(format!("Failed to update strategy: {e}")))?;

        // レスポンス用に変換
        let strategy = LearningStrategy {
            user_id:               req.user_id,
            strategy_type:         db_strategy.strategy_type,
            daily_target_items:    i32_to_u32(db_strategy.daily_target_items)?,
            new_items_per_day:     i32_to_u32(db_strategy.new_items_per_day)?,
            difficulty_threshold:  db_strategy.difficulty_threshold,
            learning_speed_factor: db_strategy.learning_speed_factor,
            retention_priority:    db_strategy.retention_priority,
            adaptive_scheduling:   db_strategy.adaptive_scheduling,
            last_adjusted_at:      Some(prost_types::Timestamp {
                seconds: current_time.timestamp(),
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
        let limit = if req.limit == 0 { 20 } else { req.limit };

        // UUIDの解析
        let user_uuid = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {e}")))?;
        let item_uuid = uuid::Uuid::parse_str(&req.item_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid item_id: {e}")))?;

        // Repository から履歴を取得
        let db_histories = self
            .review_history_repo
            .get_by_user_and_item(user_uuid, item_uuid, i64::from(limit))
            .await
            .map_err(|e| Status::internal(format!("Failed to get review history: {e}")))?;

        // 履歴データをプロトコルバッファ形式に変換
        let history: Vec<crate::proto::effect::services::algorithm::ReviewHistory> = db_histories
            .into_iter()
            .map(
                |h| -> Result<crate::proto::effect::services::algorithm::ReviewHistory, Status> {
                    Ok(crate::proto::effect::services::algorithm::ReviewHistory {
                        reviewed_at:      Some(prost_types::Timestamp {
                            seconds: h.reviewed_at.timestamp(),
                            nanos:   {
                                #[allow(clippy::cast_possible_wrap)]
                                let value = h.reviewed_at.timestamp_subsec_nanos() as i32;
                                value
                            },
                        }),
                        judgment:         h.judgment,
                        response_time_ms: i32_to_u32(h.response_time_ms)?,
                        interval_days:    i32_to_u32(h.interval_days)?,
                        easiness_factor:  h.easiness_factor,
                        session_id:       h.session_id.map(|id| id.to_string()),
                    })
                },
            )
            .collect::<Result<Vec<_>, _>>()?;

        // 現在の学習状態を取得
        let current_state = if let Some(db_state) = self
            .learning_item_repo
            .find_by_user_and_item(user_uuid, item_uuid)
            .await
            .map_err(|e| Status::internal(format!("Failed to get learning item state: {e}")))?
        {
            Some(LearningItemState {
                item_id:                  req.item_id.clone(),
                user_id:                  req.user_id.clone(),
                easiness_factor:          db_state.easiness_factor,
                repetition_number:        i32_to_u32(db_state.repetition_number)?,
                interval_days:            i32_to_u32(db_state.interval_days)?,
                mastery_level:            db_state.mastery_level,
                retention_rate:           db_state.retention_rate,
                next_review_date:         db_state.next_review_date.map(|dt| {
                    prost_types::Timestamp {
                        seconds: dt.timestamp(),
                        nanos:   {
                            #[allow(clippy::cast_possible_wrap)]
                            let value = dt.timestamp_subsec_nanos() as i32;
                            value
                        },
                    }
                }),
                last_reviewed_at:         db_state.last_reviewed_at.map(|dt| {
                    prost_types::Timestamp {
                        seconds: dt.timestamp(),
                        nanos:   {
                            #[allow(clippy::cast_possible_wrap)]
                            let value = dt.timestamp_subsec_nanos() as i32;
                            value
                        },
                    }
                }),
                total_reviews:            i32_to_u32(db_state.total_reviews)?,
                correct_count:            i32_to_u32(db_state.correct_count)?,
                incorrect_count:          i32_to_u32(db_state.incorrect_count)?,
                average_response_time_ms: db_state.average_response_time_ms,
                difficulty_level:         db_state.difficulty_level,
                is_problematic:           db_state.is_problematic,
            })
        } else {
            None
        };

        let response = GetItemHistoryResponse {
            history,
            current_state,
        };

        Ok(Response::new(response))
    }

    /// ユーザーの学習傾向分析
    #[allow(
        clippy::too_many_lines,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    async fn analyze_learning_trends(
        &self,
        request: Request<AnalyzeLearningTrendsRequest>,
    ) -> Result<Response<AnalyzeLearningTrendsResponse>, Status> {
        let req = request.into_inner();
        let days_to_analyze = if req.days_to_analyze == 0 {
            30
        } else {
            req.days_to_analyze
        };

        // UUIDの解析
        let user_uuid = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {e}")))?;

        // 分析期間の復習履歴を取得
        let since = Utc::now() - chrono::Duration::days(i64::from(days_to_analyze));
        let reviews = self
            .review_history_repo
            .get_reviews_since(user_uuid, since)
            .await
            .map_err(|e| Status::internal(format!("Failed to get review history: {e}")))?;

        if reviews.is_empty() {
            // 履歴がない場合はデフォルト値を返す
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

            return Ok(Response::new(response));
        }

        // 時間帯別パフォーマンスを分析
        let mut hourly_stats: std::collections::HashMap<u32, (u32, u32, f32)> =
            std::collections::HashMap::new();
        for review in &reviews {
            let hour = review.reviewed_at.hour();
            let entry = hourly_stats.entry(hour).or_insert((0, 0, 0.0));
            entry.0 += 1; // total count
            if review.judgment >= 3 {
                entry.1 += 1; // correct count
            }
            entry.2 += review.response_time_ms as f32;
        }

        let hourly_performance: Vec<crate::proto::effect::services::algorithm::HourlyPerformance> =
            hourly_stats
                .iter()
                .map(|(&hour, &(total, correct, total_time))| {
                    crate::proto::effect::services::algorithm::HourlyPerformance {
                        hour,
                        accuracy_rate: correct as f32 / total as f32,
                        review_count: total,
                        average_response_time_ms: total_time / total as f32,
                    }
                })
                .collect();

        // 曜日別パフォーマンスを分析
        let mut daily_stats: std::collections::HashMap<u32, (u32, u32, u32)> =
            std::collections::HashMap::new();
        for review in &reviews {
            let day_of_week = review.reviewed_at.weekday().num_days_from_sunday();
            let entry = daily_stats.entry(day_of_week).or_insert((0, 0, 0));
            entry.0 += 1; // total reviews
            if review.judgment >= 3 {
                entry.1 += 1; // correct reviews
            }
            entry.2 += 1; // days with activity
        }

        let daily_performance: Vec<crate::proto::effect::services::algorithm::DailyPerformance> =
            daily_stats
                .iter()
                .map(|(&day, &(total, correct, active_days))| {
                    let days_in_period = days_to_analyze / 7 + 1; // 週の数
                    crate::proto::effect::services::algorithm::DailyPerformance {
                        day_of_week:     day,
                        accuracy_rate:   correct as f32 / total.max(1) as f32,
                        average_reviews: total / days_in_period,
                        completion_rate: active_days as f32 / days_in_period as f32,
                    }
                })
                .collect();

        // 全学習項目を取得して進捗を計算
        let all_items = self
            .learning_item_repo
            .find_by_user(user_uuid)
            .await
            .map_err(|e| Status::internal(format!("Failed to get learning items: {e}")))?;

        // カテゴリ別進捗（現時点では品詞情報がないので、難易度レベル別に集計）
        let mut level_stats: std::collections::HashMap<i32, (u32, u32, f32)> =
            std::collections::HashMap::new();
        for item in &all_items {
            let entry = level_stats
                .entry(item.difficulty_level)
                .or_insert((0, 0, 0.0));
            entry.0 += 1; // total items
            if item.mastery_level >= 4 {
                // Proficient以上を習得とみなす
                entry.1 += 1; // mastered items
            }
            entry.2 += item.easiness_factor;
        }

        let category_progress: Vec<crate::proto::effect::services::algorithm::CategoryProgress> =
            level_stats
                .iter()
                .map(|(&level, &(total, mastered, total_easiness))| {
                    let level_name = match level {
                        1 => "A1",
                        2 => "A2",
                        3 => "B1",
                        4 => "B2",
                        5 => "C1",
                        6 => "C2",
                        _ => "Unknown",
                    };
                    crate::proto::effect::services::algorithm::CategoryProgress {
                        category:                level_name.to_string(),
                        total_items:             total,
                        mastered_items:          mastered,
                        progress_rate:           mastered as f32 / total.max(1) as f32,
                        average_easiness_factor: total_easiness / total.max(1) as f32,
                    }
                })
                .collect();

        // 習得予測日数を計算
        let total_items = all_items.len() as f32;
        let mastered_items = all_items.iter().filter(|i| i.mastery_level >= 4).count() as f32;
        let progress_rate = mastered_items / total_items.max(1.0);

        let days_since_start = days_to_analyze as f32;
        let daily_progress = progress_rate / days_since_start;
        let remaining_progress = 1.0 - progress_rate;
        let predicted_mastery_days = if daily_progress > 0.0 {
            (remaining_progress / daily_progress) as u32
        } else {
            0
        };

        // バーンアウトリスクを計算
        let recent_week_reviews = reviews
            .iter()
            .filter(|r| r.reviewed_at > Utc::now() - chrono::Duration::days(7))
            .count() as f32;
        let avg_weekly_reviews = reviews.len() as f32 / (days_to_analyze as f32 / 7.0);

        // 最近の復習数が平均の2倍を超えたらリスク上昇
        let overload_factor = if avg_weekly_reviews > 0.0 {
            (recent_week_reviews / avg_weekly_reviews - 1.0).max(0.0)
        } else {
            0.0
        };

        // 最近の正答率低下もリスク要因
        let recent_accuracy = reviews
            .iter()
            .filter(|r| r.reviewed_at > Utc::now() - chrono::Duration::days(7))
            .map(|r| if r.judgment >= 3 { 1.0 } else { 0.0 })
            .sum::<f32>()
            / recent_week_reviews.max(1.0);
        let overall_accuracy = reviews
            .iter()
            .map(|r| if r.judgment >= 3 { 1.0 } else { 0.0 })
            .sum::<f32>()
            / reviews.len().max(1) as f32;
        let accuracy_decline = (overall_accuracy - recent_accuracy).max(0.0);

        let burnout_risk =
            ((overload_factor * 0.6 + accuracy_decline * 0.4) * 100.0).min(100.0) / 100.0;

        let trends = LearningTrends {
            hourly_performance,
            daily_performance,
            category_progress,
            predicted_mastery_days,
            burnout_risk,
        };

        let response = AnalyzeLearningTrendsResponse {
            trends: Some(trends),
        };

        Ok(Response::new(response))
    }
}
