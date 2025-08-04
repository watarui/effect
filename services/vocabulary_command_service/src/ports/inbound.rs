//! インバウンドポート
//!
//! 外部からサービスを呼び出すためのインターフェース

use async_trait::async_trait;
use shared_error::DomainResult;
use shared_vocabulary_context::commands::*;
use uuid::Uuid;

/// コマンドサービスのインターフェース
#[async_trait]
pub trait CommandService: Send + Sync {
    /// 語彙項目を作成
    async fn create_vocabulary_item(&self, command: CreateVocabularyItem) -> DomainResult<Uuid>;

    /// 語彙項目を更新
    async fn update_vocabulary_item(&self, command: UpdateVocabularyItem) -> DomainResult<()>;

    /// 語彙項目を削除
    async fn delete_vocabulary_item(&self, command: DeleteVocabularyItem) -> DomainResult<()>;

    /// 例文を追加
    async fn add_example(&self, command: AddExample) -> DomainResult<()>;

    /// AI エンリッチメントを要求
    async fn request_ai_enrichment(&self, command: RequestAiEnrichment) -> DomainResult<()>;
}
