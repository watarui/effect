//! Vocabulary Context 固有の値オブジェクト

use serde::{Deserialize, Serialize};

/// 定義
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    pub text:     String,
    pub examples: Vec<String>,
}

/// 品詞
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartOfSpeech {
    Noun(NounType),
    Verb(VerbType),
    Adjective,
    Adverb,
    Pronoun,
    Preposition,
    Conjunction,
    Interjection,
    Determiner,
    Other(String),
}

/// 名詞の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NounType {
    Countable,
    Uncountable,
    Both,
}

/// 動詞の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerbType {
    Transitive,
    Intransitive,
    Both,
}

/// レジスター（使用域）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Register {
    Formal,
    Neutral,
    Informal,
    Slang,
    Technical,
}

/// ドメイン（専門分野）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
