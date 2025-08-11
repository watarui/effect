//! イベント検証の実装

use prost::Message;
use serde_json::Value as JsonValue;

use crate::registry::{Registry, SchemaRegistryError};

/// 検証エラー
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// エラーが発生したフィールド
    pub field:   String,
    /// エラーメッセージ
    pub message: String,
    /// エラーコード
    pub code:    String,
}

/// イベントバリデーター
#[derive(Clone)]
pub struct Validator {
    registry: Registry,
}

impl Validator {
    /// 新しいバリデーターを作成
    #[must_use]
    pub const fn new(registry: Registry) -> Self {
        Self { registry }
    }

    /// イベントを検証
    ///
    /// # Errors
    ///
    /// - `Error::Registry` - スキーマレジストリエラーが発生した場合
    pub async fn validate_event(
        &self,
        event_type: &str,
        event_data: &[u8],
        schema_version: Option<i32>,
    ) -> Result<Vec<ValidationError>, Error> {
        // スキーマを取得
        let _schema = self.registry.get_schema(event_type, schema_version).await?;

        let mut errors = Vec::new();

        // 基本的な検証（protobuf デコード可能か）
        if let Err(e) = Self::validate_protobuf_format(event_data) {
            errors.push(ValidationError {
                field:   String::new(),
                message: format!("Invalid protobuf format: {e}"),
                code:    "INVALID_FORMAT".to_string(),
            });
            return Ok(errors);
        }

        // イベントタイプ別の検証
        match event_type {
            t if t.starts_with("vocabulary.") => {
                Self::validate_vocabulary_event(t, event_data, &mut errors);
            },
            t if t.starts_with("learning.") => {
                Self::validate_learning_event(t, event_data, &mut errors);
            },
            t if t.starts_with("user.") => {
                Self::validate_user_event(t, event_data, &mut errors);
            },
            t if t.starts_with("algorithm.") => {
                Self::validate_algorithm_event(t, event_data, &mut errors);
            },
            t if t.starts_with("ai.") => {
                Self::validate_ai_event(t, event_data, &mut errors);
            },
            _ => {
                errors.push(ValidationError {
                    field:   "event_type".to_string(),
                    message: format!("Unknown event type: {event_type}"),
                    code:    "UNKNOWN_EVENT_TYPE".to_string(),
                });
            },
        }

        Ok(errors)
    }

    /// protobuf 形式の基本検証
    fn validate_protobuf_format(data: &[u8]) -> Result<(), prost::DecodeError> {
        // 最低限のデコード可能性をチェック
        // 実際のメッセージ型は後で検証
        let _ = prost_types::Any::decode(data)?;
        Ok(())
    }

    /// Vocabulary イベントの検証
    fn validate_vocabulary_event(
        event_type: &str,
        event_data: &[u8],
        errors: &mut Vec<ValidationError>,
    ) {
        // イベントタイプごとの詳細検証
        match event_type {
            "vocabulary.EntryCreated" => {
                // 必須フィールドのチェック
                if let Ok(json) = serde_json::from_slice::<JsonValue>(event_data) {
                    if json.get("entry_id").is_none() {
                        errors.push(ValidationError {
                            field:   "entry_id".to_string(),
                            message: "entry_id is required".to_string(),
                            code:    "REQUIRED_FIELD".to_string(),
                        });
                    }
                    if json.get("spelling").is_none() {
                        errors.push(ValidationError {
                            field:   "spelling".to_string(),
                            message: "spelling is required".to_string(),
                            code:    "REQUIRED_FIELD".to_string(),
                        });
                    }
                }
            },
            "vocabulary.ItemCreated" => {
                if let Ok(json) = serde_json::from_slice::<JsonValue>(event_data)
                    && json.get("item_id").is_none()
                {
                    errors.push(ValidationError {
                        field:   "item_id".to_string(),
                        message: "item_id is required".to_string(),
                        code:    "REQUIRED_FIELD".to_string(),
                    });
                }
            },
            _ => {
                // その他の Vocabulary イベント
            },
        }
    }

    /// Learning イベントの検証
    fn validate_learning_event(
        event_type: &str,
        event_data: &[u8],
        errors: &mut Vec<ValidationError>,
    ) {
        if event_type == "learning.SessionStarted" {
            if let Ok(json) = serde_json::from_slice::<JsonValue>(event_data) {
                if json.get("session_id").is_none() {
                    errors.push(ValidationError {
                        field:   "session_id".to_string(),
                        message: "session_id is required".to_string(),
                        code:    "REQUIRED_FIELD".to_string(),
                    });
                }
                if json.get("user_id").is_none() {
                    errors.push(ValidationError {
                        field:   "user_id".to_string(),
                        message: "user_id is required".to_string(),
                        code:    "REQUIRED_FIELD".to_string(),
                    });
                }
            }
        } else {
            // その他の Learning イベント
        }
    }

    /// User イベントの検証
    fn validate_user_event(event_type: &str, event_data: &[u8], errors: &mut Vec<ValidationError>) {
        if event_type == "user.UserSignedUp" {
            if let Ok(json) = serde_json::from_slice::<JsonValue>(event_data) {
                if json.get("user_id").is_none() {
                    errors.push(ValidationError {
                        field:   "user_id".to_string(),
                        message: "user_id is required".to_string(),
                        code:    "REQUIRED_FIELD".to_string(),
                    });
                }
                if json.get("email").is_none() {
                    errors.push(ValidationError {
                        field:   "email".to_string(),
                        message: "email is required".to_string(),
                        code:    "REQUIRED_FIELD".to_string(),
                    });
                }
            }
        } else {
            // その他の User イベント
        }
    }

    /// Algorithm イベントの検証
    const fn validate_algorithm_event(
        _event_type: &str,
        _event_data: &[u8],
        _errors: &mut [ValidationError],
    ) {
        // TODO: Algorithm イベントの詳細検証
    }

    /// AI イベントの検証
    const fn validate_ai_event(
        _event_type: &str,
        _event_data: &[u8],
        _errors: &mut [ValidationError],
    ) {
        // TODO: AI イベントの詳細検証
    }
}

/// バリデーターのエラー
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// スキーマレジストリエラー
    #[error("Schema registry error: {0}")]
    Registry(#[from] SchemaRegistryError),

    /// バリデーション失敗
    #[error("Validation failed: {0}")]
    #[allow(dead_code)]
    ValidationFailed(String),
}
