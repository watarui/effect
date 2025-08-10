//! Learning Context のイベントスキーマ定義

use std::collections::HashMap;

/// Learning Context のスキーマを取得
#[must_use]
#[allow(dead_code, clippy::too_many_lines)]
pub fn get_schemas() -> HashMap<String, String> {
    let mut schemas = HashMap::new();

    // SessionStarted イベント
    schemas.insert(
        "learning.SessionStarted".to_string(),
        r##"{
            "type": "object",
            "required": ["session_id", "user_id", "item_count", "metadata"],
            "properties": {
                "session_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "user_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "item_count": {
                    "type": "integer",
                    "minimum": 1
                },
                "strategy": {
                    "type": "string",
                    "enum": ["new", "review", "mixed"]
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    // ItemPresented イベント
    schemas.insert(
        "learning.ItemPresented".to_string(),
        r##"{
            "type": "object",
            "required": ["session_id", "item_id", "presentation_order", "metadata"],
            "properties": {
                "session_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "item_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "presentation_order": {
                    "type": "integer",
                    "minimum": 1
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    // CorrectnessJudged イベント
    schemas.insert(
        "learning.CorrectnessJudged".to_string(),
        r##"{
            "type": "object",
            "required": ["session_id", "item_id", "judgment", "metadata"],
            "properties": {
                "session_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "item_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "judgment": {
                    "type": "string",
                    "enum": ["correct", "incorrect", "partially_correct"]
                },
                "time_spent_ms": {
                    "type": "integer",
                    "minimum": 0
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    // SessionCompleted イベント
    schemas.insert(
        "learning.SessionCompleted".to_string(),
        r##"{
            "type": "object",
            "required": ["session_id", "items_completed", "items_correct", "total_time_ms", "metadata"],
            "properties": {
                "session_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "items_completed": {
                    "type": "integer",
                    "minimum": 0
                },
                "items_correct": {
                    "type": "integer",
                    "minimum": 0
                },
                "total_time_ms": {
                    "type": "integer",
                    "minimum": 0
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##.to_string(),
    );

    schemas
}
