//! タイムスタンプユーティリティ
//!
//! このモジュールはタイムスタンプ関連の型とユーティリティを含みます。

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

/// アプリケーション全体で使用される標準タイムスタンプ型
pub type Timestamp = DateTime<Utc>;

/// 現在時刻で新しいタイムスタンプを作成
#[must_use]
pub fn now() -> Timestamp {
    Utc::now()
}

/// タイムスタンプを JST として扱うための拡張トレイト
pub trait JstExt {
    /// JST でフォーマットした文字列を返す
    fn format_jst(&self, fmt: &str) -> String;

    /// JST の `DateTime` に変換
    fn to_jst(&self) -> DateTime<FixedOffset>;

    /// JST での日付部分を取得
    fn date_jst(&self) -> NaiveDate;
}

impl JstExt for Timestamp {
    fn format_jst(&self, fmt: &str) -> String {
        self.to_jst().format(fmt).to_string()
    }

    fn to_jst(&self) -> DateTime<FixedOffset> {
        // UTC+9 のオフセットを作成
        // 9時間 = 32400秒は必ず有効な範囲内なので、
        // east_opt が None を返すことはない
        #[allow(clippy::unwrap_used)]
        let jst = FixedOffset::east_opt(9 * 3600).unwrap();
        self.with_timezone(&jst)
    }

    fn date_jst(&self) -> NaiveDate {
        self.to_jst().date_naive()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, TimeZone, Timelike};

    use super::*;

    #[test]
    fn now_should_create_current_timestamp() {
        let before = Utc::now();
        let timestamp = now();
        let after = Utc::now();

        assert!(timestamp >= before);
        assert!(timestamp <= after);
    }

    #[test]
    fn jst_conversion_should_add_9_hours() {
        let utc_time = Utc.with_ymd_and_hms(2024, 1, 15, 15, 0, 0).unwrap();
        let jst_time = utc_time.to_jst();

        assert_eq!(jst_time.hour(), 0); // 15:00 UTC = 00:00 JST（翌日）
        assert_eq!(jst_time.day(), 16);
    }

    #[test]
    fn format_jst_should_format_in_jst() {
        let utc_time = Utc.with_ymd_and_hms(2024, 1, 15, 15, 30, 0).unwrap();
        let formatted = utc_time.format_jst("%Y-%m-%d %H:%M:%S");

        assert_eq!(formatted, "2024-01-16 00:30:00");
    }

    #[test]
    fn date_jst_should_return_jst_date() {
        // UTC で 1月15日 22:00 = JST で 1月16日 07:00
        let utc_time = Utc.with_ymd_and_hms(2024, 1, 15, 22, 0, 0).unwrap();
        let jst_date = utc_time.date_jst();

        assert_eq!(jst_date.year(), 2024);
        assert_eq!(jst_date.month(), 1);
        assert_eq!(jst_date.day(), 16);
    }
}
