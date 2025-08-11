use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 復習間隔のエラー
#[derive(Debug, Error)]
pub enum Error {
    /// 間隔が正の値でない
    #[error("Invalid interval value: {0}. Must be positive")]
    NonPositive(u32),
}

/// 復習間隔（日数）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interval(u32);

impl Interval {
    /// 初回の間隔（1日）
    pub const INITIAL_FIRST: u32 = 1;

    /// 2回目の間隔（6日）
    pub const INITIAL_SECOND: u32 = 6;

    /// 新しい Interval を作成
    ///
    /// # Errors
    ///
    /// 値が 0 の場合、`Error::NonPositive` を返します
    pub const fn new(days: u32) -> Result<Self, Error> {
        if days == 0 {
            return Err(Error::NonPositive(days));
        }
        Ok(Self(days))
    }

    /// 初回の間隔を作成
    #[must_use]
    pub const fn first() -> Self {
        Self(Self::INITIAL_FIRST)
    }

    /// 2回目の間隔を作成
    #[must_use]
    pub const fn second() -> Self {
        Self(Self::INITIAL_SECOND)
    }

    /// 値を取得（日数）
    #[must_use]
    pub const fn days(self) -> u32 {
        self.0
    }

    /// `EasyFactor` を使用して次の間隔を計算
    ///
    /// SM-2 アルゴリズムの公式:
    /// - n = 1: I(1) = 1
    /// - n = 2: I(2) = 6
    /// - n > 2: I(n) = I(n-1) * EF
    ///
    /// ただし、最大365日までに制限
    #[must_use]
    pub fn next(self, easy_factor: f64) -> Self {
        let next_days = (f64::from(self.0) * easy_factor).round();
        // 安全なキャスト: round() の結果を u32 の範囲内に制限
        let next_days = if next_days > f64::from(u32::MAX) {
            u32::MAX
        } else if next_days < 0.0 {
            0
        } else {
            #[allow(clippy::cast_possible_truncation)]
            #[allow(clippy::cast_sign_loss)]
            {
                next_days as u32
            }
        };
        // 最大365日（約1年）に制限
        Self(next_days.min(365))
    }

    /// 失敗時にリセット
    #[must_use]
    pub const fn reset() -> Self {
        Self::first()
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self::first()
    }
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} days", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_creation() {
        // 正常なケース
        let interval = Interval::new(10).unwrap();
        assert_eq!(interval.days(), 10);

        // エラーケース
        assert!(Interval::new(0).is_err());
    }

    #[test]
    fn test_interval_initial_values() {
        let first = Interval::first();
        assert_eq!(first.days(), 1);

        let second = Interval::second();
        assert_eq!(second.days(), 6);

        let default = Interval::default();
        assert_eq!(default.days(), 1);
    }

    #[test]
    fn test_interval_next() {
        // 初回から2回目への遷移（実際にはこのメソッドは使わない）
        let interval = Interval::first();
        let next = interval.next(2.5);
        assert_eq!(next.days(), 3); // 1 * 2.5 = 2.5 -> 3 (rounded)

        // 通常の遷移
        let interval = Interval::new(10).unwrap();
        let next = interval.next(2.5);
        assert_eq!(next.days(), 25); // 10 * 2.5 = 25

        // EasyFactor が低い場合
        let next = interval.next(1.3);
        assert_eq!(next.days(), 13); // 10 * 1.3 = 13

        // 最大値のテスト
        let interval = Interval::new(300).unwrap();
        let next = interval.next(2.0);
        assert_eq!(next.days(), 365); // 300 * 2.0 = 600 -> 365 (capped)
    }

    #[test]
    fn test_interval_reset() {
        let interval = Interval::new(100).unwrap();
        // 静的メソッドを使用
        let _ = interval; // unused variable を回避
        let reset = Interval::reset();
        assert_eq!(reset.days(), 1);
    }

    #[test]
    fn test_interval_display() {
        let interval = Interval::new(7).unwrap();
        assert_eq!(format!("{interval}"), "7 days");
    }
}
