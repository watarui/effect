//! Vocabulary Context のイベントスキーマ定義

use std::collections::HashMap;

/// Vocabulary Context のスキーマを取得
#[must_use]
#[allow(dead_code, clippy::too_many_lines)]
pub fn get_schemas() -> HashMap<String, String> {
    let mut schemas = HashMap::new();

    // EntryCreated イベント
    schemas.insert(
        "vocabulary.EntryCreated".to_string(),
        r##"{
            "type": "object",
            "required": ["entry_id", "spelling", "metadata"],
            "properties": {
                "entry_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "spelling": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 100
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    // ItemCreated イベント
    schemas.insert(
        "vocabulary.ItemCreated".to_string(),
        r##"{
            "type": "object",
            "required": ["item_id", "entry_id", "meaning", "metadata"],
            "properties": {
                "item_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "entry_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "meaning": {
                    "type": "object",
                    "required": ["disambiguator", "definition"],
                    "properties": {
                        "disambiguator": {
                            "type": "string"
                        },
                        "definition": {
                            "type": "string"
                        },
                        "part_of_speech": {
                            "type": "string",
                            "enum": ["noun", "verb", "adjective", "adverb", "pronoun", "preposition", "conjunction", "interjection"]
                        }
                    }
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##.to_string(),
    );

    // FieldUpdated イベント
    schemas.insert(
        "vocabulary.FieldUpdated".to_string(),
        r##"{
            "type": "object",
            "required": ["item_id", "field_name", "old_value", "new_value", "metadata"],
            "properties": {
                "item_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "field_name": {
                    "type": "string"
                },
                "old_value": {
                    "type": ["string", "null"]
                },
                "new_value": {
                    "type": ["string", "null"]
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    // AiGenerationRequested イベント
    schemas.insert(
        "vocabulary.AiGenerationRequested".to_string(),
        r##"{
            "type": "object",
            "required": ["item_id", "fields_to_generate", "request_id", "metadata"],
            "properties": {
                "item_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "fields_to_generate": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "minItems": 1
                },
                "request_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    // AiGenerationCompleted イベント
    schemas.insert(
        "vocabulary.AiGenerationCompleted".to_string(),
        r##"{
            "type": "object",
            "required": ["item_id", "request_id", "generated_fields", "metadata"],
            "properties": {
                "item_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "request_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "generated_fields": {
                    "type": "object",
                    "additionalProperties": {
                        "type": "string"
                    }
                },
                "metadata": {
                    "$ref": "#/definitions/EventMetadata"
                }
            }
        }"##
        .to_string(),
    );

    // ItemPublished イベント
    schemas.insert(
        "vocabulary.ItemPublished".to_string(),
        r##"{
            "type": "object",
            "required": ["item_id", "version", "metadata"],
            "properties": {
                "item_id": {
                    "type": "string",
                    "format": "uuid"
                },
                "version": {
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

    schemas
}
