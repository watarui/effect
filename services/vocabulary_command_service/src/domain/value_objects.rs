//! 値オブジェクト
//!
//! ドメインの値オブジェクトを定義

use std::fmt;

use serde::{Deserialize, Serialize};

mod conversions;

/// 品詞
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Pronoun,
    Preposition,
    Conjunction,
    Interjection,
    Determiner,
    Other(String),
}

/// レジスター（使用域）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Register {
    Formal,
    Informal,
    Neutral,
    Slang,
    Technical,
    Literary,
}

/// ドメイン（分野）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Domain {
    General,
    Academic,
    Business,
    Medical,
    Legal,
    Technical,
    Scientific,
    Literary,
    Other(String),
}

// Display trait implementations
impl fmt::Display for PartOfSpeech {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noun => write!(f, "Noun"),
            Self::Verb => write!(f, "Verb"),
            Self::Adjective => write!(f, "Adjective"),
            Self::Adverb => write!(f, "Adverb"),
            Self::Pronoun => write!(f, "Pronoun"),
            Self::Preposition => write!(f, "Preposition"),
            Self::Conjunction => write!(f, "Conjunction"),
            Self::Interjection => write!(f, "Interjection"),
            Self::Determiner => write!(f, "Determiner"),
            Self::Other(s) => write!(f, "Other({s})"),
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Formal => write!(f, "Formal"),
            Self::Informal => write!(f, "Informal"),
            Self::Neutral => write!(f, "Neutral"),
            Self::Slang => write!(f, "Slang"),
            Self::Technical => write!(f, "Technical"),
            Self::Literary => write!(f, "Literary"),
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::General => write!(f, "General"),
            Self::Academic => write!(f, "Academic"),
            Self::Business => write!(f, "Business"),
            Self::Medical => write!(f, "Medical"),
            Self::Legal => write!(f, "Legal"),
            Self::Technical => write!(f, "Technical"),
            Self::Scientific => write!(f, "Scientific"),
            Self::Literary => write!(f, "Literary"),
            Self::Other(s) => write!(f, "Other({s})"),
        }
    }
}
