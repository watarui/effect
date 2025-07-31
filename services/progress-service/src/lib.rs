//! Progress Context
//!
//! 学習進捗の追跡と分析を担当するコンテキスト（CQRS/イベントソーシング）

// 一時的に警告を抑制
#![allow(missing_docs)]
#![allow(unused)]

pub mod application {
    //! アプリケーション層

    pub mod query_handlers {
        //! クエリハンドラー
    }

    pub mod projections {
        //! プロジェクション
    }
}

pub mod infrastructure {
    //! インフラストラクチャ層

    pub mod event_store {
        //! イベントストア実装
    }

    pub mod projections {
        //! プロジェクション実装
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
