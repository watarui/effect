fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Proto ファイルのコンパイル
    // 現在は一時的に空にしておく（ビルドを通すため）
    // TODO: proto ファイルのコンパイル設定を追加

    // cargo に依存関係を通知
    println!("cargo:rerun-if-changed=../../protos/services/vocabulary_command_service.proto");
    println!("cargo:rerun-if-changed=../../protos/common/commands.proto");
    println!("cargo:rerun-if-changed=../../protos/common/types.proto");

    Ok(())
}
