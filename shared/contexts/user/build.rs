//! user context パッケージのビルドスクリプト
//!
//! User Context 固有のイベントを protobuf から生成します。

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../../protos".to_string();

    let mut config = tonic_prost_build::configure();

    // 生成コードの clippy 警告を抑制
    config = config
        .type_attribute(".", "#[allow(clippy::all)]")
        .type_attribute(".", "#[allow(dead_code)]")
        .type_attribute(".", "#[allow(missing_docs)]");

    // 外部クレートとして shared_kernel の型を使用
    config = config
        .extern_path(".effect.common", "::shared_kernel::proto::effect::common")
        .extern_path(".effect.services.user", "crate::proto");

    // サービスビルドを無効化（イベントのみ使用）
    config = config.build_server(false).build_client(false);

    // User イベント定義のみコンパイル
    config.compile_protos(
        &[&format!("{proto_root}/events/user_events.proto")],
        &[&proto_root],
    )?;

    // サービス定義を別設定でコンパイル
    let mut service_config = tonic_prost_build::configure();
    service_config = service_config
        .type_attribute(".", "#[allow(clippy::all)]")
        .type_attribute(".", "#[allow(dead_code)]")
        .type_attribute(".", "#[allow(missing_docs)]")
        .extern_path(".effect.common", "::shared_kernel::proto::effect::common")
        .build_server(false)
        .build_client(false);

    service_config.compile_protos(
        &[&format!("{proto_root}/services/user_service.proto")],
        &[&proto_root],
    )?;

    Ok(())
}
