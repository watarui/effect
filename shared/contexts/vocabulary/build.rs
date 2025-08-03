//! vocabulary context パッケージのビルドスクリプト
//!
//! Vocabulary Context 固有のイベントを protobuf から生成します。

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../../protos".to_string();

    let mut config = tonic_prost_build::configure();

    // 生成コードの clippy 警告を抑制
    config = config
        .type_attribute(".", "#[allow(clippy::all)]")
        .type_attribute(".", "#[allow(dead_code)]")
        .type_attribute(".", "#[allow(missing_docs)]");

    // 外部クレートとして shared_kernel の型を使用
    config = config.extern_path(".effect.common", "::shared_kernel::proto::effect::common");

    // サービスビルドを無効化（イベントのみ使用）
    config = config.build_server(false).build_client(false);

    config.compile_protos(
        &[
            // Vocabulary イベント定義
            &format!("{proto_root}/events/vocabulary_events.proto"),
        ],
        &[&proto_root],
    )?;

    Ok(())
}
