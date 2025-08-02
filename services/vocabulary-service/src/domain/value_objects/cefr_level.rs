//! CEFR レベルの値オブジェクト
//!
//! ヨーロッパ言語共通参照枠（CEFR）に基づく言語能力レベル

use domain_events::CefrLevel;

/// CEFR レベルの拡張メソッド
pub trait Ext {
    /// 数値レベルを取得（1-6）
    fn numeric_level(&self) -> u8;

    /// レベルの説明を取得
    fn description(&self) -> &'static str;

    /// 日本の英語学習レベルとの対応
    fn japanese_equivalent(&self) -> &'static str;

    /// より高いレベルかどうかを判定
    fn is_higher_than(&self, other: CefrLevel) -> bool;

    /// 文字列から CEFR レベルを作成
    fn from_str(s: &str) -> Option<Self>
    where
        Self: Sized;

    /// 文字列に変換
    fn as_str(&self) -> &'static str;
}

impl Ext for CefrLevel {
    fn numeric_level(&self) -> u8 {
        match self {
            Self::A1 => 1,
            Self::A2 => 2,
            Self::B1 => 3,
            Self::B2 => 4,
            Self::C1 => 5,
            Self::C2 => 6,
            Self::Unspecified => 0,
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::A1 => "Breakthrough - Can understand and use familiar everyday expressions",
            Self::A2 => "Waystage - Can understand sentences and frequently used expressions",
            Self::B1 => "Threshold - Can understand the main points of clear standard input",
            Self::B2 => "Vantage - Can understand complex texts on concrete and abstract topics",
            Self::C1 => {
                "Effective Operational Proficiency - Can understand demanding, longer texts"
            },
            Self::C2 => "Mastery - Can understand virtually everything heard or read",
            Self::Unspecified => "Unknown level",
        }
    }

    fn japanese_equivalent(&self) -> &'static str {
        match self {
            Self::A1 => "中学校レベル",
            Self::A2 => "高校基礎レベル",
            Self::B1 => "高校卒業レベル",
            Self::B2 => "大学受験レベル",
            Self::C1 => "大学上級レベル",
            Self::C2 => "ネイティブレベル",
            Self::Unspecified => "不明",
        }
    }

    fn is_higher_than(&self, other: CefrLevel) -> bool {
        self.numeric_level() > other.numeric_level()
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "A1" => Some(Self::A1),
            "A2" => Some(Self::A2),
            "B1" => Some(Self::B1),
            "B2" => Some(Self::B2),
            "C1" => Some(Self::C1),
            "C2" => Some(Self::C2),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::A1 => "A1",
            Self::A2 => "A2",
            Self::B1 => "B1",
            Self::B2 => "B2",
            Self::C1 => "C1",
            Self::C2 => "C2",
            Self::Unspecified => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cefr_level_should_have_correct_numeric_levels() {
        assert_eq!(CefrLevel::A1.numeric_level(), 1);
        assert_eq!(CefrLevel::A2.numeric_level(), 2);
        assert_eq!(CefrLevel::B1.numeric_level(), 3);
        assert_eq!(CefrLevel::B2.numeric_level(), 4);
        assert_eq!(CefrLevel::C1.numeric_level(), 5);
        assert_eq!(CefrLevel::C2.numeric_level(), 6);
    }

    #[test]
    fn cefr_level_should_parse_from_string() {
        assert_eq!(CefrLevel::from_str("A1"), Some(CefrLevel::A1));
        assert_eq!(CefrLevel::from_str("a1"), Some(CefrLevel::A1));
        assert_eq!(CefrLevel::from_str("C2"), Some(CefrLevel::C2));
        assert!(CefrLevel::from_str("Z9").is_none());
    }

    #[test]
    fn cefr_level_should_compare_correctly() {
        assert!(CefrLevel::C2.is_higher_than(CefrLevel::A1));
        assert!(CefrLevel::B2.is_higher_than(CefrLevel::B1));
        assert!(!CefrLevel::A1.is_higher_than(CefrLevel::A2));
    }

    #[test]
    fn cefr_level_should_provide_descriptions() {
        let description = CefrLevel::B2.description();
        assert!(description.contains("complex texts"));
    }

    #[test]
    fn cefr_level_should_convert_to_string() {
        assert_eq!(CefrLevel::A1.as_str(), "A1");
        assert_eq!(CefrLevel::C2.as_str(), "C2");
    }
}
