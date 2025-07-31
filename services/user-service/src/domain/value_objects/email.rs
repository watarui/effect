//! Email 値オブジェクト

use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::ValidateEmail;

/// Email アドレスを表す値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

/// Email バリデーションエラー
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// 不正な Email フォーマット
    #[error("Invalid email format: {0}")]
    InvalidFormat(String),
}

impl Email {
    /// 新しい Email インスタンスを作成
    ///
    /// # Arguments
    ///
    /// * `email` - メールアドレス文字列
    ///
    /// # Errors
    ///
    /// * `Error::InvalidFormat` - メールアドレスの形式が不正な場合
    pub fn new(email: &str) -> Result<Self, Error> {
        let email = email.trim().to_lowercase();

        if !email.validate_email() {
            return Err(Error::InvalidFormat(email));
        }

        Ok(Self(email))
    }

    /// Email アドレスを文字列として取得
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_new_should_accept_valid_email() {
        // Given
        let valid_email = "user@example.com";

        // When
        let result = Email::new(valid_email);

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "user@example.com");
    }

    #[test]
    fn email_new_should_normalize_email() {
        // Given
        let email_with_spaces = "  USER@EXAMPLE.COM  ";

        // When
        let result = Email::new(email_with_spaces);

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "user@example.com");
    }

    #[test]
    fn email_new_should_reject_invalid_email() {
        // Given
        let invalid_email = "not-an-email";

        // When
        let result = Email::new(invalid_email);

        // Then
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::InvalidFormat("not-an-email".to_string())
        );
    }
}
