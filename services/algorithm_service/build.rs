//! Build script for Algorithm Service
//!
//! Compiles protobuf files for gRPC service definitions

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_algorithm_service()?;
    compile_learning_types()?;
    compile_event_store_client()?;
    compile_algorithm_events()?;
    setup_rerun_triggers();

    Ok(())
}

/// Algorithm Service のサーバー実装をコンパイル
fn compile_algorithm_service() -> Result<(), Box<dyn std::error::Error>> {
    let mut prost_config = ::prost_build::Config::new();
    prost_config.protoc_executable(protobuf_src::protoc());
    prost_config
        .file_descriptor_set_path(
            PathBuf::from(std::env::var("OUT_DIR")?).join("algorithm_descriptor.bin"),
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

    let builder = tonic_prost_build::configure()
        .build_server(true)
        .build_client(false);

    builder.compile_with_config(
        prost_config,
        &["../../protos/services/algorithm_service.proto"],
        &["../../protos"],
    )?;

    Ok(())
}

/// Learning Types の定義（CorrectnessJudgment を含む）をコンパイル
fn compile_learning_types() -> Result<(), Box<dyn std::error::Error>> {
    let mut prost_config = ::prost_build::Config::new();
    prost_config.protoc_executable(protobuf_src::protoc());
    prost_config
        .file_descriptor_set_path(
            PathBuf::from(std::env::var("OUT_DIR")?).join("learning_types_descriptor.bin"),
        )
        .extern_path(".effect.common", "crate::proto::effect::common");

    let builder = tonic_prost_build::configure()
        .build_server(false)
        .build_client(false);

    builder.compile_with_config(
        prost_config,
        &["../../protos/common/learning_types.proto"],
        &["../../protos"],
    )?;

    Ok(())
}

/// Event Store Service のクライアント実装をコンパイル
fn compile_event_store_client() -> Result<(), Box<dyn std::error::Error>> {
    let mut prost_config = ::prost_build::Config::new();
    prost_config.protoc_executable(protobuf_src::protoc());
    prost_config
        .file_descriptor_set_path(
            PathBuf::from(std::env::var("OUT_DIR")?).join("event_store_descriptor.bin"),
        )
        .extern_path(
            ".effect.common.EventMetadata",
            "crate::proto::effect::common::EventMetadata",
        )
        .extern_path(
            ".effect.common.TraceContext",
            "crate::proto::effect::common::TraceContext",
        );

    let builder = tonic_prost_build::configure()
        .build_server(false)
        .build_client(true);

    builder.compile_with_config(
        prost_config,
        &["../../protos/services/event_store_service.proto"],
        &["../../protos"],
    )?;

    Ok(())
}

/// Algorithm Events の定義をコンパイル
fn compile_algorithm_events() -> Result<(), Box<dyn std::error::Error>> {
    let mut prost_config = ::prost_build::Config::new();
    prost_config.protoc_executable(protobuf_src::protoc());
    prost_config
        .file_descriptor_set_path(
            PathBuf::from(std::env::var("OUT_DIR")?).join("algorithm_events_descriptor.bin"),
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

    let builder = tonic_prost_build::configure()
        .build_server(false)
        .build_client(false);

    builder.compile_with_config(
        prost_config,
        &["../../protos/events/algorithm_events.proto"],
        &["../../protos"],
    )?;

    Ok(())
}

/// ビルドが変更を検知できるようにする
fn setup_rerun_triggers() {
    println!("cargo:rerun-if-changed=../../protos/services/algorithm_service.proto");
    println!("cargo:rerun-if-changed=../../protos/services/event_store_service.proto");
    println!("cargo:rerun-if-changed=../../protos/common/learning_types.proto");
    println!("cargo:rerun-if-changed=../../protos/common/types.proto");
    println!("cargo:rerun-if-changed=../../protos/common/events.proto");
    println!("cargo:rerun-if-changed=../../protos/events/algorithm_events.proto");
}
