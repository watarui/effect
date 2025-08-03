//! ビルドスクリプト

fn main() {
    // proto ファイルのパスを設定
    let proto_root = "../../protos";

    // proto ファイルが変更されたら再ビルドするように設定
    println!("cargo:rerun-if-changed={proto_root}/services/vocabulary_service.proto");
    println!("cargo:rerun-if-changed={proto_root}/common/types.proto");
    println!("cargo:rerun-if-changed={proto_root}/common/learning_types.proto");
}
