//! 専門分野ドメインの値オブジェクト
//!
//! 語彙が使用される専門分野やコンテキストを表現

use std::fmt;

use serde::{Deserialize, Serialize};

/// 専門分野ドメイン
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    /// 一般的な語彙
    General,
    /// ビジネス・経済
    Business,
    /// 科学・技術
    Science,
    /// 医学・医療
    Medicine,
    /// 法律
    Law,
    /// 教育
    Education,
    /// IT・コンピューター
    Technology,
    /// 芸術・文化
    Arts,
    /// スポーツ
    Sports,
    /// 料理・食品
    Culinary,
    /// 旅行・観光
    Travel,
    /// 環境・エコロジー
    Environment,
    /// 金融・投資
    Finance,
    /// 心理学
    Psychology,
    /// 哲学
    Philosophy,
    /// 文学
    Literature,
    /// 音楽
    Music,
    /// 映画・演劇
    Entertainment,
    /// 建築・工学
    Engineering,
    /// 農業
    Agriculture,
    /// その他
    Other(String),
}

impl Domain {
    /// ドメインの日本語名を取得
    #[must_use]
    pub fn japanese_name(&self) -> &str {
        match self {
            Self::General => "一般",
            Self::Business => "ビジネス・経済",
            Self::Science => "科学・技術",
            Self::Medicine => "医学・医療",
            Self::Law => "法律",
            Self::Education => "教育",
            Self::Technology => "IT・コンピューター",
            Self::Arts => "芸術・文化",
            Self::Sports => "スポーツ",
            Self::Culinary => "料理・食品",
            Self::Travel => "旅行・観光",
            Self::Environment => "環境・エコロジー",
            Self::Finance => "金融・投資",
            Self::Psychology => "心理学",
            Self::Philosophy => "哲学",
            Self::Literature => "文学",
            Self::Music => "音楽",
            Self::Entertainment => "映画・演劇",
            Self::Engineering => "建築・工学",
            Self::Agriculture => "農業",
            Self::Other(name) => name,
        }
    }

    /// ドメインの説明を取得
    #[must_use]
    pub fn description(&self) -> &str {
        match self {
            Self::General => "General vocabulary used in everyday contexts",
            Self::Business => "Terms used in business, economics, and commerce",
            Self::Science => "Scientific and technical terminology",
            Self::Medicine => "Medical and healthcare terminology",
            Self::Law => "Legal terminology and concepts",
            Self::Education => "Educational and academic terminology",
            Self::Technology => "IT, computer science, and technology terms",
            Self::Arts => "Arts, culture, and creative fields",
            Self::Sports => "Sports and athletic terminology",
            Self::Culinary => "Cooking, food, and restaurant terminology",
            Self::Travel => "Travel, tourism, and hospitality terms",
            Self::Environment => "Environmental and ecological terminology",
            Self::Finance => "Financial and investment terminology",
            Self::Psychology => "Psychological and behavioral terms",
            Self::Philosophy => "Philosophical concepts and terminology",
            Self::Literature => "Literary terms and concepts",
            Self::Music => "Musical terminology and concepts",
            Self::Entertainment => "Entertainment, film, and theater terms",
            Self::Engineering => "Engineering and construction terminology",
            Self::Agriculture => "Agricultural and farming terminology",
            Self::Other(name) => name,
        }
    }

    /// 専門的なドメインかどうかを判定
    #[must_use]
    pub const fn is_specialized(&self) -> bool {
        !matches!(self, Self::General)
    }

    /// ビジネス関連のドメインかどうかを判定
    #[must_use]
    pub const fn is_business_related(&self) -> bool {
        matches!(self, Self::Business | Self::Finance | Self::Law)
    }

    /// STEM 分野のドメインかどうかを判定
    #[must_use]
    pub const fn is_stem(&self) -> bool {
        matches!(
            self,
            Self::Science | Self::Technology | Self::Engineering | Self::Medicine
        )
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::General => "general",
            Self::Business => "business",
            Self::Science => "science",
            Self::Medicine => "medicine",
            Self::Law => "law",
            Self::Education => "education",
            Self::Technology => "technology",
            Self::Arts => "arts",
            Self::Sports => "sports",
            Self::Culinary => "culinary",
            Self::Travel => "travel",
            Self::Environment => "environment",
            Self::Finance => "finance",
            Self::Psychology => "psychology",
            Self::Philosophy => "philosophy",
            Self::Literature => "literature",
            Self::Music => "music",
            Self::Entertainment => "entertainment",
            Self::Engineering => "engineering",
            Self::Agriculture => "agriculture",
            Self::Other(name) => name,
        };
        write!(f, "{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_should_identify_specialized() {
        assert!(!Domain::General.is_specialized());
        assert!(Domain::Medicine.is_specialized());
        assert!(Domain::Technology.is_specialized());
    }

    #[test]
    fn domain_should_identify_business_related() {
        assert!(Domain::Business.is_business_related());
        assert!(Domain::Finance.is_business_related());
        assert!(Domain::Law.is_business_related());
        assert!(!Domain::Medicine.is_business_related());
        assert!(!Domain::Arts.is_business_related());
    }

    #[test]
    fn domain_should_identify_stem() {
        assert!(Domain::Science.is_stem());
        assert!(Domain::Technology.is_stem());
        assert!(Domain::Engineering.is_stem());
        assert!(Domain::Medicine.is_stem());
        assert!(!Domain::Arts.is_stem());
        assert!(!Domain::Business.is_stem());
    }

    #[test]
    fn domain_should_provide_japanese_names() {
        assert_eq!(Domain::Business.japanese_name(), "ビジネス・経済");
        assert_eq!(Domain::Technology.japanese_name(), "IT・コンピューター");
        assert_eq!(Domain::General.japanese_name(), "一般");
    }

    #[test]
    fn domain_should_handle_custom_domains() {
        let custom = Domain::Other("Custom Field".to_string());
        assert_eq!(custom.japanese_name(), "Custom Field");
        assert_eq!(custom.description(), "Custom Field");
        assert!(custom.is_specialized());
    }

    #[test]
    fn domain_should_display_correctly() {
        assert_eq!(Domain::Business.to_string(), "business");
        assert_eq!(Domain::General.to_string(), "general");
        let custom = Domain::Other("custom".to_string());
        assert_eq!(custom.to_string(), "custom");
    }

    #[test]
    fn domain_should_be_serializable() {
        let domain = Domain::Science;
        let json = serde_json::to_string(&domain).unwrap();
        let deserialized: Domain = serde_json::from_str(&json).unwrap();
        assert_eq!(domain, deserialized);

        let custom = Domain::Other("test".to_string());
        let json = serde_json::to_string(&custom).unwrap();
        let deserialized: Domain = serde_json::from_str(&json).unwrap();
        assert_eq!(custom, deserialized);
    }
}
