//! user context パッケージのビルドスクリプト
//!
//! User Context 固有のイベントを protobuf から生成します。

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../../protos".to_string();

    // User イベント定義のコンパイル
    {
        // prost_build::Config を作成して protoc のパスを設定
        let mut prost_config = ::prost_build::Config::new();
        prost_config.protoc_executable(protobuf_src::protoc());

        // 生成コードの clippy 警告を抑制
        prost_config
            .type_attribute(".", "#[allow(clippy::all)]")
            .type_attribute(".", "#[allow(dead_code)]")
            .type_attribute(".", "#[allow(missing_docs)]")
            // 外部クレートとして shared_kernel の型を使用
            .extern_path(".effect.common", "::shared_kernel::proto::effect::common")
            .extern_path(".effect.services.user", "crate::proto");

        // tonic_prost_build の設定
        let builder = tonic_prost_build::configure()
            .build_server(false)
            .build_client(false);

        // compile_with_config を使用
        builder.compile_with_config(
            prost_config,
            &[&format!("{proto_root}/events/user_events.proto")],
            &[&proto_root],
        )?;
    }

    // サービス定義を別設定でコンパイル
    {
        // prost_build::Config を作成して protoc のパスを設定
        let mut prost_config = ::prost_build::Config::new();
        prost_config.protoc_executable(protobuf_src::protoc());

        // 生成コードの clippy 警告を抑制
        prost_config
            .type_attribute(".", "#[allow(clippy::all)]")
            .type_attribute(".", "#[allow(dead_code)]")
            .type_attribute(".", "#[allow(missing_docs)]")
            .extern_path(".effect.common", "::shared_kernel::proto::effect::common");

        // tonic_prost_build の設定
        let builder = tonic_prost_build::configure()
            .build_server(false)
            .build_client(false);

        // compile_with_config を使用
        builder.compile_with_config(
            prost_config,
            &[&format!("{proto_root}/services/user_service.proto")],
            &[&proto_root],
        )?;
    }

    Ok(())
}
