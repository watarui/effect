use serde::{Deserialize, Serialize};
use thiserror::Error;

/// `EasyFactor` のエラー
#[derive(Debug, Error)]
pub enum Error {
    /// 値が小さすぎる（1.3未満）
    #[error("Invalid easy factor value: {0}. Must be at least 1.3")]
    TooLow(f64),
}

/// `EasyFactor` (E-Factor)
///
/// SM-2 アルゴリズムにおける難易度係数
/// - 最小値: 1.3
/// - 初期値: 2.5
/// - 値が高いほど項目が簡単であることを示す
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EasyFactor(f64);

impl EasyFactor {
    /// 最小値
    pub const MIN_VALUE: f64 = 1.3;

    /// 初期値
    pub const INITIAL_VALUE: f64 = 2.5;

    /// 新しい `EasyFactor` を作成
    ///
    /// # Errors
    ///
    /// 値が 1.3 未満の場合、`Error::TooLow` を返します
    pub fn new(value: f64) -> Result<Self, Error> {
        if value < Self::MIN_VALUE {
            return Err(Error::TooLow(value));
        }
        Ok(Self(value))
    }

    /// 初期値で作成
    #[must_use]
    pub const fn initial() -> Self {
        Self(Self::INITIAL_VALUE)
    }

    /// 値を取得
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// 難易度評価に基づいて更新
    ///
    /// SM-2 アルゴリズムの公式:
    /// EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
    ///
    /// ただし:
    /// - q: 難易度評価 (0-5)
    /// - EF': 新しい `EasyFactor`
    /// - EF: 現在の `EasyFactor`
    ///
    /// 難易度が3未満の場合は、さらに修正を加える
    #[must_use]
    pub fn update(self, difficulty: u8) -> Self {
        let q = f64::from(difficulty);

        // 基本の公式
        let delta = (5.0 - q).mul_add(-(5.0 - q).mul_add(0.02, 0.08), 0.1);
        let mut new_value = self.0 + delta;

        // 難易度が低い場合の追加調整
        if difficulty < 3 {
            // 不正解の場合、より大きく減少させる
            new_value *= 0.8;
        }

        // 最小値を保証
        new_value = new_value.max(Self::MIN_VALUE);

        Self(new_value)
    }
}

impl Default for EasyFactor {
    fn default() -> Self {
        Self::initial()
    }
}

impl std::fmt::Display for EasyFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_easy_factor_creation() {
        // 正常なケース
        let ef = EasyFactor::new(2.0).unwrap();
        assert_relative_eq!(ef.value(), 2.0);

        let ef = EasyFactor::new(1.3).unwrap();
        assert_relative_eq!(ef.value(), 1.3);

        // エラーケース
        assert!(EasyFactor::new(1.2).is_err());
        assert!(EasyFactor::new(0.0).is_err());
    }

    #[test]
    fn test_easy_factor_initial() {
        let ef = EasyFactor::initial();
        assert_relative_eq!(ef.value(), 2.5);

        let ef = EasyFactor::default();
        assert_relative_eq!(ef.value(), 2.5);
    }

    #[test]
    fn test_easy_factor_update() {
        let ef = EasyFactor::initial();

        // 完璧な回答 (5) -> EasyFactor が増加
        let ef_updated = ef.update(5);
        assert!(ef_updated.value() > ef.value());
        assert_relative_eq!(ef_updated.value(), 2.6, epsilon = 0.01);

        // 難しい回答 (0) -> EasyFactor が大幅に減少（不正解なので0.8倍される）
        let ef_updated = ef.update(0);
        assert!(ef_updated.value() < ef.value());
        // 2.5 + (-0.8) = 1.7, さらに 0.8倍 = 1.36
        assert_relative_eq!(ef_updated.value(), 1.36, epsilon = 0.01);

        // 最小値以下にならないことを確認
        let ef_low = EasyFactor::new(1.3).unwrap();
        let ef_updated = ef_low.update(0);
        assert_relative_eq!(ef_updated.value(), EasyFactor::MIN_VALUE);
    }

    #[test]
    fn test_easy_factor_update_all_difficulties() {
        let ef = EasyFactor::initial(); // 2.5

        // 各難易度での更新値を確認
        // 不正解（0-2）の場合は0.8倍される
        let test_cases = [
            (0, 1.360), // (2.5 - 0.8) * 0.8 = 1.36
            (1, 1.568), // (2.5 - 0.54) * 0.8 = 1.568
            (2, 1.744), // (2.5 - 0.32) * 0.8 = 1.744
            (3, 2.360), // 2.5 - 0.14 = 2.36
            (4, 2.500), // 変化なし
            (5, 2.600), // 2.5 + 0.1 = 2.6
        ];

        for (difficulty, expected) in test_cases {
            let updated = ef.update(difficulty);
            assert_relative_eq!(updated.value(), expected, epsilon = 0.01);
        }
    }
}
