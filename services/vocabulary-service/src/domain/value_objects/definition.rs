//! 定義に関する値オブジェクト
//!
//! 語彙エントリーの定義、例文、コロケーションを管理

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 定義エラー
#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    /// 定義テキストが空の場合
    #[error("Definition text cannot be empty")]
    EmptyDefinition,
    /// 定義テキストが長すぎる場合
    #[error("Definition text is too long (max: {max}, actual: {actual})")]
    DefinitionTooLong {
        /// 最大文字数
        max:    usize,
        /// 実際の文字数
        actual: usize,
    },
    /// 例文が長すぎる場合
    #[error("Example text is too long (max: {max}, actual: {actual})")]
    ExampleTooLong {
        /// 最大文字数
        max:    usize,
        /// 実際の文字数
        actual: usize,
    },
}

/// 定義
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Definition {
    text: String,
}

impl Definition {
    const MAX_LENGTH: usize = 500;

    /// 新しい定義を作成
    ///
    /// # Errors
    ///
    /// - 定義が空の場合
    /// - 定義が最大長を超える場合
    pub fn new(text: &str) -> Result<Self, Error> {
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return Err(Error::EmptyDefinition);
        }

        if trimmed.len() > Self::MAX_LENGTH {
            return Err(Error::DefinitionTooLong {
                max:    Self::MAX_LENGTH,
                actual: trimmed.len(),
            });
        }

        Ok(Self {
            text: trimmed.to_string(),
        })
    }

    /// 定義テキストを取得
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// 例文
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Example {
    sentence:    String,
    translation: Option<String>,
}

impl Example {
    const MAX_LENGTH: usize = 300;

    /// 新しい例文を作成
    ///
    /// # Errors
    ///
    /// 例文が最大長を超える場合
    pub fn new(sentence: &str, translation: Option<&str>) -> Result<Self, Error> {
        let trimmed = sentence.trim();

        if trimmed.len() > Self::MAX_LENGTH {
            return Err(Error::ExampleTooLong {
                max:    Self::MAX_LENGTH,
                actual: trimmed.len(),
            });
        }

        Ok(Self {
            sentence:    trimmed.to_string(),
            translation: translation.map(str::to_string),
        })
    }

    /// 例文を取得
    #[must_use]
    pub fn sentence(&self) -> &str {
        &self.sentence
    }

    /// 翻訳を取得
    #[must_use]
    pub fn translation(&self) -> Option<&str> {
        self.translation.as_deref()
    }
}

impl fmt::Display for Example {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sentence)
    }
}

/// コロケーション（語の組み合わせ）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Collocation {
    pattern:  String,
    examples: Vec<String>,
}

impl Collocation {
    /// 新しいコロケーションを作成
    #[must_use]
    pub fn new(pattern: &str, examples: Vec<&str>) -> Self {
        Self {
            pattern:  pattern.trim().to_string(),
            examples: examples.into_iter().map(|e| e.trim().to_string()).collect(),
        }
    }

    /// パターンを取得
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// 例を取得
    #[must_use]
    pub fn examples(&self) -> &[String] {
        &self.examples
    }

    /// 例を追加
    pub fn add_example(&mut self, example: &str) {
        self.examples.push(example.trim().to_string());
    }
}

impl fmt::Display for Collocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_should_be_created_with_valid_text() {
        let def = Definition::new("A state of temporary disuse or suspension").unwrap();

        assert_eq!(def.text(), "A state of temporary disuse or suspension");
    }

    #[test]
    fn definition_should_reject_empty_text() {
        let result = Definition::new("   ");
        assert!(matches!(result, Err(Error::EmptyDefinition)));
    }

    #[test]
    fn definition_should_reject_too_long_text() {
        let long_text = "a".repeat(501);
        let result = Definition::new(&long_text);
        assert!(matches!(
            result,
            Err(Error::DefinitionTooLong {
                max:    500,
                actual: 501,
            })
        ));
    }

    #[test]
    fn example_should_be_created_with_valid_sentence() {
        let example = Example::new(
            "The project is in abeyance until funding is secured.",
            Some("プロジェクトは資金が確保されるまで保留中です。"),
        )
        .unwrap();

        assert_eq!(
            example.sentence(),
            "The project is in abeyance until funding is secured."
        );
        assert_eq!(
            example.translation(),
            Some("プロジェクトは資金が確保されるまで保留中です。")
        );
    }

    #[test]
    fn example_should_reject_too_long_sentence() {
        let long_sentence = "a".repeat(301);
        let result = Example::new(&long_sentence, None);
        assert!(matches!(
            result,
            Err(Error::ExampleTooLong {
                max:    300,
                actual: 301,
            })
        ));
    }

    #[test]
    fn collocation_should_manage_patterns_and_examples() {
        let mut collocation = Collocation::new(
            "in abeyance",
            vec!["hold in abeyance", "remain in abeyance"],
        );

        assert_eq!(collocation.pattern(), "in abeyance");
        assert_eq!(collocation.examples().len(), 2);

        collocation.add_example("keep in abeyance");
        assert_eq!(collocation.examples().len(), 3);
    }

    #[test]
    fn value_objects_should_be_serializable() {
        let def = Definition::new("test definition").unwrap();
        let json = serde_json::to_string(&def).unwrap();
        let deserialized: Definition = serde_json::from_str(&json).unwrap();
        assert_eq!(def, deserialized);

        let example = Example::new("test sentence", None).unwrap();
        let json = serde_json::to_string(&example).unwrap();
        let deserialized: Example = serde_json::from_str(&json).unwrap();
        assert_eq!(example, deserialized);

        let collocation = Collocation::new("test pattern", vec!["example"]);
        let json = serde_json::to_string(&collocation).unwrap();
        let deserialized: Collocation = serde_json::from_str(&json).unwrap();
        assert_eq!(collocation, deserialized);
    }

    #[test]
    fn value_objects_should_display_correctly() {
        let def = Definition::new("test definition").unwrap();
        assert_eq!(def.text(), "test definition");

        let example = Example::new("test sentence", None).unwrap();
        assert_eq!(example.sentence(), "test sentence");

        let collocation = Collocation::new("test pattern", vec![]);
        assert_eq!(collocation.pattern(), "test pattern");
    }
}
