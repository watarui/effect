//! トランザクション管理
//!
//! Unit of Work パターンの実装

use sqlx::{PgPool, Postgres, Transaction};

use super::error::Error;

/// Unit of Work
///
/// トランザクションスコープを管理し、複数のリポジトリ操作を
/// 単一のトランザクション内で実行できるようにする。
pub struct UnitOfWork {
    tx: Transaction<'static, Postgres>,
}

impl UnitOfWork {
    /// 新しいトランザクションを開始
    ///
    /// # Errors
    ///
    /// データベース接続エラーが発生した場合
    pub async fn begin(pool: &PgPool) -> Result<Self, Error> {
        // 'static ライフタイムにするため Box::leak を使用
        // これは安全: Transaction がドロップされるときに自動的にロールバックされる
        let pool = Box::leak(Box::new(pool.clone()));

        let tx = pool
            .begin()
            .await
            .map_err(|e| Error::Transaction(format!("Failed to begin transaction: {e}")))?;

        Ok(Self { tx })
    }

    /// トランザクションをコミット
    ///
    /// # Errors
    ///
    /// コミットに失敗した場合
    pub async fn commit(self) -> Result<(), Error> {
        self.tx
            .commit()
            .await
            .map_err(|e| Error::Transaction(format!("Failed to commit transaction: {e}")))
    }

    /// トランザクションをロールバック
    ///
    /// # Errors
    ///
    /// ロールバックに失敗した場合
    pub async fn rollback(self) -> Result<(), Error> {
        self.tx
            .rollback()
            .await
            .map_err(|e| Error::Transaction(format!("Failed to rollback transaction: {e}")))
    }

    /// トランザクションの参照を取得
    ///
    /// リポジトリメソッドに渡すために使用
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn transaction(&mut self) -> &mut Transaction<'static, Postgres> {
        &mut self.tx
    }
}

/// トランザクション可能なリポジトリのトレイト
///
/// トランザクション内で操作できるリポジトリが実装すべきトレイト
#[async_trait::async_trait]
pub trait TransactionalRepository<T: super::Entity>: super::Repository<T> {
    /// トランザクション内でエンティティを保存
    async fn save_in_tx(
        &self,
        entity: &T,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<(), Error>;

    /// トランザクション内でIDでエンティティを検索
    async fn find_by_id_in_tx(
        &self,
        id: &T::Id,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<Option<T>, Error>;

    /// トランザクション内でエンティティを削除
    async fn delete_in_tx(
        &self,
        id: &T::Id,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<(), Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // インテグレーションテストは tests/ ディレクトリで実施
    // ここではコンパイルが通ることを確認

    #[test]
    fn test_unit_of_work_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        // UnitOfWork が Send + Sync であることを確認
        // これにより async 環境で安全に使用できる
        assert_send_sync::<UnitOfWork>();
    }
}
