//! user-service のビルドスクリプト

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .emit_rerun_if_changed(false)
        .compile_protos(&["proto/user_service.proto"], &["proto"])?;
    Ok(())
}
