//! ビルドスクリプト
//!
//! Protocol Buffers ファイルから Rust コードを生成

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Proto ファイルのコンパイル
    // 現在は一時的に空にしておく（ビルドを通すため）
    // TODO: proto ファイルのコンパイル設定を追加

    // cargo に依存関係を通知
    println!("cargo:rerun-if-changed=proto/vocabulary_search.proto");

    Ok(())
}
