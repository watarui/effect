//! gRPC サーバー実装

use std::sync::Arc;

// TODO: Proto ファイルから生成される型をインポート
use crate::application::CommandHandler;

/// Vocabulary Command Service の gRPC 実装
#[allow(dead_code)]
pub struct VocabularyCommandService {
    command_handler: Arc<CommandHandler>,
}

impl VocabularyCommandService {
    /// 新しい gRPC サービスを作成
    pub fn new(command_handler: Arc<CommandHandler>) -> Self {
        Self { command_handler }
    }
}

// TODO: Proto ファイルから生成される trait を実装
// #[tonic::async_trait]
// impl vocabulary_command_service_server::VocabularyCommandService for
// VocabularyCommandService {     async fn create_vocabulary_item(
//         &self,
//         request: Request<CreateVocabularyItemRequest>,
//     ) -> Result<Response<CreateVocabularyItemResponse>, Status> {
//         // コマンドに変換して処理
//     }
// }
