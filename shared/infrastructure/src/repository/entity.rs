//! エンティティの基底トレイト
//!
//! 全てのドメインエンティティが実装すべき共通インターフェース

use chrono::{DateTime, Utc};

/// エンティティの基底トレイト
///
/// データベースに永続化される全てのエンティティが実装すべきトレイト。
/// ID、タイムスタンプ、バージョンなどの共通フィールドを定義する。
pub trait Entity: Send + Sync {
    /// エンティティのID型
    type Id: Clone + Send + Sync + std::fmt::Debug + std::fmt::Display;

    /// エンティティのIDを取得
    fn id(&self) -> &Self::Id;

    /// バージョン番号を取得（楽観的ロック用）
    fn version(&self) -> u64;

    /// 作成日時を取得
    fn created_at(&self) -> DateTime<Utc>;

    /// 更新日時を取得
    fn updated_at(&self) -> DateTime<Utc>;

    /// バージョンをインクリメント（更新時に使用）
    fn increment_version(&mut self);

    /// 更新日時を現在時刻に設定
    fn touch(&mut self);
}

/// ソフトデリート可能なエンティティのトレイト
///
/// 論理削除をサポートするエンティティが実装する追加のトレイト
pub trait SoftDeletable: Entity {
    /// 削除日時を取得
    fn deleted_at(&self) -> Option<DateTime<Utc>>;

    /// 削除済みかどうかを判定
    #[must_use]
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }

    /// ソフトデリートを実行
    fn soft_delete(&mut self);

    /// ソフトデリートを取り消し
    fn restore(&mut self);
}

/// タイムスタンプ付きの構造体
///
/// `created_at` と `updated_at` を持つ構造体の共通実装
#[derive(Debug, Clone)]
pub struct Timestamped {
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
}

impl Timestamped {
    /// 新しいタイムスタンプを作成
    #[must_use]
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
        }
    }

    /// 指定した時刻でタイムスタンプを作成
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_time(time: DateTime<Utc>) -> Self {
        Self {
            created_at: time,
            updated_at: time,
        }
    }

    /// 更新日時を現在時刻に更新
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Default for Timestamped {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    // テスト用のエンティティ
    #[derive(Debug)]
    struct TestEntity {
        id:         String,
        version:    u64,
        timestamps: Timestamped,
        deleted_at: Option<DateTime<Utc>>,
    }

    impl Entity for TestEntity {
        type Id = String;

        fn id(&self) -> &Self::Id {
            &self.id
        }

        fn version(&self) -> u64 {
            self.version
        }

        fn created_at(&self) -> DateTime<Utc> {
            self.timestamps.created_at
        }

        fn updated_at(&self) -> DateTime<Utc> {
            self.timestamps.updated_at
        }

        fn increment_version(&mut self) {
            self.version += 1;
        }

        fn touch(&mut self) {
            self.timestamps.touch();
        }
    }

    impl SoftDeletable for TestEntity {
        fn deleted_at(&self) -> Option<DateTime<Utc>> {
            self.deleted_at
        }

        fn soft_delete(&mut self) {
            self.deleted_at = Some(Utc::now());
            self.touch();
        }

        fn restore(&mut self) {
            self.deleted_at = None;
            self.touch();
        }
    }

    #[test]
    fn test_entity_trait() {
        let entity = TestEntity {
            id:         "test-id".to_string(),
            version:    1,
            timestamps: Timestamped::new(),
            deleted_at: None,
        };

        assert_eq!(entity.id(), "test-id");
        assert_eq!(entity.version(), 1);
        assert!(!entity.is_deleted());
    }

    #[test]
    fn test_increment_version() {
        let mut entity = TestEntity {
            id:         "test-id".to_string(),
            version:    1,
            timestamps: Timestamped::new(),
            deleted_at: None,
        };

        entity.increment_version();
        assert_eq!(entity.version(), 2);
    }

    #[test]
    fn test_touch_updates_timestamp() {
        let mut entity = TestEntity {
            id:         "test-id".to_string(),
            version:    1,
            timestamps: Timestamped::new(),
            deleted_at: None,
        };

        let original_updated_at = entity.updated_at();

        // 時間差を作るため少し待機
        thread::sleep(Duration::from_millis(10));

        entity.touch();

        assert!(entity.updated_at() > original_updated_at);
        assert_eq!(entity.created_at(), original_updated_at); // created_at は変わらない
    }

    #[test]
    fn test_soft_delete() {
        let mut entity = TestEntity {
            id:         "test-id".to_string(),
            version:    1,
            timestamps: Timestamped::new(),
            deleted_at: None,
        };

        assert!(!entity.is_deleted());

        entity.soft_delete();
        assert!(entity.is_deleted());
        assert!(entity.deleted_at().is_some());

        entity.restore();
        assert!(!entity.is_deleted());
        assert!(entity.deleted_at().is_none());
    }
}
