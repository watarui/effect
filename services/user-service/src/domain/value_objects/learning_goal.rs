//! 学習目標

use serde::{Deserialize, Serialize};

use super::user_profile::CefrLevel;

/// 学習目標
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningGoal {
    /// 一般的な CEFR レベル目標
    GeneralLevel(CefrLevel),
    /// 特定の目標なし
    NoSpecificGoal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learning_goal_serde() {
        let goal = LearningGoal::GeneralLevel(CefrLevel::B2);
        let json = serde_json::to_string(&goal).unwrap();
        let deserialized: LearningGoal = serde_json::from_str(&json).unwrap();
        assert_eq!(goal, deserialized);

        let goal = LearningGoal::NoSpecificGoal;
        let json = serde_json::to_string(&goal).unwrap();
        let deserialized: LearningGoal = serde_json::from_str(&json).unwrap();
        assert_eq!(goal, deserialized);
    }
}
