use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::{EasyFactor, Interval, Repetition};

/// SM-2 アルゴリズムの計算結果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sm2Result {
    /// 次の復習間隔（日数）
    pub interval: Interval,

    /// 更新された `EasyFactor`
    pub easy_factor: EasyFactor,

    /// 更新された学習回数
    pub repetition: Repetition,

    /// 次の復習日
    pub next_review_date: DateTime<Utc>,
}

impl Sm2Result {
    /// 新しい結果を作成
    #[must_use]
    pub fn new(
        interval: Interval,
        easy_factor: EasyFactor,
        repetition: Repetition,
        current_date: DateTime<Utc>,
    ) -> Self {
        let next_review_date = current_date + Duration::days(i64::from(interval.days()));

        Self {
            interval,
            easy_factor,
            repetition,
            next_review_date,
        }
    }

    /// 次の復習までの日数を取得
    #[must_use]
    pub const fn days_until_review(&self) -> u32 {
        self.interval.days()
    }

    /// 学習が成功したかどうか（リセットされていない）
    #[must_use]
    pub const fn is_successful(&self) -> bool {
        self.repetition.count() > 0
    }
}
