//! AI Integration Context
//!
//! AI サービスとの統合を担当するコンテキスト

// 一時的に警告を抑制
#![allow(missing_docs)]
#![allow(unused)]

pub mod domain {
    //! ドメイン層

    pub mod aggregates {
        //! 集約
    }

    pub mod events {
        //! ドメインイベント
    }

    pub mod commands {
        //! コマンド
    }

    pub mod value_objects {
        //! 値オブジェクト
    }

    pub mod services {
        //! ドメインサービス（Anti-Corruption Layer）
    }
}

pub mod application {
    //! アプリケーション層

    pub mod command_handlers {
        //! コマンドハンドラー
    }

    pub mod query_handlers {
        //! クエリハンドラー
    }

    pub mod services {
        //! アプリケーションサービス
    }
}

pub mod infrastructure {
    //! インフラストラクチャ層

    pub mod repositories {
        //! リポジトリ実装
    }

    pub mod ai_providers {
        //! AI プロバイダー実装
    }
}

pub mod ports {
    //! ポート定義

    pub mod inbound {
        //! インバウンドポート
    }

    pub mod outbound {
        //! アウトバウンドポート
    }
}
