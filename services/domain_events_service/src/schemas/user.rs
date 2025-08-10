//! User Context のイベントスキーマ定義

use std::collections::HashMap;

/// User Context のスキーマを取得
#[must_use]
#[allow(dead_code, clippy::too_many_lines)]
pub fn get_schemas() -> HashMap<String, String> {
    let mut schemas = HashMap::new();

    // UserSignedUp イベント
    schemas.insert(
        "user.UserSignedUp".to_string(),
        r##"{
            "type": "object",
            "required": ["user_id", "email", "display_name", "metadata"],
            "properties": {
                "user_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "email": {
                    "type": "string",
                    "format": "email"
                },
                "display_name": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 100
                },
                "photo_url": {
                    "type": ["string", "null"],
                    "format": "uri"
                },
                "initial_role": {
                    "type": "string",
                    "enum": ["user", "admin", "moderator"]
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    // ProfileUpdated イベント
    schemas.insert(
        "user.ProfileUpdated".to_string(),
        r##"{
            "type": "object",
            "required": ["user_id", "updated_fields", "metadata"],
            "properties": {
                "user_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "updated_fields": {
                    "type": "object",
                    "properties": {
                        "display_name": {
                            "type": "string"
                        },
                        "photo_url": {
                            "type": "string",
                            "format": "uri"
                        },
                        "bio": {
                            "type": "string"
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

    // LearningGoalSet イベント
    schemas.insert(
        "user.LearningGoalSet".to_string(),
        r##"{
            "type": "object",
            "required": ["user_id", "goal", "metadata"],
            "properties": {
                "user_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "goal": {
                    "type": "object",
                    "required": ["target_level", "daily_items", "deadline"],
                    "properties": {
                        "target_level": {
                            "type": "string",
                            "enum": ["A1", "A2", "B1", "B2", "C1", "C2"]
                        },
                        "daily_items": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 100
                        },
                        "deadline": {
                            "type": "string",
                            "format": "date-time"
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
