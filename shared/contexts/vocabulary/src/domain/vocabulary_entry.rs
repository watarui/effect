//! 語彙エントリードメインモデル（Vocabulary Context 内で共有）
//!
//! 軽量な語彙エントリー（検索・一覧表示用）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{CefrLevel, EntryId};

use super::{Domain, PartOfSpeech, Register};

/// 語彙エントリー（軽量版）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabularyEntry {
    pub id:                 EntryId,
    pub word:               String,
    pub primary_definition: String,
    pub part_of_speech:     PartOfSpeech,
    pub cefr_level:         Option<CefrLevel>,
    pub register:           Register,
    pub domain:             Domain,
    pub created_at:         DateTime<Utc>,
    pub updated_at:         DateTime<Utc>,
}

impl VocabularyEntry {
    /// 新しい語彙エントリーを作成
    pub fn new(
        word: String,
        primary_definition: String,
        part_of_speech: PartOfSpeech,
        cefr_level: Option<CefrLevel>,
        register: Register,
        domain: Domain,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: EntryId::new(),
            word,
            primary_definition,
            part_of_speech,
            cefr_level,
            register,
            domain,
            created_at: now,
            updated_at: now,
        }
    }
}
