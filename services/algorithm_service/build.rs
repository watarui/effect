//! Build script for Algorithm Service
//!
//! Compiles protobuf files for gRPC service definitions

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Algorithm Service のサーバー実装（common は shared_kernel から使用、ただし
    // CorrectnessJudgment は追加）
    let config = tonic_prost_build::configure()
        .file_descriptor_set_path(
            std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("algorithm_descriptor.bin"),
        )
        .extern_path(
            ".effect.common.EventMetadata",
            "crate::proto::effect::common::EventMetadata",
        )
        .extern_path(
            ".effect.common.TraceContext",
            "crate::proto::effect::common::TraceContext",
        )
        .extern_path(
            ".effect.common.CorrectnessJudgment",
            "crate::proto::effect::learning::CorrectnessJudgment",
        );

    config
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../protos/services/algorithm_service.proto"],
            &["../../protos"],
        )?;

    // Learning Types の定義（CorrectnessJudgment を含む）
    let learning_config = tonic_prost_build::configure()
        .file_descriptor_set_path(
            std::path::PathBuf::from(std::env::var("OUT_DIR")?)
                .join("learning_types_descriptor.bin"),
        )
        .extern_path(".effect.common", "crate::proto::effect::common");

    learning_config
        .build_server(false)
        .build_client(false)
        .compile_protos(
            &["../../protos/common/learning_types.proto"],
            &["../../protos"],
        )?;

    // Event Store Service のクライアント実装
    let event_config = tonic_prost_build::configure()
        .file_descriptor_set_path(
            std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("event_store_descriptor.bin"),
        )
        .extern_path(
            ".effect.common.EventMetadata",
            "crate::proto::effect::common::EventMetadata",
        )
        .extern_path(
            ".effect.common.TraceContext",
            "crate::proto::effect::common::TraceContext",
        );

    event_config
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &["../../protos/services/event_store_service.proto"],
            &["../../protos"],
        )?;

    // Algorithm Events の定義
    let event_proto_config = tonic_prost_build::configure()
        .file_descriptor_set_path(
            std::path::PathBuf::from(std::env::var("OUT_DIR")?)
                .join("algorithm_events_descriptor.bin"),
        )
        .extern_path(
            ".effect.common.EventMetadata",
            "crate::proto::effect::common::EventMetadata",
        )
        .extern_path(
            ".effect.common.TraceContext",
            "crate::proto::effect::common::TraceContext",
        )
        .extern_path(
            ".effect.common.CorrectnessJudgment",
            "crate::proto::effect::learning::CorrectnessJudgment",
        );

    event_proto_config
        .build_server(false)
        .build_client(false)
        .compile_protos(
            &["../../protos/events/algorithm_events.proto"],
            &["../../protos"],
        )?;

    // ビルドが変更を検知できるようにする
    println!("cargo:rerun-if-changed=../../protos/services/algorithm_service.proto");
    println!("cargo:rerun-if-changed=../../protos/services/event_store_service.proto");
    println!("cargo:rerun-if-changed=../../protos/common/learning_types.proto");
    println!("cargo:rerun-if-changed=../../protos/common/types.proto");
    println!("cargo:rerun-if-changed=../../protos/common/events.proto");
    println!("cargo:rerun-if-changed=../../protos/events/algorithm_events.proto");

    Ok(())
}
