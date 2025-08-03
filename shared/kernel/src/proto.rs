//! Proto 生成コードを含むモジュール

// Proto 生成コードを含める
#[allow(warnings)]
#[allow(missing_docs)]
pub mod effect {
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/effect.common.rs"));
    }
}

// 共通型を再エクスポート
pub use effect::common::{EventMetadata as ProtoEventMetadata, TraceContext as ProtoTraceContext};
