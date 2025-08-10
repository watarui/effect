//! AI Context のイベントスキーマ定義

use std::collections::HashMap;

/// AI Context のスキーマを取得
#[must_use]
#[allow(dead_code, clippy::too_many_lines)]
pub fn get_schemas() -> HashMap<String, String> {
    let mut schemas = HashMap::new();

    // TaskCreated イベント
    schemas.insert(
        "ai.TaskCreated".to_string(),
        r##"{
            "type": "object",
            "required": ["task_id", "task_type", "payload", "metadata"],
            "properties": {
                "task_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "task_type": {
                    "type": "string",
                    "enum": ["generate_example", "generate_hint", "generate_explanation", "translate", "evaluate"]
                },
                "payload": {
                    "type": "object"
                },
                "priority": {
                    "type": "string",
                    "enum": ["low", "normal", "high"],
                    "default": "normal"
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##.to_string(),
    );

    // TaskCompleted イベント
    schemas.insert(
        "ai.TaskCompleted".to_string(),
        r##"{
            "type": "object",
            "required": ["task_id", "result", "metadata"],
            "properties": {
                "task_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "result": {
                    "type": "object"
                },
                "processing_time_ms": {
                    "type": "integer",
                    "minimum": 0
                },
                "tokens_used": {
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "integer",
                            "minimum": 0
                        },
                        "completion": {
                            "type": "integer",
                            "minimum": 0
                        }
                    }
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    // TaskFailed イベント
    schemas.insert(
        "ai.TaskFailed".to_string(),
        r##"{
            "type": "object",
            "required": ["task_id", "error", "metadata"],
            "properties": {
                "task_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "error": {
                    "type": "object",
                    "required": ["code", "message"],
                    "properties": {
                        "code": {
                            "type": "string"
                        },
                        "message": {
                            "type": "string"
                        },
                        "retry_count": {
                            "type": "integer",
                            "minimum": 0
                        },
                        "is_retryable": {
                            "type": "boolean"
                        }
                    }
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    schemas
}
