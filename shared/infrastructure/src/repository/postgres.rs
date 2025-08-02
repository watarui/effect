//! `PostgreSQL` リポジトリの基底実裆
//!
//! 共通のデータベース操作をマクロとして提供

/// INSERT 文を生成するマクロ
///
/// タイムスタンプ（`created_at`, `updated_at`）を自動的に現在時刻に設定する
#[macro_export]
macro_rules! insert {
    (
        table: $table:expr,
        entity: $entity:expr,
        columns: [$($column:ident),* $(,)?],
        pool: $pool:expr $(,)?
    ) => {{
        use chrono::Utc;

        let now = Utc::now();

        // VALUESプレースホルダーを生成
        let mut placeholders = Vec::new();
        let mut idx = 1;
        $(
            placeholders.push(format!("${}", idx));
            idx += 1;
            let _ = stringify!($column); // 使用していることを示す
        )*
        let values_clause = placeholders.join(", ");

        let query = format!(
            r"
            INSERT INTO {} ({}, created_at, updated_at, version)
            VALUES ({}, ${}, ${}, ${})
            ",
            $table,
            stringify!($($column),*),
            values_clause,
            idx,
            idx + 1,
            idx + 2
        );

        sqlx::query(&query)
            $(
                .bind(&$entity.$column)
            )*
            .bind(now)
            .bind(now)
            .bind(1_i64) // 初期バージョンは1
            .execute($pool)
            .await
            .map(|_| ())
            .map_err($crate::repository::Error::from_sqlx)
    }};
}

/// UPDATE 文を生成するマクロ（楽観的ロック付き）
///
/// - version チェックを行い、不一致の場合はエラーを返す
/// - `updated_at` を現在時刻に更新
/// - version をインクリメント
#[macro_export]
macro_rules! update {
    (
        table: $table:expr,
        entity: $entity:expr,
        id_column: $id_column:expr,
        columns: [$($column:ident),* $(,)?],
        pool: $pool:expr $(,)?
    ) => {{
        use chrono::Utc;
        use $crate::repository::{Entity, Error};

        let now = Utc::now();
        let current_version = $entity.version();
        #[allow(clippy::cast_possible_wrap)]
        let current_version_i64 = current_version as i64;
        let new_version_i64 = current_version_i64 + 1;

        // カラムをセット句に変換
        let mut set_clauses = Vec::new();
        let mut idx = 1;
        $(
            set_clauses.push(format!("{} = ${}", stringify!($column), idx));
            idx += 1;
        )*
        set_clauses.push(format!("updated_at = ${}", idx));
        idx += 1;
        set_clauses.push(format!("version = ${}", idx));
        let set_clause = set_clauses.join(", ");

        let id_idx = idx;
        idx += 1;
        let version_idx = idx;

        let query = format!(
            r"
            UPDATE {}
            SET {}
            WHERE {} = ${} AND version = ${} AND deleted_at IS NULL
            RETURNING version
            ",
            $table,
            set_clause,
            $id_column,
            id_idx,
            version_idx
        );

        let result = sqlx::query_scalar::<_, i64>(&query)
            $(
                .bind(&$entity.$column)
            )*
            .bind(now)
            .bind(new_version_i64)
            .bind($entity.id())
            .bind(current_version_i64)
            .fetch_optional($pool)
            .await
            .map_err(Error::from_sqlx)?;

        match result {
            Some(_) => Ok(()),
            None => {
                // バージョン不一致または存在しない
                // 実際のバージョンを確認
                let actual_version: Option<i64> = sqlx::query_scalar(&format!(
                    "SELECT version FROM {} WHERE {} = $1 AND deleted_at IS NULL",
                    $table, $id_column
                ))
                .bind($entity.id())
                .fetch_optional($pool)
                .await
                .map_err(Error::from_sqlx)?;

                match actual_version {
                    Some(v) => {
                        #[allow(clippy::cast_sign_loss)]
                        #[allow(clippy::cast_sign_loss)]
                        let actual = v as u64;
                        Err(Error::optimistic_lock_failure(
                            current_version,
                            actual
                        ))
                    },
                    None => Err(Error::not_found(
                        $table,
                        &format!("{:?}", $entity.id())
                    )),
                }
            }
        }
    }};
}

/// SELECT 文を生成するマクロ（削除済みを除外）
#[macro_export]
macro_rules! select_by_id {
    (
        table:
        $table:expr,id_column:
        $id_column:expr,id:
        $id:expr,pool:
        $pool:expr,mapper:
        $mapper:expr $(,)?
    ) => {{
        let query = format!(
            "SELECT * FROM {} WHERE {} = $1 AND deleted_at IS NULL",
            $table, $id_column
        );

        sqlx::query(&query)
            .bind($id)
            .fetch_optional($pool)
            .await
            .map_err($crate::repository::Error::from_sqlx)?
            .map($mapper)
            .transpose()
            .map_err($crate::repository::Error::from_sqlx)
    }};
}

/// DELETE 文を生成するマクロ
#[macro_export]
macro_rules! delete {
    (table: $table:expr,id_column: $id_column:expr,id: $id:expr,pool: $pool:expr $(,)?) => {{
        let query = format!("DELETE FROM {} WHERE {} = $1", $table, $id_column);

        let result = sqlx::query(&query)
            .bind($id)
            .execute($pool)
            .await
            .map_err($crate::repository::Error::from_sqlx)?;

        if result.rows_affected() == 0 {
            Err($crate::repository::Error::not_found(
                $table,
                &$crate::hex::encode($id),
            ))
        } else {
            Ok(())
        }
    }};
}

/// EXISTS クエリを生成するマクロ（削除済みを除外）
#[macro_export]
macro_rules! exists {
    (table: $table:expr,id_column: $id_column:expr,id: $id:expr,pool: $pool:expr $(,)?) => {{
        let query = format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1 AND deleted_at IS NULL)",
            $table, $id_column
        );

        sqlx::query_scalar::<_, bool>(&query)
            .bind($id)
            .fetch_one($pool)
            .await
            .map_err($crate::repository::Error::from_sqlx)
    }};
}

/// ソフトデリート文を生成するマクロ
///
/// `deleted_at` フィールドに現在時刻を設定する
#[macro_export]
macro_rules! soft_delete {
    (table: $table:expr,id_column: $id_column:expr,id: $id:expr,pool: $pool:expr $(,)?) => {{
        use chrono::Utc;

        let now = Utc::now();
        let query = format!(
            "UPDATE {} SET deleted_at = $1, updated_at = $2 WHERE {} = $3 AND deleted_at IS NULL",
            $table, $id_column
        );

        let result = sqlx::query(&query)
            .bind(now)
            .bind(now)
            .bind($id)
            .execute($pool)
            .await
            .map_err($crate::repository::Error::from_sqlx)?;

        if result.rows_affected() == 0 {
            // エンティティが存在しないか、既に削除済み
            let exists_query = format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)",
                $table, $id_column
            );
            let exists = sqlx::query_scalar::<_, bool>(&exists_query)
                .bind($id)
                .fetch_one($pool)
                .await
                .map_err($crate::repository::Error::from_sqlx)?;

            if !exists {
                Err($crate::repository::Error::not_found(
                    $table,
                    &$crate::hex::encode($id),
                ))
            } else {
                // 既に削除済み
                Ok(())
            }
        } else {
            Ok(())
        }
    }};
}

/// ソフトデリートを復元するマクロ
///
/// `deleted_at` フィールドを NULL に設定する
#[macro_export]
macro_rules! restore {
    (table: $table:expr,id_column: $id_column:expr,id: $id:expr,pool: $pool:expr $(,)?) => {{
        use chrono::Utc;

        let now = Utc::now();
        let query = format!(
            "UPDATE {} SET deleted_at = NULL, updated_at = $1 WHERE {} = $2 AND deleted_at IS NOT \
             NULL",
            $table, $id_column
        );

        let result = sqlx::query(&query)
            .bind(now)
            .bind($id)
            .execute($pool)
            .await
            .map_err($crate::repository::Error::from_sqlx)?;

        if result.rows_affected() == 0 {
            // エンティティが存在しないか、削除されていない
            let exists_query = format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)",
                $table, $id_column
            );
            let exists = sqlx::query_scalar::<_, bool>(&exists_query)
                .bind($id)
                .fetch_one($pool)
                .await
                .map_err($crate::repository::Error::from_sqlx)?;

            if !exists {
                Err($crate::repository::Error::not_found(
                    $table,
                    &$crate::hex::encode($id),
                ))
            } else {
                // 削除されていない
                Ok(())
            }
        } else {
            Ok(())
        }
    }};
}

/// SELECT 文を生成するマクロ（削除済みを含む）
#[macro_export]
macro_rules! select_by_id_with_deleted {
    (
        table:
        $table:expr,id_column:
        $id_column:expr,id:
        $id:expr,pool:
        $pool:expr,mapper:
        $mapper:expr $(,)?
    ) => {{
        let query = format!("SELECT * FROM {} WHERE {} = $1", $table, $id_column);

        sqlx::query(&query)
            .bind($id)
            .fetch_optional($pool)
            .await
            .map_err($crate::repository::Error::from_sqlx)?
            .map($mapper)
            .transpose()
            .map_err($crate::repository::Error::from_sqlx)
    }};
}

/// 削除済みのエンティティのみを取得するマクロ
#[macro_export]
macro_rules! select_deleted {
    (table: $table:expr,pool: $pool:expr,mapper: $mapper:expr $(,)?) => {{
        let query = format!("SELECT * FROM {} WHERE deleted_at IS NOT NULL", $table);

        sqlx::query(&query)
            .fetch_all($pool)
            .await
            .map_err($crate::repository::Error::from_sqlx)?
            .into_iter()
            .map($mapper)
            .collect::<Result<Vec<_>, _>>()
            .map_err($crate::repository::Error::from_sqlx)
    }};
}

/// 複数の ID でエンティティを一括取得するマクロ（削除済みを除外）
#[macro_export]
macro_rules! select_by_ids {
    (
        table:
        $table:expr,id_column:
        $id_column:expr,ids:
        $ids:expr,pool:
        $pool:expr,mapper:
        $mapper:expr $(,)?
    ) => {{
        if $ids.is_empty() {
            Ok(vec![])
        } else {
            let placeholders = (1..=$ids.len())
                .map(|i| format!("${}", i))
                .collect::<Vec<_>>()
                .join(", ");

            let query = format!(
                "SELECT * FROM {} WHERE {} IN ({}) AND deleted_at IS NULL",
                $table, $id_column, placeholders
            );

            let mut query_builder = sqlx::query(&query);
            for id in $ids {
                query_builder = query_builder.bind(id);
            }

            query_builder
                .fetch_all($pool)
                .await
                .map_err($crate::repository::Error::from_sqlx)?
                .into_iter()
                .map($mapper)
                .collect::<Result<Vec<_>, _>>()
                .map_err($crate::repository::Error::from_sqlx)
        }
    }};
}

/// 全てのエンティティを取得するマクロ（削除済みを除外）
#[macro_export]
macro_rules! select_all {
    (table: $table:expr,pool: $pool:expr,mapper: $mapper:expr $(,)?) => {{
        let query = format!("SELECT * FROM {} WHERE deleted_at IS NULL", $table);

        sqlx::query(&query)
            .fetch_all($pool)
            .await
            .map_err($crate::repository::Error::from_sqlx)?
            .into_iter()
            .map($mapper)
            .collect::<Result<Vec<_>, _>>()
            .map_err($crate::repository::Error::from_sqlx)
    }};
}

/// エンティティ数を取得するマクロ（削除済みを除外）
#[macro_export]
macro_rules! count {
    (table: $table:expr,pool: $pool:expr $(,)?) => {{
        let query = format!("SELECT COUNT(*) FROM {} WHERE deleted_at IS NULL", $table);

        sqlx::query_scalar::<_, i64>(&query)
            .fetch_one($pool)
            .await
            .map_err($crate::repository::Error::from_sqlx)
    }};
}

// ヘルパーマクロ：トークンの数を数える
#[doc(hidden)]
#[macro_export]
macro_rules! count_tts {
    () => (0);
    ($head:tt $($tail:tt)*) => (1 + count_tts!($($tail)*));
}

// ヘルパーマクロ：トークンのインデックスを取得
#[doc(hidden)]
#[macro_export]
macro_rules! index_of {
    ($target:tt, $($elem:tt)*) => {
        index_of!(@acc 1, $target, $($elem)*)
    };
    (@acc $idx:expr, $target:tt, $head:tt $($tail:tt)*) => {
        if stringify!($target) == stringify!($head) {
            $idx
        } else {
            index_of!(@acc $idx + 1, $target, $($tail)*)
        }
    };
    (@acc $idx:expr, $target:tt,) => {
        panic!("Token not found")
    };
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use sqlx::{PgPool, postgres::PgPoolOptions};
    use uuid::Uuid;

    use crate::repository::{Entity, Error, Repository, SoftDeletable};

    // テスト用のモックエンティティ
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockEntity {
        id:         Uuid,
        name:       String,
        value:      i32,
        version:    u64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    }

    impl MockEntity {
        fn new(name: String, value: i32) -> Self {
            let now = Utc::now();
            Self {
                id: Uuid::new_v4(),
                name,
                value,
                version: 1,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            }
        }
    }

    impl Entity for MockEntity {
        type Id = Uuid;

        fn id(&self) -> &Self::Id {
            &self.id
        }

        fn version(&self) -> u64 {
            self.version
        }

        fn created_at(&self) -> DateTime<Utc> {
            self.created_at
        }

        fn updated_at(&self) -> DateTime<Utc> {
            self.updated_at
        }

        fn increment_version(&mut self) {
            self.version += 1;
        }

        fn touch(&mut self) {
            self.updated_at = Utc::now();
        }
    }

    impl crate::repository::entity::SoftDeletable for MockEntity {
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

    // テスト用のリポジトリ
    struct MockRepository {
        pool: PgPool,
    }

    impl MockRepository {
        fn new(pool: PgPool) -> Self {
            Self { pool }
        }
    }

    #[async_trait]
    impl Repository<MockEntity> for MockRepository {
        async fn save(&self, entity: &MockEntity) -> Result<(), Error> {
            let exists = exists!(
                table: "mock_entities",
                id_column: "id",
                id: entity.id().as_bytes(),
                pool: &self.pool
            )?;

            if exists {
                update!(
                    table: "mock_entities",
                    entity: entity,
                    id_column: "id",
                    columns: [name, value],
                    pool: &self.pool
                )
            } else {
                insert!(
                    table: "mock_entities",
                    entity: entity,
                    columns: [id, name, value],
                    pool: &self.pool
                )
            }
        }

        async fn find_by_id(&self, id: &Uuid) -> Result<Option<MockEntity>, Error> {
            select_by_id!(
                table: "mock_entities",
                id_column: "id",
                id: id.as_bytes(),
                pool: &self.pool,
                mapper: |row| map_row_to_mock(&row)
            )
        }

        async fn delete(&self, id: &Uuid) -> Result<(), Error> {
            delete!(
                table: "mock_entities",
                id_column: "id",
                id: id.as_bytes(),
                pool: &self.pool
            )
        }

        async fn exists(&self, id: &Uuid) -> Result<bool, Error> {
            exists!(
                table: "mock_entities",
                id_column: "id",
                id: id.as_bytes(),
                pool: &self.pool
            )
        }

        async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<MockEntity>, Error> {
            let id_bytes: Vec<&[u8]> = ids.iter().map(|id| id.as_bytes() as &[u8]).collect();
            select_by_ids!(
                table: "mock_entities",
                id_column: "id",
                ids: &id_bytes,
                pool: &self.pool,
                mapper: |row| map_row_to_mock(&row)
            )
        }

        async fn find_all(&self) -> Result<Vec<MockEntity>, Error> {
            select_all!(
                table: "mock_entities",
                pool: &self.pool,
                mapper: |row| map_row_to_mock(&row)
            )
        }

        async fn count(&self) -> Result<i64, Error> {
            count!(table: "mock_entities", pool: &self.pool)
        }
    }

    #[async_trait]
    impl SoftDeletable<MockEntity> for MockRepository {
        async fn soft_delete(&self, id: &Uuid) -> Result<(), Error> {
            soft_delete!(
                table: "mock_entities",
                id_column: "id",
                id: id.as_bytes(),
                pool: &self.pool
            )
        }

        async fn restore(&self, id: &Uuid) -> Result<(), Error> {
            restore!(
                table: "mock_entities",
                id_column: "id",
                id: id.as_bytes(),
                pool: &self.pool
            )
        }

        async fn find_by_id_with_deleted(&self, id: &Uuid) -> Result<Option<MockEntity>, Error> {
            select_by_id_with_deleted!(
                table: "mock_entities",
                id_column: "id",
                id: id.as_bytes(),
                pool: &self.pool,
                mapper: |row| map_row_to_mock(&row)
            )
        }

        async fn find_deleted(&self) -> Result<Vec<MockEntity>, Error> {
            select_deleted!(
                table: "mock_entities",
                pool: &self.pool,
                mapper: |row| map_row_to_mock(&row)
            )
        }
    }

    // Row から MockEntity への変換
    fn map_row_to_mock(row: &sqlx::postgres::PgRow) -> Result<MockEntity, sqlx::Error> {
        use sqlx::Row;

        let id_bytes: Vec<u8> = row.try_get("id")?;
        let id = Uuid::from_slice(&id_bytes).map_err(|e| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid UUID: {e}"),
            )))
        })?;

        Ok(MockEntity {
            id,
            name: row.try_get("name")?,
            value: row.try_get("value")?,
            version: {
                let v: i64 = row.try_get("version")?;
                #[allow(clippy::cast_sign_loss)]
                let version = v as u64;
                version
            },
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,
        })
    }

    // テスト用データベースのセットアップ
    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/effect_test".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap();

        // テスト用テーブルを作成
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS mock_entities (
                id BYTEA PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                value INTEGER NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                deleted_at TIMESTAMPTZ
            )
            ",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    // テスト後のクリーンアップ
    async fn cleanup_test_db(pool: &PgPool) {
        sqlx::query("DROP TABLE IF EXISTS mock_entities")
            .execute(pool)
            .await
            .unwrap();
    }

    #[test]
    fn test_count_tts() {
        assert_eq!(count_tts!(), 0);
        assert_eq!(count_tts!(a), 1);
        assert_eq!(count_tts!(a b c), 3);
    }

    #[test]
    fn test_index_of() {
        assert_eq!(index_of!(a, a b c), 1);
        assert_eq!(index_of!(b, a b c), 2);
        assert_eq!(index_of!(c, a b c), 3);
    }

    // 以下、マクロのテストケース
    #[tokio::test]
    async fn test_insert_macro() {
        let Ok(_) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("Skipping test: TEST_DATABASE_URL not set");
            return;
        };

        let pool = setup_test_db().await;
        let repo = MockRepository::new(pool.clone());
        let entity = MockEntity::new("test".to_string(), 42);

        // INSERT の実行
        let result = repo.save(&entity).await;
        assert!(result.is_ok());

        // 確認
        let found = repo.find_by_id(&entity.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, "test");
        assert_eq!(found.value, 42);
        assert_eq!(found.version, 1);

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_update_macro_with_optimistic_lock() {
        let Ok(_) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("Skipping test: TEST_DATABASE_URL not set");
            return;
        };

        let pool = setup_test_db().await;
        let repo = MockRepository::new(pool.clone());
        let mut entity = MockEntity::new("test".to_string(), 42);

        // 初回保存
        repo.save(&entity).await.unwrap();

        // 更新
        entity.name = "updated".to_string();
        entity.value = 100;
        let result = repo.save(&entity).await;
        assert!(result.is_ok());

        // 確認
        let found = repo.find_by_id(&entity.id).await.unwrap().unwrap();
        assert_eq!(found.name, "updated");
        assert_eq!(found.value, 100);
        assert_eq!(found.version, 2); // バージョンがインクリメントされている

        // 古いバージョンで更新しようとするとエラー
        entity.version = 1;
        let result = repo.save(&entity).await;
        assert!(matches!(result, Err(Error::OptimisticLockFailure { .. })));

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_soft_delete_and_restore() {
        let Ok(_) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("Skipping test: TEST_DATABASE_URL not set");
            return;
        };

        let pool = setup_test_db().await;
        let repo = MockRepository::new(pool.clone());
        let entity = MockEntity::new("test".to_string(), 42);

        // 保存
        repo.save(&entity).await.unwrap();

        // ソフトデリート
        repo.soft_delete(&entity.id).await.unwrap();

        // 通常の検索では見つからない
        let found = repo.find_by_id(&entity.id).await.unwrap();
        assert!(found.is_none());

        // 削除済みを含めた検索では見つかる
        let found = repo.find_by_id_with_deleted(&entity.id).await.unwrap();
        assert!(found.is_some());

        // リストア
        repo.restore(&entity.id).await.unwrap();

        // 再び通常の検索で見つかる
        let found = repo.find_by_id(&entity.id).await.unwrap();
        assert!(found.is_some());

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_select_by_ids_macro() {
        let Ok(_) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("Skipping test: TEST_DATABASE_URL not set");
            return;
        };

        let pool = setup_test_db().await;
        let repo = MockRepository::new(pool.clone());

        // 複数のエンティティを保存
        let entities: Vec<_> = (0..5)
            .map(|i| MockEntity::new(format!("test{i}"), i))
            .collect();

        for entity in &entities {
            repo.save(entity).await.unwrap();
        }

        // 一部のIDで検索
        let ids: Vec<_> = entities.iter().take(3).map(|e| e.id).collect();
        let found = repo.find_by_ids(&ids).await.unwrap();
        assert_eq!(found.len(), 3);

        // 空のIDリストで検索
        let found = repo.find_by_ids(&[]).await.unwrap();
        assert!(found.is_empty());

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_count_macro() {
        let Ok(_) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("Skipping test: TEST_DATABASE_URL not set");
            return;
        };

        let pool = setup_test_db().await;
        let repo = MockRepository::new(pool.clone());

        // 初期状態では0
        assert_eq!(repo.count().await.unwrap(), 0);

        // エンティティを追加
        for i in 0..3 {
            let entity = MockEntity::new(format!("test{i}"), i);
            repo.save(&entity).await.unwrap();
        }

        assert_eq!(repo.count().await.unwrap(), 3);

        // 1つソフトデリート
        let entity = MockEntity::new("to_delete".to_string(), 999);
        repo.save(&entity).await.unwrap();
        repo.soft_delete(&entity.id).await.unwrap();

        // カウントはソフトデリートされたものを除外
        assert_eq!(repo.count().await.unwrap(), 3);

        cleanup_test_db(&pool).await;
    }
}
