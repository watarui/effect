use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 難易度評価のエラー
#[derive(Debug, Error)]
pub enum Error {
    /// 無効な難易度値（0-5の範囲外）
    #[error("Invalid difficulty value: {0}. Must be between 0 and 5")]
    InvalidValue(u8),
}

/// 難易度評価（0-5）
///
/// SM-2 アルゴリズムでの回答の質を表す
/// - 0: 完全に忘れた
/// - 1: 間違えたが、正解を見て思い出した
/// - 2: 間違えたが、正解を見て簡単だと感じた
/// - 3: 正解したが、思い出すのに苦労した
/// - 4: 正解したが、ためらいがあった
/// - 5: 完璧に思い出した
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Difficulty(u8);

impl Difficulty {
    /// 新しい Difficulty を作成
    ///
    /// # Errors
    ///
    /// 値が 0-5 の範囲外の場合、`Error::InvalidValue` を返します
    pub const fn new(value: u8) -> Result<Self, Error> {
        if value > 5 {
            return Err(Error::InvalidValue(value));
        }
        Ok(Self(value))
    }

    /// 値を取得
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// 正解かどうか（3以上で正解とみなす）
    #[must_use]
    pub const fn is_correct(self) -> bool {
        self.0 >= 3
    }

    /// 簡単だったかどうか（4以上で簡単とみなす）
    #[must_use]
    pub const fn is_easy(self) -> bool {
        self.0 >= 4
    }

    /// 難しかったかどうか（2以下で難しいとみなす）
    #[must_use]
    pub const fn is_hard(self) -> bool {
        self.0 <= 2
    }
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_creation() {
        // 正常なケース
        for i in 0..=5 {
            let difficulty = Difficulty::new(i).unwrap();
            assert_eq!(difficulty.value(), i);
        }

        // エラーケース
        assert!(Difficulty::new(6).is_err());
        assert!(Difficulty::new(100).is_err());
    }

    #[test]
    fn test_difficulty_is_correct() {
        assert!(!Difficulty::new(0).unwrap().is_correct());
        assert!(!Difficulty::new(1).unwrap().is_correct());
        assert!(!Difficulty::new(2).unwrap().is_correct());
        assert!(Difficulty::new(3).unwrap().is_correct());
        assert!(Difficulty::new(4).unwrap().is_correct());
        assert!(Difficulty::new(5).unwrap().is_correct());
    }

    #[test]
    fn test_difficulty_is_easy() {
        assert!(!Difficulty::new(0).unwrap().is_easy());
        assert!(!Difficulty::new(3).unwrap().is_easy());
        assert!(Difficulty::new(4).unwrap().is_easy());
        assert!(Difficulty::new(5).unwrap().is_easy());
    }

    #[test]
    fn test_difficulty_is_hard() {
        assert!(Difficulty::new(0).unwrap().is_hard());
        assert!(Difficulty::new(1).unwrap().is_hard());
        assert!(Difficulty::new(2).unwrap().is_hard());
        assert!(!Difficulty::new(3).unwrap().is_hard());
        assert!(!Difficulty::new(5).unwrap().is_hard());
    }
}
