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
        let query = format!(
            r#"
            INSERT INTO {} ({}, created_at, updated_at, version)
            VALUES ({}, $1, $2, $3)
            "#,
            $table,
            stringify!($($column),*),
            (1..=count_tts!($($column)*)).map(|i| format!("${}", i + 3)).collect::<Vec<_>>().join(", ")
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
        let new_version = current_version + 1;
        #[allow(clippy::cast_possible_wrap)]
        let current_version_i64 = current_version as i64;

        let set_clause = vec![
            $(
                format!("{} = ${}", stringify!($column), index_of!($column, $($column)*) + 1),
            )*
            format!("updated_at = ${}", count_tts!($($column)*) + 1),
            format!("version = ${}", count_tts!($($column)*) + 2),
        ].join(", ");

        let query = format!(
            r#"
            UPDATE {}
            SET {}
            WHERE {} = ${} AND version = ${} AND deleted_at IS NULL
            RETURNING version
            "#,
            $table,
            set_clause,
            $id_column,
            count_tts!($($column)*) + 3,
            count_tts!($($column)*) + 4
        );

        let result = sqlx::query_scalar::<_, i64>(&query)
            $(
                .bind(&$entity.$column)
            )*
            .bind(now)
            .bind(new_version)
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
                        let actual = v as u64;
                        Err(Error::optimistic_lock_failure(
                            current_version,
                            actual
                        ))
                    },
                    None => Err(Error::not_found(
                        std::any::type_name::<T>(),
                        $entity.id()
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
                $id.to_string(),
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
                    $id.to_string(),
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
                    $id.to_string(),
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
}
