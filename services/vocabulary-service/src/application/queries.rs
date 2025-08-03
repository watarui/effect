//! Vocabulary Service のクエリ定義

use common_types::ItemId;
use serde::{Deserialize, Serialize};

/// 語彙項目取得クエリ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetItem {
    /// 取得対象の項目 ID
    pub item_id: ItemId,
}

/// 語彙項目一括取得クエリ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetItems {
    /// 取得対象の項目 ID リスト（最大100件）
    pub item_ids: Vec<ItemId>,
}

/// 語彙エントリー検索クエリ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchEntries {
    /// 検索クエリ（部分一致）
    pub query:  String,
    /// 最大取得件数（デフォルト: 10、最大: 50）
    pub limit:  Option<u32>,
    /// オフセット
    pub offset: Option<u32>,
}

/// 語彙エントリー取得クエリ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetEntry {
    /// エントリー ID
    pub entry_id: String,
}

/// 最近追加された項目取得クエリ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetRecentItems {
    /// 最大取得件数（デフォルト: 20、最大: 100）
    pub limit: Option<u32>,
    /// この日時以降のみ取得
    pub since: Option<chrono::DateTime<chrono::Utc>>,
}

/// 語彙項目履歴取得クエリ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetItemHistory {
    /// 対象の項目 ID
    pub item_id: ItemId,
    /// 最大取得件数（デフォルト: 20）
    pub limit:   Option<u32>,
    /// オフセット
    pub offset:  Option<u32>,
}
