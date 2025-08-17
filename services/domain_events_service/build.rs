//! Domain Events Service のビルドスクリプト

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // prost_build::Config を作成して protoc のパスを設定
    let mut prost_config = ::prost_build::Config::new();
    prost_config.protoc_executable(protobuf_src::protoc());

    // 生成コードの clippy 警告を抑制
    prost_config
        .type_attribute(".", "#[allow(clippy::all)]")
        .type_attribute(".", "#[allow(dead_code)]")
        .type_attribute(".", "#[allow(missing_docs)]");

    // tonic_prost_build の設定
    let builder = tonic_prost_build::configure()
        // サービス生成コードにも clippy 警告を抑制
        .server_mod_attribute(".", "#[allow(clippy::all)]")
        .client_mod_attribute(".", "#[allow(clippy::all)]")
        .server_attribute(".", "#[allow(clippy::all)]")
        .client_attribute(".", "#[allow(clippy::all)]")
        .build_server(true)
        .build_client(true);

    // compile_with_config を使用
    builder.compile_with_config(
        prost_config,
        &["../../protos/services/domain_events_service.proto"],
        &["../../protos"],
    )?;

    // ビルドが変更を検知できるようにする
    println!("cargo:rerun-if-changed=../../protos/services/domain_events_service.proto");

    Ok(())
}
