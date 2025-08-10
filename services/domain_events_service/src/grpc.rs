//! gRPC サーバー実装

use std::{net::SocketAddr, sync::Arc};

use tonic::{Request, Response, Status, transport::Server};
use tracing::info;

use crate::{
    config::Config,
    registry::{SchemaInfo, SchemaRegistry},
    validator::{EventValidator, ValidationError},
};

// Protocol Buffers から生成されたコード
#[allow(clippy::all)]
#[allow(clippy::pedantic)]
#[allow(clippy::restriction)]
#[allow(warnings)]
pub mod proto {
    tonic::include_proto!("effect.domain_events");
}

use proto::{
    EventTypeInfo,
    GetSchemaRequest,
    GetSchemaResponse,
    GetSchemaVersionRequest,
    GetSchemaVersionResponse,
    ListEventTypesRequest,
    ListEventTypesResponse,
    RegisterSchemaRequest,
    RegisterSchemaResponse,
    Schema,
    ValidateEventRequest,
    ValidateEventResponse,
    domain_events_service_server::{DomainEventsService, DomainEventsServiceServer},
};

/// Domain Events Service の gRPC 実装
pub struct DomainEventsServiceImpl {
    registry:  Arc<SchemaRegistry>,
    validator: Arc<EventValidator>,
}

#[tonic::async_trait]
impl DomainEventsService for DomainEventsServiceImpl {
    async fn get_schema(
        &self,
        request: Request<GetSchemaRequest>,
    ) -> Result<Response<GetSchemaResponse>, Status> {
        let req = request.into_inner();

        let schema = self
            .registry
            .get_schema(&req.event_type, req.version)
            .await
            .map_err(|e| Status::not_found(format!("Schema not found: {e}")))?;

        Ok(Response::new(GetSchemaResponse {
            schema: Some(schema_info_to_proto(schema)),
        }))
    }

    async fn validate_event(
        &self,
        request: Request<ValidateEventRequest>,
    ) -> Result<Response<ValidateEventResponse>, Status> {
        let req = request.into_inner();

        let event_data = req
            .event_data
            .ok_or_else(|| Status::invalid_argument("event_data is required"))?
            .value;

        let errors = self
            .validator
            .validate_event(&req.event_type, &event_data, req.schema_version)
            .await
            .map_err(|e| Status::internal(format!("Validation failed: {e}")))?;

        let is_valid = errors.is_empty();
        let proto_errors = errors.into_iter().map(validation_error_to_proto).collect();

        Ok(Response::new(ValidateEventResponse {
            is_valid,
            errors: proto_errors,
        }))
    }

    async fn register_schema(
        &self,
        request: Request<RegisterSchemaRequest>,
    ) -> Result<Response<RegisterSchemaResponse>, Status> {
        let req = request.into_inner();

        let (schema_id, version) = self
            .registry
            .register_schema(&req.event_type, &req.schema_definition, &req.description)
            .await
            .map_err(|e| Status::internal(format!("Failed to register schema: {e}")))?;

        Ok(Response::new(RegisterSchemaResponse {
            schema_id: schema_id.to_string(),
            version,
        }))
    }

    async fn list_event_types(
        &self,
        request: Request<ListEventTypesRequest>,
    ) -> Result<Response<ListEventTypesResponse>, Status> {
        let req = request.into_inner();

        let event_types = self
            .registry
            .list_event_types(req.context.as_deref())
            .await
            .map_err(|e| Status::internal(format!("Failed to list event types: {e}")))?;

        let proto_event_types = event_types
            .into_iter()
            .map(|et| EventTypeInfo {
                event_type:      et.event_type,
                context:         et.context,
                description:     et.description,
                current_version: et.current_version,
                is_deprecated:   et.is_deprecated,
            })
            .collect();

        Ok(Response::new(ListEventTypesResponse {
            event_types: proto_event_types,
        }))
    }

    async fn get_schema_version(
        &self,
        request: Request<GetSchemaVersionRequest>,
    ) -> Result<Response<GetSchemaVersionResponse>, Status> {
        let req = request.into_inner();

        let (current_version, available_versions) = self
            .registry
            .get_schema_versions(&req.event_type)
            .await
            .map_err(|e| Status::not_found(format!("Event type not found: {e}")))?;

        Ok(Response::new(GetSchemaVersionResponse {
            current_version,
            available_versions,
        }))
    }
}

/// gRPC サーバーを起動
pub async fn start_server(
    config: Config,
    registry: SchemaRegistry,
    validator: EventValidator,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("0.0.0.0:{}", config.grpc.port).parse()?;

    let service = DomainEventsServiceImpl {
        registry:  Arc::new(registry),
        validator: Arc::new(validator),
    };

    info!("Domain Events Service gRPC server listening on {}", addr);

    Server::builder()
        .add_service(DomainEventsServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

// ヘルパー関数

fn schema_info_to_proto(schema: SchemaInfo) -> Schema {
    Schema {
        id:          schema.id.to_string(),
        event_type:  schema.event_type,
        version:     schema.version,
        definition:  schema.definition,
        description: schema.description,
        created_at:  Some(prost_types::Timestamp {
            seconds: schema.created_at.timestamp(),
            nanos:   i32::try_from(schema.created_at.timestamp_subsec_nanos()).unwrap_or(0),
        }),
        updated_at:  Some(prost_types::Timestamp {
            seconds: schema.updated_at.timestamp(),
            nanos:   i32::try_from(schema.updated_at.timestamp_subsec_nanos()).unwrap_or(0),
        }),
    }
}

fn validation_error_to_proto(error: ValidationError) -> proto::ValidationError {
    proto::ValidationError {
        field:   error.field,
        message: error.message,
        code:    error.code,
    }
}
