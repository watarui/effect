//! Vocabulary Context のクエリ

use serde::{Deserialize, Serialize};
use shared_kernel::{CefrLevel, ItemId};

use crate::domain::{Domain, PartOfSpeech};

/// 語彙項目を取得
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetVocabularyItem {
    pub item_id: ItemId,
}

/// 語彙項目を検索
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchVocabularyItems {
    pub query:          Option<String>,
    pub part_of_speech: Option<PartOfSpeech>,
    pub cefr_level:     Option<CefrLevel>,
    pub domain:         Option<Domain>,
    pub limit:          Option<usize>,
    pub offset:         Option<usize>,
}

/// 語彙エントリーを検索
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchVocabularyEntries {
    pub query:      Option<String>,
    pub cefr_level: Option<CefrLevel>,
    pub limit:      Option<usize>,
    pub offset:     Option<usize>,
}
