//! user-service のビルドスクリプト

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../protos".to_string();

    let mut config = tonic_prost_build::configure();

    // google.protobuf.Timestamp を除いて serde を適用
    config = config.type_attribute("effect", "#[derive(serde::Serialize, serde::Deserialize)]");

    // 生成コードの clippy 警告を抑制
    config = config.type_attribute(".", "#[allow(clippy::all)]");

    config.compile_protos(
        &[
            &format!("{proto_root}/common/types.proto"),
            &format!("{proto_root}/services/user_service.proto"),
            &format!("{proto_root}/events/user_events.proto"),
        ],
        &[&proto_root],
    )?;

    Ok(())
}
