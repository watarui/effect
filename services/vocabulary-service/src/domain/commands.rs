//! Vocabulary Service のコマンド定義

use common_types::{ItemId, UserId};
use domain_events::CefrLevel;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{
    domain::Domain,
    part_of_speech::PartOfSpeech,
    register::Register,
};

/// 語彙項目作成コマンド
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateItem {
    /// 単語の綴り
    pub word:           String,
    /// 品詞
    pub part_of_speech: PartOfSpeech,
    /// CEFR レベル（オプション）
    pub cefr_level:     Option<CefrLevel>,
    /// レジスター（言語使用域）
    pub register:       Register,
    /// ドメイン（専門分野）
    pub domain:         Domain,
    /// 定義のリスト
    pub definitions:    Vec<String>,
    /// 作成者のユーザー ID
    pub created_by:     UserId,
}

/// 語彙項目更新コマンド
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateItem {
    /// 更新対象の項目 ID
    pub item_id:          ItemId,
    /// 品詞（更新する場合）
    pub part_of_speech:   Option<PartOfSpeech>,
    /// CEFR レベル（更新する場合）
    pub cefr_level:       Option<CefrLevel>,
    /// レジスター（更新する場合）
    pub register:         Option<Register>,
    /// ドメイン（更新する場合）
    pub domain:           Option<Domain>,
    /// 定義のリスト（更新する場合）
    pub definitions:      Option<Vec<String>>,
    /// 発音記号（更新する場合）
    pub pronunciation:    Option<String>,
    /// 同義語（更新する場合）
    pub synonyms:         Option<Vec<String>>,
    /// 反意語（更新する場合）
    pub antonyms:         Option<Vec<String>>,
    /// 例文（更新する場合）
    pub examples:         Option<Vec<String>>,
    /// コロケーション（更新する場合）
    pub collocations:     Option<Vec<String>>,
    /// 更新者のユーザー ID
    pub updated_by:       UserId,
    /// 楽観的ロック用のバージョン
    pub expected_version: u64,
}

/// 語彙項目削除コマンド
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteItem {
    /// 削除対象の項目 ID
    pub item_id:    ItemId,
    /// 削除者のユーザー ID
    pub deleted_by: UserId,
}

/// AI 生成リクエストコマンド
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestAIGeneration {
    /// 対象の項目 ID
    pub item_id:         ItemId,
    /// 再生成かどうか
    pub is_regeneration: bool,
    /// リクエスト者のユーザー ID
    pub requested_by:    UserId,
}
