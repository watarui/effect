// Vocabulary Command Service Library
//
// CQRS + Event Sourcing の Write 側を担当
// ヘキサゴナルアーキテクチャを採用

pub mod config;
pub mod error;

// ドメイン層
pub mod domain {
    pub mod aggregates;
    pub mod commands;
    pub mod events;
    pub mod value_objects;

    // 再エクスポート
    pub use aggregates::*;
    pub use commands::*;
    pub use events::*;
    pub use value_objects::*;
}

// ポート層（インターフェース）
pub mod ports {
    pub mod event_store;
    pub mod repositories;

    pub use event_store::*;
    pub use repositories::*;
}

// アプリケーション層（ユースケース）
pub mod application {
    pub mod commands {
        pub mod create_vocabulary_item;
        pub mod update_vocabulary_item;

        #[cfg(test)]
        pub mod test_helpers;

        pub use create_vocabulary_item::CreateVocabularyItemHandler;
        pub use update_vocabulary_item::UpdateVocabularyItemHandler;
    }
}

// インフラストラクチャ層（技術的実装）
pub mod infrastructure {
    // TODO: 実装予定
    pub mod repositories {
        // PostgreSQL 実装
    }

    pub mod event_store {
        // Event Store 実装
    }

    pub mod grpc {
        // gRPC サーバー実装
    }
}
