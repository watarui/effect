//! Build script for Algorithm Service
//!
//! Compiles protobuf files for gRPC service definitions

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = tonic_prost_build::configure().file_descriptor_set_path(
        std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("algorithm_descriptor.bin"),
    );

    config
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../protos/services/algorithm_service.proto"],
            &["../../protos"],
        )?;

    // ビルドが変更を検知できるようにする
    println!("cargo:rerun-if-changed=../../protos/services/algorithm_service.proto");
    println!("cargo:rerun-if-changed=../../protos/common/learning_types.proto");
    println!("cargo:rerun-if-changed=../../protos/common/types.proto");
    println!("cargo:rerun-if-changed=../../protos/events/algorithm_events.proto");

    Ok(())
}
