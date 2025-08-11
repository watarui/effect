use chrono::{DateTime, Utc};

use crate::domain::value_objects::{Difficulty, EasyFactor, Interval, Repetition, Sm2Result};

/// SM-2 アルゴリズム計算サービス
///
/// `SuperMemo` 2 アルゴリズムの実装
/// 間隔反復学習システムの中核となる計算ロジック
#[derive(Debug, Clone)]
pub struct Sm2Calculator;

impl Sm2Calculator {
    /// SM-2 アルゴリズムを実行
    ///
    /// # Arguments
    /// - `difficulty`: 回答の難易度評価 (0-5)
    /// - `current_repetition`: 現在の学習回数
    /// - `current_interval`: 現在の復習間隔
    /// - `current_easy_factor`: 現在の `EasyFactor`
    /// - `current_date`: 現在の日時
    ///
    /// # Returns
    /// 計算結果を含む `Sm2Result`
    #[must_use]
    pub fn calculate(
        difficulty: Difficulty,
        current_repetition: Repetition,
        current_interval: Interval,
        current_easy_factor: EasyFactor,
        current_date: DateTime<Utc>,
    ) -> Sm2Result {
        // 難易度評価が正解（3以上）の場合
        if difficulty.is_correct() {
            // 学習回数をインクリメント
            let new_repetition = current_repetition.increment();

            // 新しい間隔を計算
            let new_interval =
                Self::calculate_interval(new_repetition, current_interval, current_easy_factor);

            // EasyFactor を更新
            let new_easy_factor = current_easy_factor.update(difficulty.value());

            Sm2Result::new(new_interval, new_easy_factor, new_repetition, current_date)
        } else {
            // 不正解の場合はリセット
            Self::reset(current_easy_factor, difficulty, current_date)
        }
    }

    /// 新規項目の初回学習
    #[must_use]
    pub fn initial_learning(difficulty: Difficulty, current_date: DateTime<Utc>) -> Sm2Result {
        let initial_easy_factor = EasyFactor::initial();

        if difficulty.is_correct() {
            // 初回学習で正解
            let new_repetition = Repetition::new(1);
            let new_interval = Interval::first();
            let new_easy_factor = initial_easy_factor.update(difficulty.value());

            Sm2Result::new(new_interval, new_easy_factor, new_repetition, current_date)
        } else {
            // 初回学習で不正解
            Self::reset(initial_easy_factor, difficulty, current_date)
        }
    }

    /// 間隔を計算
    fn calculate_interval(
        repetition: Repetition,
        current_interval: Interval,
        easy_factor: EasyFactor,
    ) -> Interval {
        match repetition.count() {
            1 => Interval::first(),                          // 初回は1日後
            2 => Interval::second(),                         // 2回目は6日後
            _ => current_interval.next(easy_factor.value()), // 3回目以降は EF を使用
        }
    }

    /// リセット処理
    fn reset(
        current_easy_factor: EasyFactor,
        difficulty: Difficulty,
        current_date: DateTime<Utc>,
    ) -> Sm2Result {
        // 学習回数をリセット
        let new_repetition = Repetition::reset();

        // 間隔をリセット（1日に戻す）
        let new_interval = Interval::reset();

        // EasyFactor は更新（ただし減少させる）
        let new_easy_factor = current_easy_factor.update(difficulty.value());

        Sm2Result::new(new_interval, new_easy_factor, new_repetition, current_date)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    fn setup() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn test_initial_learning_correct() {
        let now = setup();

        // 初回学習で正解（難易度4）
        let difficulty = Difficulty::new(4).unwrap();
        let result = Sm2Calculator::initial_learning(difficulty, now);

        assert_eq!(result.repetition.count(), 1);
        assert_eq!(result.interval.days(), 1);
        assert_relative_eq!(result.easy_factor.value(), 2.5); // 初期値2.5から変化なし
    }

    #[test]
    fn test_initial_learning_perfect() {
        let now = setup();

        // 初回学習で完璧（難易度5）
        let difficulty = Difficulty::new(5).unwrap();
        let result = Sm2Calculator::initial_learning(difficulty, now);

        assert_eq!(result.repetition.count(), 1);
        assert_eq!(result.interval.days(), 1);
        assert!(result.easy_factor.value() > 2.5); // EF が増加
    }

    #[test]
    fn test_initial_learning_incorrect() {
        let now = setup();

        // 初回学習で不正解（難易度2）
        let difficulty = Difficulty::new(2).unwrap();
        let result = Sm2Calculator::initial_learning(difficulty, now);

        assert_eq!(result.repetition.count(), 0); // リセット
        assert_eq!(result.interval.days(), 1);
        assert!(result.easy_factor.value() < 2.5); // EF が減少
    }

    #[test]
    fn test_second_review_correct() {
        let now = setup();

        // 2回目の復習で正解
        let difficulty = Difficulty::new(4).unwrap();
        let current_repetition = Repetition::new(1);
        let current_interval = Interval::first();
        let current_easy_factor = EasyFactor::initial();

        let result = Sm2Calculator::calculate(
            difficulty,
            current_repetition,
            current_interval,
            current_easy_factor,
            now,
        );

        assert_eq!(result.repetition.count(), 2);
        assert_eq!(result.interval.days(), 6); // 2回目は6日後
        assert_relative_eq!(result.easy_factor.value(), 2.5);
    }

    #[test]
    fn test_third_review_correct() {
        let now = setup();

        // 3回目の復習で正解
        let difficulty = Difficulty::new(4).unwrap();
        let current_repetition = Repetition::new(2);
        let current_interval = Interval::second();
        let current_easy_factor = EasyFactor::initial();

        let result = Sm2Calculator::calculate(
            difficulty,
            current_repetition,
            current_interval,
            current_easy_factor,
            now,
        );

        assert_eq!(result.repetition.count(), 3);
        assert_eq!(result.interval.days(), 15); // 6 * 2.5 = 15
        assert_relative_eq!(result.easy_factor.value(), 2.5);
    }

    #[test]
    fn test_review_incorrect_resets() {
        let now = setup();

        // 何回目でも不正解ならリセット
        let difficulty = Difficulty::new(1).unwrap();
        let current_repetition = Repetition::new(5);
        let current_interval = Interval::new(30).unwrap();
        let current_easy_factor = EasyFactor::new(2.8).unwrap();

        let result = Sm2Calculator::calculate(
            difficulty,
            current_repetition,
            current_interval,
            current_easy_factor,
            now,
        );

        assert_eq!(result.repetition.count(), 0); // リセット
        assert_eq!(result.interval.days(), 1); // 1日に戻る
        assert!(result.easy_factor.value() < 2.8); // EF は減少
    }

    #[test]
    fn test_difficulty_affects_easy_factor() {
        let now = setup();

        let current_repetition = Repetition::new(3);
        let current_interval = Interval::new(15).unwrap();
        let current_easy_factor = EasyFactor::initial();

        // 難易度3で正解
        let diff3 = Difficulty::new(3).unwrap();
        let result3 = Sm2Calculator::calculate(
            diff3,
            current_repetition,
            current_interval,
            current_easy_factor,
            now,
        );

        // 難易度5で正解
        let diff5 = Difficulty::new(5).unwrap();
        let result5 = Sm2Calculator::calculate(
            diff5,
            current_repetition,
            current_interval,
            current_easy_factor,
            now,
        );

        // 難易度が高いほど EasyFactor が増加
        assert!(result5.easy_factor.value() > result3.easy_factor.value());
    }

    #[test]
    fn test_next_review_date() {
        let now = setup();

        let difficulty = Difficulty::new(4).unwrap();
        let result = Sm2Calculator::initial_learning(difficulty, now);

        // 次の復習日は現在日時 + 間隔日数
        let expected = now + chrono::Duration::days(1);
        assert_eq!(result.next_review_date.date_naive(), expected.date_naive());
    }
}
