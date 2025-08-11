//! Learning Algorithm Context
//!
//! 学習アルゴリズム（SM-2）と復習スケジューリングを担当するコンテキスト
//!
//! # Architecture
//!
//! このサービスはヘキサゴナルアーキテクチャに従って構成されています：
//! - `domain`: ビジネスロジックとドメインモデル
//! - `infrastructure`: 技術的実装（gRPC, Repository, イベント発行）
//! - `proto`: Protocol Buffers 定義

/// ドメイン層
pub mod domain {
    /// SM-2 アルゴリズムサービス
    pub mod services;
    /// 値オブジェクト
    pub mod value_objects;
}

/// インフラストラクチャ層
pub mod infrastructure;

/// Protocol Buffers 生成コード
pub mod proto;
