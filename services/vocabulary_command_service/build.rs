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

    config
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../../protos/services/vocabulary_command_service.proto"],
            &["../../protos"],
        )?;

    // ビルドが変更を検知できるようにする
    println!("cargo:rerun-if-changed=../../protos/services/vocabulary_command_service.proto");
    println!("cargo:rerun-if-changed=../../protos/common/commands.proto");
    println!("cargo:rerun-if-changed=../../protos/common/types.proto");

    Ok(())
}
