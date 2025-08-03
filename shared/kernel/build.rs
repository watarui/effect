//! shared_kernel パッケージのビルドスクリプト
//!
//! 共通の型定義（EventMetadata など）を protobuf から生成します。

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../protos".to_string();

    let mut config = tonic_prost_build::configure();

    // 生成コードの clippy 警告を抑制
    config = config
        .type_attribute(".", "#[allow(clippy::all)]")
        .type_attribute(".", "#[allow(dead_code)]")
        .type_attribute(".", "#[allow(missing_docs)]");

    // サービスビルドを無効化（共通型のみ使用）
    config = config.build_server(false).build_client(false);

    config.compile_protos(
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
