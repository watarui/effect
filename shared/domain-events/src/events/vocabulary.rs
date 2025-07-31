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
