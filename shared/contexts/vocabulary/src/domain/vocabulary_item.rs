//! 語彙項目ドメインモデル（Vocabulary Context 内で共有）
//!
//! Vocabulary Context 内の各マイクロサービスで共有される
//! VocabularyItem の定義。各サービスは必要に応じて
//! このモデルを拡張または投影して使用する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{CefrLevel, ItemId};

use super::{Definition, Domain, PartOfSpeech, Register};

/// 語彙項目（Vocabulary Context 共通モデル）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabularyItem {
    pub id:             ItemId,
    pub word:           String,
    pub definitions:    Vec<Definition>,
    pub part_of_speech: PartOfSpeech,
    pub cefr_level:     Option<CefrLevel>,
    pub register:       Register,
    pub domain:         Domain,
    pub pronunciation:  Option<String>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

impl VocabularyItem {
    /// 新しい語彙項目を作成
    pub fn new(
        word: String,
        definitions: Vec<Definition>,
        part_of_speech: PartOfSpeech,
        cefr_level: Option<CefrLevel>,
        register: Register,
        domain: Domain,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ItemId::new(),
            word,
            definitions,
            part_of_speech,
            cefr_level,
            register,
            domain,
            pronunciation: None,
            created_at: now,
            updated_at: now,
        }
    }
}
