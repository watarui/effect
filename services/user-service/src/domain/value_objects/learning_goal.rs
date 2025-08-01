//! 学習目標

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::user_profile::CefrLevel;

/// 学習目標
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LearningGoal {
    /// IELTS スコア目標
    IeltsScore(IeltsScore),
    /// TOEFL スコア目標
    ToeflScore(ToeflScore),
    /// TOEIC スコア目標
    ToeicScore(ToeicScore),
    /// 英検レベル目標
    EikenLevel(EikenLevel),
    /// 一般的な CEFR レベル目標
    GeneralLevel(CefrLevel),
    /// 特定の目標なし
    NoSpecificGoal,
}

/// IELTS スコア目標
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IeltsScore {
    /// 総合スコア (1.0 - 9.0, 0.5 刻み)
    pub overall:   f32,
    /// リーディングスコア
    pub reading:   Option<f32>,
    /// リスニングスコア
    pub listening: Option<f32>,
    /// ライティングスコア
    pub writing:   Option<f32>,
    /// スピーキングスコア
    pub speaking:  Option<f32>,
}

/// TOEFL スコア目標
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToeflScore {
    /// 総合スコア (0 - 120)
    pub total:     u8,
    /// リーディングスコア (0 - 30)
    pub reading:   Option<u8>,
    /// リスニングスコア (0 - 30)
    pub listening: Option<u8>,
    /// スピーキングスコア (0 - 30)
    pub speaking:  Option<u8>,
    /// ライティングスコア (0 - 30)
    pub writing:   Option<u8>,
}

/// TOEIC スコア目標
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToeicScore {
    /// 総合スコア (10 - 990)
    pub total:     u16,
    /// リスニングスコア (5 - 495)
    pub listening: Option<u16>,
    /// リーディングスコア (5 - 495)
    pub reading:   Option<u16>,
}

/// 英検レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EikenLevel {
    /// 5級
    Level5,
    /// 4級
    Level4,
    /// 3級
    Level3,
    /// 準2級
    PreLevel2,
    /// 2級
    Level2,
    /// 準1級
    PreLevel1,
    /// 1級
    Level1,
}

/// 学習目標エラー
#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    /// IELTS スコアが範囲外
    #[error("IELTS overall score must be between 1.0 and 9.0 (0.5 increments), got: {0}")]
    IeltsOverall(f32),

    /// IELTS セクションスコアが範囲外
    #[error("IELTS section score must be between 1.0 and 9.0 (0.5 increments), got: {0}")]
    IeltsSection(f32),

    /// TOEFL 総合スコアが範囲外
    #[error("TOEFL total score must be between 0 and 120, got: {0}")]
    ToeflTotal(u8),

    /// TOEFL セクションスコアが範囲外
    #[error("TOEFL section score must be between 0 and 30, got: {0}")]
    ToeflSection(u8),

    /// TOEIC 総合スコアが範囲外
    #[error("TOEIC total score must be between 10 and 990, got: {0}")]
    ToeicTotal(u16),

    /// TOEIC リスニングスコアが範囲外
    #[error("TOEIC listening score must be between 5 and 495, got: {0}")]
    ToeicListening(u16),

    /// TOEIC リーディングスコアが範囲外
    #[error("TOEIC reading score must be between 5 and 495, got: {0}")]
    ToeicReading(u16),
}

impl IeltsScore {
    /// 新しい IELTS スコア目標を作成
    ///
    /// # Errors
    ///
    /// スコアが有効範囲外の場合はエラーを返す
    pub fn new(
        overall: f32,
        reading: Option<f32>,
        listening: Option<f32>,
        writing: Option<f32>,
        speaking: Option<f32>,
    ) -> Result<Self, Error> {
        // 総合スコアのバリデーション
        if !Self::is_valid_score(overall) {
            return Err(Error::IeltsOverall(overall));
        }

        // 各セクションスコアのバリデーション
        for (score, _name) in [
            (reading, "reading"),
            (listening, "listening"),
            (writing, "writing"),
            (speaking, "speaking"),
        ] {
            if let Some(s) = score
                && !Self::is_valid_score(s)
            {
                return Err(Error::IeltsSection(s));
            }
        }

        Ok(Self {
            overall,
            reading,
            listening,
            writing,
            speaking,
        })
    }

    /// スコアが有効範囲内かチェック (1.0 - 9.0, 0.5 刻み)
    fn is_valid_score(score: f32) -> bool {
        (1.0..=9.0).contains(&score) && (score * 2.0).fract() == 0.0
    }
}

impl ToeflScore {
    /// 新しい TOEFL スコア目標を作成
    ///
    /// # Errors
    ///
    /// スコアが有効範囲外の場合はエラーを返す
    pub fn new(
        total: u8,
        reading: Option<u8>,
        listening: Option<u8>,
        speaking: Option<u8>,
        writing: Option<u8>,
    ) -> Result<Self, Error> {
        // 総合スコアのバリデーション
        if total > 120 {
            return Err(Error::ToeflTotal(total));
        }

        // 各セクションスコアのバリデーション
        for score in [reading, listening, speaking, writing].iter().flatten() {
            if *score > 30 {
                return Err(Error::ToeflSection(*score));
            }
        }

        Ok(Self {
            total,
            reading,
            listening,
            speaking,
            writing,
        })
    }
}

impl ToeicScore {
    /// 新しい TOEIC スコア目標を作成
    ///
    /// # Errors
    ///
    /// スコアが有効範囲外の場合はエラーを返す
    pub fn new(total: u16, listening: Option<u16>, reading: Option<u16>) -> Result<Self, Error> {
        // 総合スコアのバリデーション
        if !(10..=990).contains(&total) {
            return Err(Error::ToeicTotal(total));
        }

        // リスニングスコアのバリデーション
        if let Some(l) = listening
            && !(5..=495).contains(&l)
        {
            return Err(Error::ToeicListening(l));
        }

        // リーディングスコアのバリデーション
        if let Some(r) = reading
            && !(5..=495).contains(&r)
        {
            return Err(Error::ToeicReading(r));
        }

        Ok(Self {
            total,
            listening,
            reading,
        })
    }
}

impl std::fmt::Display for EikenLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Level5 => write!(f, "5級"),
            Self::Level4 => write!(f, "4級"),
            Self::Level3 => write!(f, "3級"),
            Self::PreLevel2 => write!(f, "準2級"),
            Self::Level2 => write!(f, "2級"),
            Self::PreLevel1 => write!(f, "準1級"),
            Self::Level1 => write!(f, "1級"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ielts_score_validation() {
        // 有効なスコア
        assert!(IeltsScore::new(7.5, Some(8.0), Some(7.0), Some(6.5), Some(7.5)).is_ok());
        assert!(IeltsScore::new(9.0, None, None, None, None).is_ok());
        assert!(IeltsScore::new(1.0, None, None, None, None).is_ok());

        // 無効な総合スコア
        assert!(IeltsScore::new(0.5, None, None, None, None).is_err());
        assert!(IeltsScore::new(9.5, None, None, None, None).is_err());
        assert!(IeltsScore::new(7.3, None, None, None, None).is_err()); // 0.5刻みでない

        // 無効なセクションスコア
        assert!(IeltsScore::new(7.0, Some(10.0), None, None, None).is_err());
        assert!(IeltsScore::new(7.0, None, Some(0.0), None, None).is_err());
    }

    #[test]
    fn toefl_score_validation() {
        // 有効なスコア
        assert!(ToeflScore::new(100, Some(25), Some(26), Some(24), Some(25)).is_ok());
        assert!(ToeflScore::new(0, None, None, None, None).is_ok());
        assert!(ToeflScore::new(120, Some(30), Some(30), Some(30), Some(30)).is_ok());

        // 無効な総合スコア
        assert!(ToeflScore::new(121, None, None, None, None).is_err());

        // 無効なセクションスコア
        assert!(ToeflScore::new(100, Some(31), None, None, None).is_err());
    }

    #[test]
    fn toeic_score_validation() {
        // 有効なスコア
        assert!(ToeicScore::new(800, Some(400), Some(400)).is_ok());
        assert!(ToeicScore::new(990, Some(495), Some(495)).is_ok());
        assert!(ToeicScore::new(10, Some(5), Some(5)).is_ok());

        // 無効な総合スコア
        assert!(ToeicScore::new(9, None, None).is_err());
        assert!(ToeicScore::new(991, None, None).is_err());

        // 無効なセクションスコア
        assert!(ToeicScore::new(800, Some(496), None).is_err());
        assert!(ToeicScore::new(800, None, Some(4)).is_err());
    }

    #[test]
    fn eiken_level_display() {
        assert_eq!(EikenLevel::Level5.to_string(), "5級");
        assert_eq!(EikenLevel::PreLevel2.to_string(), "準2級");
        assert_eq!(EikenLevel::Level1.to_string(), "1級");
    }

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
