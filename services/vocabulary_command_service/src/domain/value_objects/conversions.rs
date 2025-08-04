//! 値オブジェクトの変換実装

use super::*;

// shared_vocabulary_context の型から domain の型への変換
impl From<shared_vocabulary_context::domain::PartOfSpeech> for PartOfSpeech {
    fn from(value: shared_vocabulary_context::domain::PartOfSpeech) -> Self {
        use shared_vocabulary_context::domain::PartOfSpeech as SharedPos;
        match value {
            SharedPos::Noun(_) => Self::Noun, // 名詞の詳細は省略
            SharedPos::Verb(_) => Self::Verb, // 動詞の詳細は省略
            SharedPos::Adjective => Self::Adjective,
            SharedPos::Adverb => Self::Adverb,
            SharedPos::Pronoun => Self::Pronoun,
            SharedPos::Preposition => Self::Preposition,
            SharedPos::Conjunction => Self::Conjunction,
            SharedPos::Interjection => Self::Interjection,
            SharedPos::Determiner => Self::Determiner,
            SharedPos::Other(s) => Self::Other(s),
        }
    }
}

impl From<shared_vocabulary_context::domain::Register> for Register {
    fn from(value: shared_vocabulary_context::domain::Register) -> Self {
        use shared_vocabulary_context::domain::Register as SharedReg;
        match value {
            SharedReg::Formal => Self::Formal,
            SharedReg::Neutral => Self::Neutral,
            SharedReg::Informal => Self::Informal,
            SharedReg::Slang => Self::Slang,
            SharedReg::Technical => Self::Technical,
        }
    }
}

impl From<shared_vocabulary_context::domain::Domain> for Domain {
    fn from(value: shared_vocabulary_context::domain::Domain) -> Self {
        use shared_vocabulary_context::domain::Domain as SharedDom;
        match value {
            SharedDom::General => Self::General,
            SharedDom::Academic => Self::Academic,
            SharedDom::Business => Self::Business,
            SharedDom::Medical => Self::Medical,
            SharedDom::Legal => Self::Legal,
            SharedDom::Technical => Self::Technical,
            SharedDom::Scientific => Self::Scientific,
            SharedDom::Literary => Self::Literary,
            SharedDom::Other(s) => Self::Other(s),
        }
    }
}

// UserId から Uuid への変換は application 層で直接 as_uuid() を使う
