//! インフラストラクチャ層
//!
//! Event Store、Pub/Sub との連携実装

/// Event Store リポジトリ（仮実装）
pub struct EventStoreRepository;

impl EventStoreRepository {
    /// 新しいリポジトリを作成
    pub fn new() -> Self {
        Self
    }
}

impl Default for EventStoreRepository {
    fn default() -> Self {
        Self::new()
    }
}
