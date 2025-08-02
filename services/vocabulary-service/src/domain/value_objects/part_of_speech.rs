//! 品詞に関する値オブジェクト
//!
//! 語彙エントリーの品詞情報を管理する値オブジェクト

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 名詞の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NounType {
    /// 可算名詞
    Countable,
    /// 不可算名詞
    Uncountable,
    /// 可算・不可算両方
    Both,
}

impl fmt::Display for NounType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Countable => write!(f, "countable"),
            Self::Uncountable => write!(f, "uncountable"),
            Self::Both => write!(f, "both"),
        }
    }
}

/// 動詞の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerbType {
    /// 他動詞
    Transitive,
    /// 自動詞
    Intransitive,
    /// 他動詞・自動詞両方
    Both,
}

impl fmt::Display for VerbType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transitive => write!(f, "transitive"),
            Self::Intransitive => write!(f, "intransitive"),
            Self::Both => write!(f, "both"),
        }
    }
}

/// 形容詞の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdjectiveType {
    /// 叙述的形容詞
    Predicative,
    /// 限定的形容詞
    Attributive,
    /// 叙述・限定両方
    Both,
}

impl fmt::Display for AdjectiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Predicative => write!(f, "predicative"),
            Self::Attributive => write!(f, "attributive"),
            Self::Both => write!(f, "both"),
        }
    }
}

/// 品詞
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartOfSpeech {
    /// 名詞
    Noun(NounType),
    /// 動詞
    Verb(VerbType),
    /// 形容詞
    Adjective(AdjectiveType),
    /// 副詞
    Adverb,
    /// 前置詞
    Preposition,
    /// 接続詞
    Conjunction,
    /// 冠詞
    Article,
    /// 代名詞
    Pronoun,
    /// 感嘆詞
    Interjection,
    /// 助動詞
    Modal,
    /// 句動詞
    PhrasalVerb,
    /// イディオム
    Idiom,
}

impl PartOfSpeech {
    /// 基本的な品詞名を取得
    #[must_use]
    pub const fn base_name(&self) -> &'static str {
        match self {
            Self::Noun(_) => "noun",
            Self::Verb(_) => "verb",
            Self::Adjective(_) => "adjective",
            Self::Adverb => "adverb",
            Self::Preposition => "preposition",
            Self::Conjunction => "conjunction",
            Self::Article => "article",
            Self::Pronoun => "pronoun",
            Self::Interjection => "interjection",
            Self::Modal => "modal",
            Self::PhrasalVerb => "phrasal_verb",
            Self::Idiom => "idiom",
        }
    }

    /// 詳細な品詞情報を含む文字列を取得
    #[must_use]
    pub fn full_name(&self) -> String {
        match self {
            Self::Noun(noun_type) => format!("noun ({noun_type})"),
            Self::Verb(verb_type) => format!("verb ({verb_type})"),
            Self::Adjective(adj_type) => format!("adjective ({adj_type})"),
            _ => self.base_name().to_string(),
        }
    }
}

impl fmt::Display for PartOfSpeech {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

/// 品詞のパースエラー
#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    /// 無効な品詞
    #[error("Invalid part of speech: {0}")]
    InvalidPartOfSpeech(String),
}

impl FromStr for PartOfSpeech {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "noun" | "noun (countable)" => Ok(Self::Noun(NounType::Countable)),
            "noun (uncountable)" => Ok(Self::Noun(NounType::Uncountable)),
            "noun (both)" => Ok(Self::Noun(NounType::Both)),
            "verb" | "verb (transitive)" => Ok(Self::Verb(VerbType::Transitive)),
            "verb (intransitive)" => Ok(Self::Verb(VerbType::Intransitive)),
            "verb (both)" => Ok(Self::Verb(VerbType::Both)),
            "adjective" | "adjective (predicative)" => {
                Ok(Self::Adjective(AdjectiveType::Predicative))
            },
            "adjective (attributive)" => Ok(Self::Adjective(AdjectiveType::Attributive)),
            "adjective (both)" => Ok(Self::Adjective(AdjectiveType::Both)),
            "adverb" => Ok(Self::Adverb),
            "preposition" => Ok(Self::Preposition),
            "conjunction" => Ok(Self::Conjunction),
            "article" => Ok(Self::Article),
            "pronoun" => Ok(Self::Pronoun),
            "interjection" => Ok(Self::Interjection),
            "modal" => Ok(Self::Modal),
            "phrasal_verb" => Ok(Self::PhrasalVerb),
            "idiom" => Ok(Self::Idiom),
            _ => Err(Error::InvalidPartOfSpeech(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noun_type_should_display_correctly() {
        assert_eq!(NounType::Countable.to_string(), "countable");
        assert_eq!(NounType::Uncountable.to_string(), "uncountable");
        assert_eq!(NounType::Both.to_string(), "both");
    }

    #[test]
    fn verb_type_should_display_correctly() {
        assert_eq!(VerbType::Transitive.to_string(), "transitive");
        assert_eq!(VerbType::Intransitive.to_string(), "intransitive");
        assert_eq!(VerbType::Both.to_string(), "both");
    }

    #[test]
    fn part_of_speech_should_return_base_name() {
        let noun = PartOfSpeech::Noun(NounType::Countable);
        assert_eq!(noun.base_name(), "noun");

        let verb = PartOfSpeech::Verb(VerbType::Transitive);
        assert_eq!(verb.base_name(), "verb");

        let adverb = PartOfSpeech::Adverb;
        assert_eq!(adverb.base_name(), "adverb");
    }

    #[test]
    fn part_of_speech_should_return_full_name() {
        let noun = PartOfSpeech::Noun(NounType::Countable);
        assert_eq!(noun.full_name(), "noun (countable)");

        let verb = PartOfSpeech::Verb(VerbType::Transitive);
        assert_eq!(verb.full_name(), "verb (transitive)");

        let adverb = PartOfSpeech::Adverb;
        assert_eq!(adverb.full_name(), "adverb");
    }

    #[test]
    fn part_of_speech_should_be_serializable() {
        let pos = PartOfSpeech::Noun(NounType::Countable);
        let json = serde_json::to_string(&pos).unwrap();
        let deserialized: PartOfSpeech = serde_json::from_str(&json).unwrap();
        assert_eq!(pos, deserialized);
    }
}
