//! Domain Events Service クライアントライブラリ
//!
//! 他のサービスから Domain Events Service に接続するためのクライアント

use tonic::transport::Channel;

use crate::grpc::proto::{
    EventTypeInfo,
    GetSchemaRequest,
    ListEventTypesRequest,
    Schema,
    ValidateEventRequest,
    domain_events_service_client::DomainEventsServiceClient,
};

/// Domain Events クライアント
pub struct DomainEventsClient {
    inner: DomainEventsServiceClient<Channel>,
}

impl DomainEventsClient {
    /// 新しいクライアントを作成
    pub async fn new(url: String) -> Result<Self, tonic::transport::Error> {
        let inner = DomainEventsServiceClient::connect(url).await?;
        Ok(Self { inner })
    }

    /// スキーマを取得
    pub async fn get_schema(
        &mut self,
        event_type: String,
        version: Option<i32>,
    ) -> Result<Schema, tonic::Status> {
        let request = tonic::Request::new(GetSchemaRequest {
            event_type,
            version,
        });

        let response = self.inner.get_schema(request).await?;

        response
            .into_inner()
            .schema
            .ok_or_else(|| tonic::Status::not_found("Schema not found"))
    }

    /// イベントを検証
    pub async fn validate_event(
        &mut self,
        event_type: String,
        event_data: Vec<u8>,
        schema_version: Option<i32>,
    ) -> Result<bool, tonic::Status> {
        use prost_types::Any;

        let request = tonic::Request::new(ValidateEventRequest {
            event_type,
            event_data: Some(Any {
                type_url: String::new(),
                value:    event_data,
            }),
            schema_version,
        });

        let response = self.inner.validate_event(request).await?;
        Ok(response.into_inner().is_valid)
    }

    /// イベントタイプ一覧を取得
    pub async fn list_event_types(
        &mut self,
        context: Option<String>,
    ) -> Result<Vec<EventTypeInfo>, tonic::Status> {
        let request = tonic::Request::new(ListEventTypesRequest { context });

        let response = self.inner.list_event_types(request).await?;
        Ok(response.into_inner().event_types)
    }
}
