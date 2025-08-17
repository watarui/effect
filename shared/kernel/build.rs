//! shared_kernel パッケージのビルドスクリプト
//!
//! 共通の型定義（EventMetadata など）を protobuf から生成します。

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../protos".to_string();

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
        .build_server(false)
        .build_client(false);

    // compile_with_config を使用
    builder.compile_with_config(
        prost_config,
        &[
            // 共通型定義
            &format!("{proto_root}/common/types.proto"),
            &format!("{proto_root}/common/commands.proto"),
            &format!("{proto_root}/common/events.proto"),
        ],
        &[&proto_root],
    )?;

    Ok(())
}
