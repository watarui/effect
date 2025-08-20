//! 出力ポート（外部システムとのインターフェース）

use async_trait::async_trait;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::{
        events::StoredEvent,
        projections::{
            ProjectionCheckpoint,
            ProjectionState,
            VocabularyEntryProjection,
            VocabularyExampleProjection,
            VocabularyItemProjection,
        },
    },
    error::Result,
};

/// AI エンリッチメントデータ
pub struct ItemEnrichmentData {
    pub part_of_speech:    Option<String>,
    pub definition:        Option<String>,
    pub ipa_pronunciation: Option<String>,
    pub cefr_level:        Option<String>,
    pub frequency_rank:    Option<i32>,
}

/// イベントストアからのイベント購読
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// 指定位置からイベントを取得
    async fn fetch_events(&self, from_position: i64, batch_size: usize)
    -> Result<Vec<StoredEvent>>;

    /// イベントストリームを購読
    async fn subscribe(&self, from_position: i64) -> Result<EventStream>;
}

/// イベントストリーム
pub struct EventStream {
    // 実装詳細は infrastructure 層で定義
}

/// Read Model リポジトリ
#[async_trait]
pub trait ReadModelRepository: Send + Sync {
    /// VocabularyEntry を永続化
    async fn save_entry(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        entry: &VocabularyEntryProjection,
    ) -> Result<()>;

    /// VocabularyItem を永続化
    async fn save_item(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item: &VocabularyItemProjection,
    ) -> Result<()>;

    /// 例文を追加
    async fn add_example(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        example: &VocabularyExampleProjection,
    ) -> Result<()>;

    /// Item の公開状態を更新
    async fn update_item_published(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item_id: Uuid,
        is_published: bool,
        version: i64,
    ) -> Result<()>;

    /// Item の削除状態を更新
    async fn update_item_deleted(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item_id: Uuid,
        is_deleted: bool,
        version: i64,
    ) -> Result<()>;

    /// AI エンリッチメントデータを更新
    async fn update_item_enrichment(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item_id: Uuid,
        enrichment: ItemEnrichmentData,
        version: i64,
    ) -> Result<()>;

    /// Entry の主要項目を設定
    async fn update_entry_primary_item(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        entry_id: Uuid,
        primary_item_id: Option<Uuid>,
        version: i64,
    ) -> Result<()>;

    /// Item カウントを更新
    async fn update_item_count(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        entry_id: Uuid,
    ) -> Result<()>;

    /// 例文カウントを増やす
    async fn increment_example_count(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item_id: Uuid,
    ) -> Result<()>;

    /// トランザクションを開始
    async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>>;
}

/// プロジェクション状態リポジトリ
#[async_trait]
pub trait ProjectionStateRepository: Send + Sync {
    /// プロジェクション状態を取得
    async fn get_state(&self, name: &str) -> Result<Option<ProjectionState>>;

    /// プロジェクション状態を保存
    async fn save_state(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        state: &ProjectionState,
    ) -> Result<()>;

    /// エラーを記録
    async fn record_error(&self, name: &str, error: &str) -> Result<()>;

    /// チェックポイントを保存
    async fn save_checkpoint(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        checkpoint: &ProjectionCheckpoint,
    ) -> Result<()>;
}
