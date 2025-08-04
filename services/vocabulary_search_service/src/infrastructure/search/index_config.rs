//! Meilisearch インデックス設定

use crate::domain::error::SearchError;

/// Meilisearch インデックスを設定
pub async fn configure_meilisearch_index(
    _client: &meilisearch_sdk::client::Client,
    _index_name: &str,
) -> Result<(), SearchError> {
    // TODO: Meilisearch SDK のバージョンアップ後に実装
    Ok(())
}

/// スワップ設定を作成
pub async fn create_swap_settings(_old_index: &str, _new_index: &str) -> Result<(), SearchError> {
    // TODO: Meilisearch SDK のバージョンアップ後に実装
    Ok(())
}
