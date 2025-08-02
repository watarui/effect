//! レジスター（言語使用域）の値オブジェクト
//!
//! 語彙の使用される文脈やフォーマリティレベルを表現

use std::fmt;

use serde::{Deserialize, Serialize};

/// レジスター（言語使用域）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Register {
    /// 極めて正式（学術論文、法的文書）
    VeryFormal,
    /// 正式（ビジネス文書、公式発表）
    Formal,
    /// 中立（標準的な書き言葉、ニュース）
    Neutral,
    /// 非正式（友人との会話、カジュアルなメール）
    Informal,
    /// 俗語・スラング
    Slang,
    /// 専門用語
    Technical,
    /// 古語・廃語
    Archaic,
    /// 地域方言
    Regional,
}

impl Register {
    /// フォーマリティレベルを数値で取得（1=最もカジュアル、10=最もフォーマル）
    #[must_use]
    pub const fn formality_level(self) -> u8 {
        match self {
            Self::Slang => 1,
            Self::Informal => 3,
            Self::Regional => 4,
            Self::Neutral | Self::Archaic => 5, // 中立として扱う
            Self::Technical => 6,
            Self::Formal => 8,
            Self::VeryFormal => 10,
        }
    }

    /// 使用が推奨される文脈の説明を取得
    #[must_use]
    pub const fn context_description(self) -> &'static str {
        match self {
            Self::VeryFormal => "Academic papers, legal documents, official ceremonies",
            Self::Formal => "Business documents, official announcements, formal presentations",
            Self::Neutral => "Standard written language, news, educational materials",
            Self::Informal => "Casual conversation, personal emails, social media",
            Self::Slang => "Very casual speech, specific social groups",
            Self::Technical => "Professional or academic contexts within specific fields",
            Self::Archaic => "Historical texts, literature (no longer in common use)",
            Self::Regional => "Specific geographic regions or dialects",
        }
    }

    /// ビジネス文脈での適切性を判定
    #[must_use]
    pub const fn is_business_appropriate(self) -> bool {
        matches!(
            self,
            Self::VeryFormal | Self::Formal | Self::Neutral | Self::Technical
        )
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::VeryFormal => "very formal",
            Self::Formal => "formal",
            Self::Neutral => "neutral",
            Self::Informal => "informal",
            Self::Slang => "slang",
            Self::Technical => "technical",
            Self::Archaic => "archaic",
            Self::Regional => "regional",
        };
        write!(f, "{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_should_have_correct_formality_levels() {
        assert_eq!(Register::Slang.formality_level(), 1);
        assert_eq!(Register::Informal.formality_level(), 3);
        assert_eq!(Register::Neutral.formality_level(), 5);
        assert_eq!(Register::Formal.formality_level(), 8);
        assert_eq!(Register::VeryFormal.formality_level(), 10);
    }

    #[test]
    fn register_should_identify_business_appropriate() {
        assert!(Register::VeryFormal.is_business_appropriate());
        assert!(Register::Formal.is_business_appropriate());
        assert!(Register::Neutral.is_business_appropriate());
        assert!(Register::Technical.is_business_appropriate());

        assert!(!Register::Informal.is_business_appropriate());
        assert!(!Register::Slang.is_business_appropriate());
        assert!(!Register::Regional.is_business_appropriate());
        assert!(!Register::Archaic.is_business_appropriate());
    }

    #[test]
    fn register_should_provide_context_descriptions() {
        let formal = Register::Formal;
        let description = formal.context_description();
        assert!(description.contains("Business"));
    }

    #[test]
    fn register_should_display_correctly() {
        assert_eq!(Register::VeryFormal.to_string(), "very formal");
        assert_eq!(Register::Slang.to_string(), "slang");
        assert_eq!(Register::Technical.to_string(), "technical");
    }

    #[test]
    fn register_should_be_serializable() {
        let register = Register::Formal;
        let json = serde_json::to_string(&register).unwrap();
        let deserialized: Register = serde_json::from_str(&json).unwrap();
        assert_eq!(register, deserialized);
    }
}
