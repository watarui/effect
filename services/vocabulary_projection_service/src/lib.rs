//! Vocabulary Projection Service
//!
//! Event Store からイベントを購読し、Read Model を構築するサービス
//! ヘキサゴナルアーキテクチャを採用

pub mod config;
pub mod error;

// ドメイン層
pub mod domain {
    pub mod events;
    pub mod projections;
}

// ポート層（インターフェース）
pub mod ports {
    pub mod inbound;
    pub mod outbound;
}

// アプリケーション層（ユースケース）
pub mod application {
    pub mod event_handlers;
    pub mod processor;
}

// インフラストラクチャ層（技術的実装）
pub mod infrastructure {
    pub mod repositories {
        pub mod postgres_projection_state;
        pub mod postgres_read_model;
    }

    pub mod adapters {
        pub mod event_store_subscriber;
    }
}
