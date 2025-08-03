//! Vocabulary Context のコマンド

use serde::{Deserialize, Serialize};
use shared_kernel::{CefrLevel, ItemId, UserId};

use crate::domain::{Domain, PartOfSpeech, Register};

/// 語彙項目を作成
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateVocabularyItem {
    pub word:           String,
    pub definitions:    Vec<String>,
    pub part_of_speech: PartOfSpeech,
    pub cefr_level:     Option<CefrLevel>,
    pub register:       Register,
    pub domain:         Domain,
    pub user_id:        UserId,
}

/// 語彙項目を更新
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateVocabularyItem {
    pub item_id:     ItemId,
    pub definitions: Option<Vec<String>>,
    pub cefr_level:  Option<CefrLevel>,
    pub user_id:     UserId,
}

/// 語彙項目を削除
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteVocabularyItem {
    pub item_id: ItemId,
    pub user_id: UserId,
}

/// 例文を追加
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddExample {
    pub item_id:          ItemId,
    pub definition_index: usize,
    pub example:          String,
    pub user_id:          UserId,
}

/// AI による情報生成を要求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestAiEnrichment {
    pub item_id: ItemId,
    pub word:    String,
    pub context: String,
}
