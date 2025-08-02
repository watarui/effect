//! リポジトリの基底トレイト
//!
//! 全てのリポジトリが実装すべき共通インターフェースを定義

use async_trait::async_trait;

use super::{Entity, Result};

/// リポジトリの基底トレイト
///
/// データの永続化層へのアクセスを抽象化する共通インターフェース
#[async_trait]
pub trait Repository<T: Entity>: Send + Sync {
    /// エンティティを保存（作成または更新）
    ///
    /// - 新規エンティティの場合: INSERT を実行
    /// - 既存エンティティの場合: UPDATE を実行（楽観的ロックチェック付き）
    /// - タイムスタンプ（`created_at`/`updated_at`）は自動管理
    ///
    /// # Errors
    ///
    /// - `UniqueViolation`: 一意制約違反
    /// - `OptimisticLockFailure`: 楽観的ロック失敗
    /// - `Database`: その他のデータベースエラー
    async fn save(&self, entity: &T) -> Result<()>;

    /// ID でエンティティを検索
    ///
    /// # Returns
    ///
    /// - `Some(entity)`: エンティティが見つかった場合
    /// - `None`: エンティティが見つからなかった場合
    ///
    /// # Errors
    ///
    /// - `Database`: データベースエラー
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>>;

    /// ID でエンティティを削除
    ///
    /// 物理削除を実行する。ソフトデリートが必要な場合は
    /// `SoftDeletableRepository` トレイトを使用する。
    ///
    /// # Errors
    ///
    /// - `NotFound`: エンティティが見つからない
    /// - `Database`: データベースエラー
    async fn delete(&self, id: &T::Id) -> Result<()>;

    /// ID でエンティティの存在を確認
    ///
    /// # Errors
    ///
    /// - `Database`: データベースエラー
    async fn exists(&self, id: &T::Id) -> Result<bool>;

    /// 複数のIDでエンティティを一括取得
    ///
    /// 存在しないIDは結果に含まれない。
    ///
    /// # Errors
    ///
    /// - `Database`: データベースエラー
    async fn find_by_ids(&self, ids: &[T::Id]) -> Result<Vec<T>>;

    /// 全てのエンティティを取得
    ///
    /// **注意**: 大量のデータがある場合はページネーションを使用すること
    ///
    /// # Errors
    ///
    /// - `Database`: データベースエラー
    async fn find_all(&self) -> Result<Vec<T>>;

    /// エンティティ数を取得
    ///
    /// # Errors
    ///
    /// - `Database`: データベースエラー
    async fn count(&self) -> Result<i64>;
}

/// ソフトデリート可能なリポジトリのトレイト
///
/// 論理削除をサポートするリポジトリが実装する追加のトレイト
#[async_trait]
pub trait SoftDeletableRepository<T: Entity>: Repository<T> {
    /// ソフトデリート（論理削除）を実行
    ///
    /// `deleted_at` フィールドに現在時刻を設定する。
    ///
    /// # Errors
    ///
    /// - `NotFound`: エンティティが見つからない
    /// - `Database`: データベースエラー
    async fn soft_delete(&self, id: &T::Id) -> Result<()>;

    /// ソフトデリートを取り消し
    ///
    /// `deleted_at` フィールドを `NULL` に設定する。
    ///
    /// # Errors
    ///
    /// - `NotFound`: エンティティが見つからない
    /// - `Database`: データベースエラー
    async fn restore(&self, id: &T::Id) -> Result<()>;

    /// 削除済みを含めて ID でエンティティを検索
    ///
    /// # Errors
    ///
    /// - `Database`: データベースエラー
    async fn find_by_id_with_deleted(&self, id: &T::Id) -> Result<Option<T>>;

    /// 削除済みのエンティティのみを取得
    ///
    /// # Errors
    ///
    /// - `Database`: データベースエラー
    async fn find_deleted(&self) -> Result<Vec<T>>;
}

/// ページネーション情報
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    /// ページ番号（1始まり）
    pub page:     u32,
    /// 1ページあたりの件数
    pub per_page: u32,
}

impl Pagination {
    /// 新しいページネーション情報を作成
    #[must_use]
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page:     page.max(1),
            per_page: per_page.clamp(1, 100),
        }
    }

    /// OFFSET 値を計算
    #[must_use]
    #[allow(clippy::cast_lossless)]
    pub const fn offset(self) -> i64 {
        ((self.page - 1) * self.per_page) as i64
    }

    /// LIMIT 値を取得
    #[must_use]
    #[allow(clippy::cast_lossless)]
    pub const fn limit(self) -> i64 {
        self.per_page as i64
    }
}

/// ページネーション結果
#[derive(Debug, Clone)]
pub struct Page<T> {
    /// 現在のページのアイテム
    pub items:       Vec<T>,
    /// 総アイテム数
    pub total_count: i64,
    /// 現在のページ番号
    pub current:     u32,
    /// 1ページあたりの件数
    pub size:        u32,
}

impl<T> Page<T> {
    /// 新しいページネーション結果を作成
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(items: Vec<T>, total_count: i64, pagination: Pagination) -> Self {
        Self {
            items,
            total_count,
            current: pagination.page,
            size: pagination.per_page,
        }
    }

    /// 総ページ数を計算
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::manual_div_ceil
    )]
    pub const fn total_pages(&self) -> u32 {
        // 整数演算で切り上げ除算を実現
        let total = self.total_count as u32;
        // div_ceil は const fn では使えないため手動実装
        (total + self.size - 1) / self.size
    }

    /// 次のページが存在するか
    #[must_use]
    pub const fn has_next_page(&self) -> bool {
        self.current < self.total_pages()
    }

    /// 前のページが存在するか
    #[must_use]
    pub const fn has_prev_page(&self) -> bool {
        self.current > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(1, 20);
        assert_eq!(pagination.offset(), 0);
        assert_eq!(pagination.limit(), 20);

        let pagination = Pagination::new(3, 20);
        assert_eq!(pagination.offset(), 40);
        assert_eq!(pagination.limit(), 20);
    }

    #[test]
    fn test_pagination_bounds() {
        // ページ番号は最小1
        let pagination = Pagination::new(0, 20);
        assert_eq!(pagination.page, 1);

        // per_page は最大100
        let pagination = Pagination::new(1, 200);
        assert_eq!(pagination.per_page, 100);
    }

    #[test]
    fn test_page() {
        let items = vec![1, 2, 3, 4, 5];
        let pagination = Pagination::new(2, 5);
        let page = Page::new(items, 23, pagination);

        assert_eq!(page.total_pages(), 5);
        assert!(page.has_prev_page());
        assert!(page.has_next_page());

        // 最初のページ
        let first_page = Page::new(vec![1, 2, 3], 10, Pagination::new(1, 3));
        assert!(!first_page.has_prev_page());
        assert!(first_page.has_next_page());

        // 最後のページ
        let last_page = Page::new(vec![1], 10, Pagination::new(4, 3));
        assert!(last_page.has_prev_page());
        assert!(!last_page.has_next_page());
    }
}
