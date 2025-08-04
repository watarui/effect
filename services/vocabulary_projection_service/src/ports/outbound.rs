//! アウトバウンドポート
//!
//! 外部リソースにアクセスするためのインターフェース

use async_trait::async_trait;
use shared_error::DomainResult;
use uuid::Uuid;

use crate::domain::read_models::{ProjectionState, VocabularyItemView};

/// Read Model リポジトリのインターフェース
#[async_trait]
pub trait ReadModelRepository: Send + Sync {
    /// 語彙項目ビューを保存（upsert）
    async fn save_item_view(&self, view: &VocabularyItemView) -> DomainResult<()>;

    /// 語彙項目ビューを取得
    async fn get_item_view(&self, item_id: Uuid) -> DomainResult<Option<VocabularyItemView>>;

    /// 語彙項目ビューを更新
    async fn update_item_view(&self, view: &VocabularyItemView) -> DomainResult<()>;

    /// 語彙項目ビューを削除
    async fn delete_item_view(&self, item_id: Uuid) -> DomainResult<()>;

    /// エントリIDで語彙項目ビューを取得
    async fn get_item_views_by_entry(
        &self,
        entry_id: Uuid,
    ) -> DomainResult<Vec<VocabularyItemView>>;
}

/// プロジェクション状態リポジトリのインターフェース
#[async_trait]
pub trait ProjectionStateRepository: Send + Sync {
    /// プロジェクション状態を取得
    async fn get_state(&self, projection_name: &str) -> DomainResult<Option<ProjectionState>>;

    /// プロジェクション状態を保存（upsert）
    async fn save_state(&self, state: &ProjectionState) -> DomainResult<()>;
}

/// イベントサブスクライバーのインターフェース
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// サブスクリプションを開始
    async fn subscribe(&self) -> DomainResult<()>;

    /// サブスクリプションを停止
    async fn unsubscribe(&self) -> DomainResult<()>;
}
