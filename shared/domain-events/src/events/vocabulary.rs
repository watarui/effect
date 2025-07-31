//! Vocabulary Context イベント

use common_types::{ItemId, UserId};
use serde::{Deserialize, Serialize};

use crate::EventMetadata;

/// Vocabulary Context のイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VocabularyEvent {
    /// 語彙エントリーが作成された
    EntryCreated {
        /// イベントメタデータ
        metadata: EventMetadata,
        /// エントリー ID
        entry_id: ItemId,
        /// スペリング
        spelling: String,
    },
    /// 項目が作成された
    ItemCreated {
        /// イベントメタデータ
        metadata:   EventMetadata,
        /// 項目 ID
        item_id:    ItemId,
        /// スペリング
        spelling:   String,
        /// 作成者
        created_by: UserId,
    },
    /// AI 生成が要求された
    AIGenerationRequested {
        /// イベントメタデータ
        metadata: EventMetadata,
        /// 項目 ID
        item_id:  ItemId,
        /// スペリング
        spelling: String,
    },
    /// AI 情報が生成された
    AIInfoGenerated {
        /// イベントメタデータ
        metadata:      EventMetadata,
        /// 項目 ID
        item_id:       ItemId,
        /// 発音
        pronunciation: String,
        /// 定義リスト
        definitions:   Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_created_should_have_creator() {
        let user_id = UserId::new();
        let event = VocabularyEvent::ItemCreated {
            metadata:   EventMetadata::new(),
            item_id:    ItemId::new(),
            spelling:   "example".to_string(),
            created_by: user_id,
        };

        match event {
            VocabularyEvent::ItemCreated {
                created_by,
                spelling,
                ..
            } => {
                assert_eq!(created_by, user_id);
                assert_eq!(spelling, "example");
            },
            _ => unreachable!("Expected ItemCreated event"),
        }
    }

    #[test]
    fn ai_info_generated_should_contain_definitions() {
        let definitions = vec!["定義1".to_string(), "定義2".to_string()];

        let event = VocabularyEvent::AIInfoGenerated {
            metadata:      EventMetadata::new(),
            item_id:       ItemId::new(),
            pronunciation: "/ɪɡˈzæmpəl/".to_string(),
            definitions:   definitions.clone(),
        };

        match event {
            VocabularyEvent::AIInfoGenerated {
                definitions: defs, ..
            } => {
                assert_eq!(defs.len(), 2);
                assert_eq!(defs, definitions);
            },
            _ => unreachable!("Expected AIInfoGenerated event"),
        }
    }
}
