//! 値オブジェクト
//!
//! ドメインの値オブジェクトを定義

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
