use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    application::commands::{
        CreateVocabularyItemHandler,
        DeleteVocabularyItemHandler,
        UpdateVocabularyItemHandler,
    },
    domain::{
        CreateVocabularyItem,
        DeleteVocabularyItem,
        Disambiguation,
        EntryId,
        ItemId,
        UpdateVocabularyItem,
    },
    error::Error,
};

// Proto から生成されたコード
pub mod proto {
    pub mod effect {
        pub mod common {
            tonic::include_proto!("effect.common");
        }
        pub mod services {
            pub mod vocabulary_command {
                tonic::include_proto!("effect.services.vocabulary_command");
            }
        }
    }

    // 再エクスポート
    pub use effect::{common::*, services::vocabulary_command::*};
}

use proto::{
    AddExampleRequest,
    AddExampleResponse,
    CreateVocabularyItemRequest,
    CreateVocabularyItemResponse,
    DeleteVocabularyItemRequest,
    DeleteVocabularyItemResponse,
    RequestAiEnrichmentRequest,
    RequestAiEnrichmentResponse,
    UpdateVocabularyItemRequest,
    UpdateVocabularyItemResponse,
    vocabulary_command_service_server::VocabularyCommandService,
};

/// Vocabulary Command Service の gRPC 実装
pub struct VocabularyCommandServiceImpl<ER, IR, ES>
where
    ER: crate::ports::repositories::VocabularyEntryRepository + Send + Sync,
    IR: crate::ports::repositories::VocabularyItemRepository + Send + Sync,
    ES: crate::ports::event_store::EventStore + Send + Sync,
{
    create_handler: Arc<CreateVocabularyItemHandler<ER, IR, ES>>,
    update_handler: Arc<UpdateVocabularyItemHandler<IR, ES>>,
    delete_handler: Arc<DeleteVocabularyItemHandler<ER, IR, ES>>,
}

impl<ER, IR, ES> VocabularyCommandServiceImpl<ER, IR, ES>
where
    ER: crate::ports::repositories::VocabularyEntryRepository + Send + Sync,
    IR: crate::ports::repositories::VocabularyItemRepository + Send + Sync,
    ES: crate::ports::event_store::EventStore + Send + Sync,
{
    pub fn new(
        create_handler: Arc<CreateVocabularyItemHandler<ER, IR, ES>>,
        update_handler: Arc<UpdateVocabularyItemHandler<IR, ES>>,
        delete_handler: Arc<DeleteVocabularyItemHandler<ER, IR, ES>>,
    ) -> Self {
        Self {
            create_handler,
            update_handler,
            delete_handler,
        }
    }
}

#[tonic::async_trait]
impl<ER, IR, ES> VocabularyCommandService for VocabularyCommandServiceImpl<ER, IR, ES>
where
    ER: crate::ports::repositories::VocabularyEntryRepository + Send + Sync + 'static,
    IR: crate::ports::repositories::VocabularyItemRepository + Send + Sync + 'static,
    ES: crate::ports::event_store::EventStore + Send + Sync + 'static,
{
    async fn create_vocabulary_item(
        &self,
        request: Request<CreateVocabularyItemRequest>,
    ) -> Result<Response<CreateVocabularyItemResponse>, Status> {
        let req = request.into_inner();

        // 新しいエントリIDを生成
        let entry_id = EntryId::new();

        let command = CreateVocabularyItem {
            entry_id:       *entry_id.as_uuid(),
            spelling:       req.word.clone(),
            disambiguation: if req.definitions.is_empty() {
                None
            } else {
                Some(req.definitions[0].clone())
            },
        };

        // ハンドラーを実行
        let item = self
            .create_handler
            .handle(command)
            .await
            .map_err(|e| match e {
                Error::Validation(msg) => Status::invalid_argument(msg),
                Error::Conflict(msg) => Status::already_exists(msg),
                _ => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(CreateVocabularyItemResponse {
            item_id: item.item_id.to_string(),
        }))
    }

    async fn update_vocabulary_item(
        &self,
        request: Request<UpdateVocabularyItemRequest>,
    ) -> Result<Response<UpdateVocabularyItemResponse>, Status> {
        let req = request.into_inner();

        // item_id をパース
        let item_id = ItemId::from_uuid(
            Uuid::parse_str(&req.item_id)
                .map_err(|e| Status::invalid_argument(format!("Invalid item_id: {}", e)))?,
        );

        // updates から disambiguation の更新を取得
        let mut new_disambiguation = None;
        for update in req.updates {
            if update.field_name == "disambiguation" {
                // JSON 形式の値をパース
                let value: Option<String> = serde_json::from_str(&update.value_json)
                    .map_err(|e| Status::invalid_argument(format!("Invalid JSON value: {}", e)))?;
                new_disambiguation = Some(Disambiguation::new(value).map_err(|e| {
                    Status::invalid_argument(format!("Invalid disambiguation: {}", e))
                })?);
            }
        }

        if new_disambiguation.is_none() {
            return Err(Status::invalid_argument("No valid updates provided"));
        }

        let command = UpdateVocabularyItem {
            item_id:        *item_id.as_uuid(),
            disambiguation: new_disambiguation
                .unwrap()
                .as_option()
                .map(|s| s.to_string()),
            version:        req.expected_version as i64,
        };

        // ハンドラーを実行
        let item = self
            .update_handler
            .handle(command)
            .await
            .map_err(|e| match e {
                Error::NotFound(msg) => Status::not_found(msg),
                Error::Conflict(msg) => Status::aborted(msg),
                Error::Validation(msg) => Status::invalid_argument(msg),
                _ => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(UpdateVocabularyItemResponse {
            new_version: item.version.value() as u32,
        }))
    }

    async fn delete_vocabulary_item(
        &self,
        request: Request<DeleteVocabularyItemRequest>,
    ) -> Result<Response<DeleteVocabularyItemResponse>, Status> {
        let req = request.into_inner();

        // メタデータからユーザーIDを取得
        let metadata = req
            .metadata
            .ok_or_else(|| Status::invalid_argument("metadata is required"))?;

        // プロトコルバッファからドメインモデルへ変換
        let command = DeleteVocabularyItem {
            item_id:    Uuid::parse_str(&req.item_id)
                .map_err(|e| Status::invalid_argument(format!("Invalid item_id: {}", e)))?,
            deleted_by: Uuid::parse_str(&metadata.issued_by)
                .map_err(|e| Status::invalid_argument(format!("Invalid issued_by: {}", e)))?,
        };

        // ハンドラー実行
        self.delete_handler
            .handle(command)
            .await
            .map_err(|e| match e {
                Error::NotFound(msg) => Status::not_found(msg),
                Error::Conflict(msg) => Status::already_exists(msg),
                _ => Status::internal(format!("Failed to delete vocabulary item: {}", e)),
            })?;

        Ok(Response::new(DeleteVocabularyItemResponse {}))
    }

    async fn add_example(
        &self,
        _request: Request<AddExampleRequest>,
    ) -> Result<Response<AddExampleResponse>, Status> {
        // TODO: AddExampleHandler を実装後に対応
        Err(Status::unimplemented("Add example is not implemented yet"))
    }

    async fn request_ai_enrichment(
        &self,
        _request: Request<RequestAiEnrichmentRequest>,
    ) -> Result<Response<RequestAiEnrichmentResponse>, Status> {
        // TODO: AI Service との連携実装後に対応
        Err(Status::unimplemented(
            "Request AI enrichment is not implemented yet",
        ))
    }
}
