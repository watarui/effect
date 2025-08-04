//! gRPC サーバー実装
//!
//! Vocabulary Query Service の gRPC エンドポイント

use std::sync::Arc;

use crate::application::query_handlers::{GetEntryHandler, GetItemHandler, GetStatsHandler};

// TODO: Proto ファイルから生成されたコードを include する
// 現在は一時的に空のモジュールを定義

/// gRPC サーバー実装
pub struct VocabularyQueryGrpcServer {
    _get_item_handler:  Arc<GetItemHandler>,
    _get_entry_handler: Arc<GetEntryHandler>,
    _get_stats_handler: Arc<GetStatsHandler>,
}

impl VocabularyQueryGrpcServer {
    /// 新しいサーバーを作成
    pub fn new(
        get_item_handler: Arc<GetItemHandler>,
        get_entry_handler: Arc<GetEntryHandler>,
        get_stats_handler: Arc<GetStatsHandler>,
    ) -> Self {
        Self {
            _get_item_handler:  get_item_handler,
            _get_entry_handler: get_entry_handler,
            _get_stats_handler: get_stats_handler,
        }
    }
}

// TODO: VocabularyQueryService trait の実装を追加
