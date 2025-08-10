//! Domain Events Service のビルドスクリプト

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = tonic_prost_build::configure();

    // 生成コードの clippy 警告を抑制
    config = config
        .type_attribute(".", "#[allow(clippy::all)]")
        .type_attribute(".", "#[allow(dead_code)]")
        .type_attribute(".", "#[allow(missing_docs)]")
        // サービス生成コードにも clippy 警告を抑制
        .server_mod_attribute(".", "#[allow(clippy::all)]")
        .client_mod_attribute(".", "#[allow(clippy::all)]")
        .server_attribute(".", "#[allow(clippy::all)]")
        .client_attribute(".", "#[allow(clippy::all)]");

    // Domain Events Service の gRPC 定義をコンパイル
    config
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../../protos/services/domain_events_service.proto"],
            &["../../protos"],
        )?;

    // ビルドが変更を検知できるようにする
    println!("cargo:rerun-if-changed=../../protos/services/domain_events_service.proto");

    Ok(())
}
