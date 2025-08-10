//! gRPC サーバー実装

use std::{net::SocketAddr, sync::Arc};

use tonic::{Request, Response, Status, transport::Server};
use tracing::info;
use uuid::Uuid;

use crate::{config::Config, event_bus::EventBus, repository::PostgresEventStore};

// Protocol Buffers から生成されたコード
#[allow(clippy::all)]
#[allow(clippy::pedantic)]
#[allow(clippy::restriction)]
#[allow(warnings)]
pub mod proto {
    tonic::include_proto!("effect.event_store");
}

use proto::{
    event_store_service_server::{EventStoreService, EventStoreServiceServer},
    *,
};

/// Event Store Service の gRPC 実装
pub struct EventStoreServiceImpl {
    repository:           Arc<PostgresEventStore>,
    event_bus:            Arc<EventBus>,
    #[allow(dead_code)]
    domain_events_client: Option<DomainEventsClient>,
}

// Domain Events Service クライアント
struct DomainEventsClient {
    // TODO: 実際の gRPC クライアント実装
}

#[tonic::async_trait]
impl EventStoreService for EventStoreServiceImpl {
    async fn append_events(
        &self,
        request: Request<AppendEventsRequest>,
    ) -> Result<Response<AppendEventsResponse>, Status> {
        let req = request.into_inner();

        let stream_id = Uuid::parse_str(&req.stream_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid stream_id: {e}")))?;

        let events: Vec<serde_json::Value> = req
            .events
            .into_iter()
            .map(|e| {
                if let Some(any_data) = e.data {
                    // Any の value を JSON として扱う
                    serde_json::from_slice(&any_data.value).unwrap_or(serde_json::json!({}))
                } else {
                    serde_json::json!({})
                }
            })
            .collect();

        let expected_version = if req.expected_version < 0 {
            None
        } else {
            Some(req.expected_version)
        };

        let version = self
            .repository
            .append_events(
                stream_id,
                &req.stream_type,
                events.clone(),
                expected_version,
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to append events: {e}")))?;

        // Event Bus に発行
        for (i, event) in events.into_iter().enumerate() {
            let event_type = format!("{}.Event{}", req.stream_type, i); // TODO: 実際のイベントタイプ
            if let Err(e) = self
                .event_bus
                .publish_event(&event_type, &stream_id, event)
                .await
            {
                // エラーをログに記録して続行（At-least-once 保証）
                tracing::error!("Failed to publish event to Event Bus: {}", e);
            }
        }

        Ok(Response::new(AppendEventsResponse {
            next_version: version,
            event_ids:    vec![], // TODO: 実際の event_id を返す
        }))
    }

    async fn get_events(
        &self,
        request: Request<GetEventsRequest>,
    ) -> Result<Response<GetEventsResponse>, Status> {
        let req = request.into_inner();

        let stream_id = Uuid::parse_str(&req.stream_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid stream_id: {e}")))?;

        let to_version = if req.to_version < 0 {
            None
        } else {
            Some(req.to_version)
        };

        let events = self
            .repository
            .get_events(stream_id, &req.stream_type, req.from_version, to_version)
            .await
            .map_err(|e| Status::internal(format!("Failed to get events: {e}")))?;

        let proto_events = events
            .into_iter()
            .map(|e| {
                use std::collections::HashMap;

                use prost_types::Any;

                // JSON を Any に変換
                let data_bytes = e.data.to_string().into_bytes();
                let any_data = Any {
                    type_url: "type.googleapis.com/effect.event_store.Event".to_string(),
                    value:    data_bytes,
                };

                // metadata を HashMap に変換
                let mut metadata_map = HashMap::new();
                if let Some(obj) = e.metadata.as_object() {
                    for (k, v) in obj {
                        metadata_map.insert(k.clone(), v.to_string());
                    }
                }

                StoredEvent {
                    event_id:    e.event_id.to_string(),
                    stream_id:   e.stream_id.to_string(),
                    stream_type: e.stream_type,
                    version:     e.version,
                    event_type:  e.event_type,
                    data:        Some(any_data),
                    metadata:    metadata_map,
                    created_at:  Some(prost_types::Timestamp {
                        seconds: e.created_at.timestamp(),
                        nanos:   e.created_at.timestamp_subsec_nanos() as i32,
                    }),
                    position:    e.position.to_string(),
                }
            })
            .collect();

        Ok(Response::new(GetEventsResponse {
            events:           proto_events,
            next_version:     0,    // TODO: 次のバージョンを適切に設定
            is_end_of_stream: true, // TODO: ストリーム終端の判定
        }))
    }

    async fn get_snapshot(
        &self,
        request: Request<GetSnapshotRequest>,
    ) -> Result<Response<GetSnapshotResponse>, Status> {
        let req = request.into_inner();

        let stream_id = Uuid::parse_str(&req.stream_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid stream_id: {e}")))?;

        let max_version = if req.max_version < 0 {
            None
        } else {
            Some(req.max_version)
        };

        let snapshot = self
            .repository
            .get_snapshot(stream_id, &req.stream_type, max_version)
            .await
            .map_err(|e| Status::internal(format!("Failed to get snapshot: {e}")))?;

        Ok(Response::new(GetSnapshotResponse {
            snapshot: snapshot.as_ref().map(|s| {
                use prost_types::Any;

                // JSON を Any に変換
                let data_bytes = s.data.to_string().into_bytes();
                let any_data = Any {
                    type_url: "type.googleapis.com/effect.event_store.Snapshot".to_string(),
                    value:    data_bytes,
                };

                Snapshot {
                    snapshot_id: s.snapshot_id.to_string(),
                    stream_id:   s.stream_id.to_string(),
                    stream_type: s.stream_type.clone(),
                    version:     s.version,
                    data:        Some(any_data),
                    created_at:  Some(prost_types::Timestamp {
                        seconds: s.created_at.timestamp(),
                        nanos:   s.created_at.timestamp_subsec_nanos() as i32,
                    }),
                }
            }),
            found:    snapshot.is_some(),
        }))
    }

    async fn save_snapshot(
        &self,
        request: Request<SaveSnapshotRequest>,
    ) -> Result<Response<SaveSnapshotResponse>, Status> {
        let req = request.into_inner();

        let stream_id = Uuid::parse_str(&req.stream_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid stream_id: {e}")))?;

        let data = if let Some(any_data) = req.data {
            serde_json::from_slice(&any_data.value)
                .map_err(|e| Status::invalid_argument(format!("Invalid snapshot data: {e}")))?
        } else {
            serde_json::json!({})
        };

        self.repository
            .save_snapshot(stream_id, &req.stream_type, req.version, data)
            .await
            .map_err(|e| Status::internal(format!("Failed to save snapshot: {e}")))?;

        Ok(Response::new(SaveSnapshotResponse {
            snapshot_id: Uuid::new_v4().to_string(),
        }))
    }

    type SubscribeToStreamStream = tonic::codec::Streaming<EventNotification>;

    async fn subscribe_to_stream(
        &self,
        _request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeToStreamStream>, Status> {
        // TODO: 実装
        Err(Status::unimplemented("Not implemented"))
    }

    type SubscribeToAllStream = tonic::codec::Streaming<EventNotification>;

    async fn subscribe_to_all(
        &self,
        _request: Request<SubscribeAllRequest>,
    ) -> Result<Response<Self::SubscribeToAllStream>, Status> {
        // TODO: 実装
        Err(Status::unimplemented("Not implemented"))
    }
}

/// gRPC サーバーを起動
pub async fn start_server(
    config: Config,
    repository: PostgresEventStore,
    event_bus: EventBus,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse()?;

    // Domain Events Service クライアント作成（必要に応じて）
    let domain_events_client = if config.domain_events.enable_validation {
        // TODO: 実際の gRPC クライアント接続
        None
    } else {
        None
    };

    let service = EventStoreServiceImpl {
        repository: Arc::new(repository),
        event_bus: Arc::new(event_bus),
        domain_events_client,
    };

    info!("Event Store Service listening on {}", addr);

    Server::builder()
        .add_service(EventStoreServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
