use serde::{Deserialize, Serialize};

/// 学習回数
///
/// SM-2 アルゴリズムでの復習回数を追跡
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repetition(u32);

impl Repetition {
    /// 新しい Repetition を作成
    #[must_use]
    pub const fn new(count: u32) -> Self {
        Self(count)
    }

    /// 初期値（0回）
    #[must_use]
    pub const fn initial() -> Self {
        Self(0)
    }

    /// 値を取得
    #[must_use]
    pub const fn count(self) -> u32 {
        self.0
    }

    /// インクリメント
    #[must_use]
    pub const fn increment(self) -> Self {
        Self(self.0 + 1)
    }

    /// リセット
    #[must_use]
    pub const fn reset() -> Self {
        Self::initial()
    }

    /// 初回学習かどうか
    #[must_use]
    pub const fn is_first(self) -> bool {
        self.0 == 0
    }

    /// 2回目の学習かどうか
    #[must_use]
    pub const fn is_second(self) -> bool {
        self.0 == 1
    }
}

impl Default for Repetition {
    fn default() -> Self {
        Self::initial()
    }
}

impl std::fmt::Display for Repetition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repetition_creation() {
        let rep = Repetition::new(5);
        assert_eq!(rep.count(), 5);

        let rep = Repetition::initial();
        assert_eq!(rep.count(), 0);

        let rep = Repetition::default();
        assert_eq!(rep.count(), 0);
    }

    #[test]
    fn test_repetition_increment() {
        let rep = Repetition::initial();
        assert_eq!(rep.count(), 0);

        let rep = rep.increment();
        assert_eq!(rep.count(), 1);

        let rep = rep.increment();
        assert_eq!(rep.count(), 2);
    }

    #[test]
    fn test_repetition_reset() {
        let rep = Repetition::new(10);
        // 静的メソッドを使用
        let _ = rep; // unused variable を回避
        let reset = Repetition::reset();
        assert_eq!(reset.count(), 0);
    }

    #[test]
    fn test_repetition_is_first_second() {
        let rep = Repetition::initial();
        assert!(rep.is_first());
        assert!(!rep.is_second());

        let rep = rep.increment();
        assert!(!rep.is_first());
        assert!(rep.is_second());

        let rep = rep.increment();
        assert!(!rep.is_first());
        assert!(!rep.is_second());
    }
}
