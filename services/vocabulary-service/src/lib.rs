//! Vocabulary Context
//!
//! 語彙エントリーの管理を担当するコンテキスト

pub mod domain {
    //! ドメイン層

    pub mod entities {
        //! エンティティ
        pub mod vocabulary_entry;
        pub mod vocabulary_item;
    }

    pub mod value_objects {
        //! 値オブジェクト
        pub mod cefr_level;
        pub mod definition;
        pub mod domain;
        pub mod part_of_speech;
        pub mod register;
    }

    pub mod events {
        //! ドメインイベント
    }

    pub mod commands {
        //! コマンド
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

pub mod adapters {
    //! アダプター層

    pub mod outbound {
        //! アウトバウンドアダプター

        pub mod repository {
            //! リポジトリ実装
            pub mod postgres;
        }
    }
}

pub mod ports {
    //! ポート定義

    pub mod inbound {
        //! インバウンドポート
    }

    pub mod outbound {
        //! アウトバウンドポート
        pub mod repository;
    }
}
