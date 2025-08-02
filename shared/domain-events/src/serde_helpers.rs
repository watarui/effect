//! serde ヘルパーモジュール
//!
//! Protocol Buffers の Well-Known Types を JSON
//! シリアライゼーション可能にするための カスタム serde 実装を提供します。

#![allow(clippy::ref_option)] // protobuf の自動生成コードに合わせるため

/// 1秒あたりのナノ秒数
const NANOS_PER_SEC: i128 = 1_000_000_000;

/// google.protobuf.Timestamp の serde 実装
///
/// RFC3339 形式で JSON シリアライゼーション・デシリアライゼーションを行います。
pub mod timestamp {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    /// Timestamp を RFC3339 形式の文字列としてシリアライズ
    ///
    /// # Arguments
    ///
    /// * `timestamp` - シリアライズする Timestamp (Option)
    /// * `serializer` - serde Serializer
    ///
    /// # Returns
    ///
    /// シリアライズ結果
    ///
    /// # Errors
    ///
    /// 無効な timestamp 値の場合エラーを返す
    pub fn serialize<S>(
        timestamp: &Option<prost_types::Timestamp>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match timestamp {
            Some(ts) => {
                // prost_types::Timestamp から chrono::DateTime<Utc> に変換
                let nanos = u32::try_from(ts.nanos)
                    .map_err(|_| serde::ser::Error::custom("invalid nanos value"))?;
                let datetime = DateTime::<Utc>::from_timestamp(ts.seconds, nanos)
                    .ok_or_else(|| serde::ser::Error::custom("invalid timestamp"))?;

                // RFC3339 形式でシリアライズ
                serializer.serialize_str(&datetime.to_rfc3339())
            },
            None => serializer.serialize_none(),
        }
    }

    /// RFC3339 形式の文字列から Timestamp にデシリアライズ
    ///
    /// # Arguments
    ///
    /// * `deserializer` - serde Deserializer
    ///
    /// # Returns
    ///
    /// デシリアライズされた Timestamp (Option)
    ///
    /// # Errors
    ///
    /// 無効な RFC3339 文字列の場合エラーを返す
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<prost_types::Timestamp>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt_str: Option<String> = Option::deserialize(deserializer)?;

        match opt_str {
            Some(s) => {
                // RFC3339 形式の文字列をパース
                let datetime = DateTime::parse_from_rfc3339(&s)
                    .map_err(serde::de::Error::custom)?
                    .with_timezone(&Utc);

                // chrono::DateTime<Utc> から prost_types::Timestamp に変換
                let nanos = i32::try_from(datetime.timestamp_subsec_nanos())
                    .map_err(|_| serde::de::Error::custom("nanos overflow"))?;
                Ok(Some(prost_types::Timestamp {
                    seconds: datetime.timestamp(),
                    nanos,
                }))
            },
            None => Ok(None),
        }
    }
}

/// google.protobuf.Duration の serde 実装（将来の拡張用）
///
/// 秒とナノ秒を含む文字列形式（例: "1.5s"）でシリアライゼーションを行います。
pub mod duration {
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::NANOS_PER_SEC;

    /// Duration を文字列形式でシリアライズ
    ///
    /// # Arguments
    ///
    /// * `duration` - シリアライズする Duration (Option)
    /// * `serializer` - serde Serializer
    ///
    /// # Returns
    ///
    /// シリアライズ結果
    ///
    /// # Errors
    ///
    /// シリアライゼーションが失敗した場合エラーを返す
    pub fn serialize<S>(
        duration: &Option<prost_types::Duration>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => {
                let total_nanos = i128::from(d.seconds) * NANOS_PER_SEC + i128::from(d.nanos);
                // 精度を保つため、秒単位の整数部分と小数部分を別々に計算
                let whole_seconds = total_nanos / NANOS_PER_SEC;
                let fractional_nanos = total_nanos % NANOS_PER_SEC;
                #[allow(clippy::cast_precision_loss)]
                let seconds =
                    whole_seconds as f64 + (fractional_nanos as f64 / NANOS_PER_SEC as f64);
                serializer.serialize_str(&format!("{seconds}s"))
            },
            None => serializer.serialize_none(),
        }
    }

    /// 文字列形式から Duration にデシリアライズ
    ///
    /// # Arguments
    ///
    /// * `deserializer` - serde Deserializer
    ///
    /// # Returns
    ///
    /// デシリアライズされた Duration (Option)
    ///
    /// # Errors
    ///
    /// 無効な duration フォーマットの場合エラーを返す
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<prost_types::Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt_str: Option<String> = Option::deserialize(deserializer)?;

        match opt_str {
            Some(s) => {
                // "1.5s" のような形式をパース
                let s = s.trim_end_matches('s');
                let seconds_f64: f64 = s
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid duration format"))?;

                // 精度を保つため、整数部分と小数部分を別々に処理
                #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
                let total_nanos = (seconds_f64 * NANOS_PER_SEC as f64) as i128;
                let seconds = i64::try_from(total_nanos / NANOS_PER_SEC)
                    .map_err(|_| serde::de::Error::custom("seconds overflow"))?;
                let nanos = i32::try_from(total_nanos % NANOS_PER_SEC)
                    .map_err(|_| serde::de::Error::custom("nanos overflow"))?;

                Ok(Some(prost_types::Duration { seconds, nanos }))
            },
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestTimestamp {
        #[serde(
            with = "super::timestamp",
            skip_serializing_if = "Option::is_none",
            default
        )]
        ts: Option<prost_types::Timestamp>,
    }

    #[test]
    fn test_timestamp_serialization() {
        let test = TestTimestamp {
            ts: Some(prost_types::Timestamp {
                seconds: 1_609_459_200, // 2021-01-01T00:00:00Z
                nanos:   0,
            }),
        };

        let json = serde_json::to_string(&test).unwrap();
        assert!(json.contains("2021-01-01T00:00:00"));

        let deserialized: TestTimestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, test);
    }

    #[test]
    fn test_timestamp_none_serialization() {
        let test = TestTimestamp { ts: None };

        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r"{}"); // skip_serializing_if = "Option::is_none" により空のJSONになる

        let deserialized: TestTimestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, test);
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestDuration {
        #[serde(with = "super::duration", skip_serializing_if = "Option::is_none")]
        dur: Option<prost_types::Duration>,
    }

    #[test]
    fn test_duration_serialization() {
        let test = TestDuration {
            dur: Some(prost_types::Duration {
                seconds: 1,
                nanos:   500_000_000,
            }),
        };

        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"dur":"1.5s"}"#);

        let deserialized: TestDuration = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, test);
    }
}
