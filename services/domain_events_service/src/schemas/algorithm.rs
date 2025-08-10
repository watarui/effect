//! Algorithm Context のイベントスキーマ定義

use std::collections::HashMap;

/// Algorithm Context のスキーマを取得
#[must_use]
#[allow(dead_code, clippy::too_many_lines)]
pub fn get_schemas() -> HashMap<String, String> {
    let mut schemas = HashMap::new();

    // ReviewScheduleUpdated イベント
    schemas.insert(
        "algorithm.ReviewScheduleUpdated".to_string(),
        r##"{
            "type": "object",
            "required": ["item_id", "user_id", "next_review", "interval_days", "metadata"],
            "properties": {
                "item_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "user_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "next_review": {
                    "type": "string",
                    "format": "date-time"
                },
                "interval_days": {
                    "type": "number",
                    "minimum": 0
                },
                "easiness_factor": {
                    "type": "number",
                    "minimum": 1.3
                },
                "repetition_count": {
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

    // DifficultyAdjusted イベント
    schemas.insert(
        "algorithm.DifficultyAdjusted".to_string(),
        r##"{
            "type": "object",
            "required": ["item_id", "user_id", "old_difficulty", "new_difficulty", "metadata"],
            "properties": {
                "item_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "user_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "old_difficulty": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 1
                },
                "new_difficulty": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 1
                },
                "reason": {
                    "type": "string"
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    // PerformanceAnalyzed イベント
    schemas.insert(
        "algorithm.PerformanceAnalyzed".to_string(),
        r##"{
            "type": "object",
            "required": ["user_id", "analysis", "metadata"],
            "properties": {
                "user_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "analysis": {
                    "type": "object",
                    "required": ["accuracy_rate", "retention_rate", "learning_velocity"],
                    "properties": {
                        "accuracy_rate": {
                            "type": "number",
                            "minimum": 0,
                            "maximum": 1
                        },
                        "retention_rate": {
                            "type": "number",
                            "minimum": 0,
                            "maximum": 1
                        },
                        "learning_velocity": {
                            "type": "number",
                            "minimum": 0
                        },
                        "period_days": {
                            "type": "integer",
                            "minimum": 1
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
