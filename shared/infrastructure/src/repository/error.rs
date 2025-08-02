//! リポジトリ層のエラー定義
//!
//! データベース操作で発生する可能性のあるエラーを統一的に扱う

use thiserror::Error;

/// リポジトリエラー
#[derive(Error, Debug)]
pub enum Error {
    /// エンティティが見つからない
    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound {
        /// エンティティの型名
        entity_type: &'static str,
        /// エンティティの ID
        id:          String,
    },

    /// 一意制約違反
    #[error("Unique constraint violation: {constraint}")]
    UniqueViolation {
        /// 違反した制約名
        constraint: String,
    },

    /// 楽観的ロック失敗
    #[error("Optimistic lock failure: expected version {expected}, but found {actual}")]
    OptimisticLockFailure {
        /// 期待されたバージョン
        expected: u64,
        /// 実際のバージョン
        actual:   u64,
    },

    /// データベース接続エラー
    #[error("Database connection error: {0}")]
    Connection(String),

    /// クエリ実行エラー
    #[error("Query execution error: {0}")]
    QueryExecution(String),

    /// トランザクションエラー
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// データマッピングエラー
    #[error("Data mapping error: {0}")]
    DataMapping(String),

    /// その他のデータベースエラー
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl Error {
    /// エンティティが見つからないエラーを作成
    #[must_use]
    pub fn not_found<T: ToString + ?Sized>(entity_type: &'static str, id: &T) -> Self {
        Self::NotFound {
            entity_type,
            id: id.to_string(),
        }
    }

    /// 一意制約違反エラーを作成
    #[must_use]
    pub fn unique_violation(constraint: impl Into<String>) -> Self {
        Self::UniqueViolation {
            constraint: constraint.into(),
        }
    }

    /// 楽観的ロック失敗エラーを作成
    #[must_use]
    pub const fn optimistic_lock_failure(expected: u64, actual: u64) -> Self {
        Self::OptimisticLockFailure { expected, actual }
    }

    /// `SQLx` エラーから適切なリポジトリエラーに変換
    #[must_use]
    pub fn from_sqlx(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => {
                // この場合は呼び出し側で適切な NotFound エラーに変換すべき
                Self::Database(err)
            },
            sqlx::Error::Database(db_err) => {
                // PostgreSQL のエラーコードをチェック
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        // 一意制約違反
                        "23505" => {
                            if let Some(constraint) = db_err.constraint() {
                                return Self::unique_violation(constraint);
                            }
                        },
                        // 外部キー制約違反
                        "23503" => {
                            return Self::Database(err);
                        },
                        _ => {},
                    }
                }
                Self::Database(err)
            },
            _ => Self::Database(err),
        }
    }
}

/// リポジトリ操作の結果型
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let err = Error::not_found("User", "user-123");
        assert_eq!(err.to_string(), "Entity not found: User with id user-123");
    }

    #[test]
    fn test_unique_violation_error() {
        let err = Error::unique_violation("users_email_key");
        assert_eq!(
            err.to_string(),
            "Unique constraint violation: users_email_key"
        );
    }

    #[test]
    fn test_optimistic_lock_failure() {
        let err = Error::optimistic_lock_failure(2, 3);
        assert_eq!(
            err.to_string(),
            "Optimistic lock failure: expected version 2, but found 3"
        );
    }
}
