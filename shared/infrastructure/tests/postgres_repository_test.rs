//! `PostgreSQL` リポジトリのインテグレーションテスト
#![cfg(test)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use infrastructure::{
    Entity,
    Repository,
    RepositoryError as Error,
    SoftDeletable,
    count,
    delete,
    exists,
    insert,
    restore,
    select_all,
    select_by_id,
    select_by_id_with_deleted,
    select_by_ids,
    select_deleted,
    soft_delete,
    update,
};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use uuid::Uuid;

// テスト用のドメインエンティティ
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Product {
    id:         Uuid,
    name:       String,
    price:      i32,
    stock:      i32,
    version:    u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl Product {
    fn new(name: String, price: i32, stock: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            price,
            stock,
            version: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

impl Entity for Product {
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

impl infrastructure::repository::entity::SoftDeletable for Product {
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
struct ProductRepository {
    pool: PgPool,
}

impl ProductRepository {
    const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Product> for ProductRepository {
    async fn save(&self, entity: &Product) -> Result<(), Error> {
        let exists = exists!(
            table: "products",
            id_column: "id",
            id: entity.id().as_bytes(),
            pool: &self.pool
        )?;

        if exists {
            update!(
                table: "products",
                entity: entity,
                id_column: "id",
                columns: [name, price, stock],
                pool: &self.pool
            )
        } else {
            insert!(
                table: "products",
                entity: entity,
                columns: [id, name, price, stock],
                pool: &self.pool
            )
        }
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Product>, Error> {
        select_by_id!(
            table: "products",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool,
            mapper: |row| map_row_to_product(&row)
        )
    }

    async fn delete(&self, id: &Uuid) -> Result<(), Error> {
        delete!(
            table: "products",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool
        )
    }

    async fn exists(&self, id: &Uuid) -> Result<bool, Error> {
        exists!(
            table: "products",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool
        )
    }

    async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Product>, Error> {
        let id_bytes: Vec<&[u8]> = ids.iter().map(|id| id.as_bytes() as &[u8]).collect();
        select_by_ids!(
            table: "products",
            id_column: "id",
            ids: &id_bytes,
            pool: &self.pool,
            mapper: |row| map_row_to_product(&row)
        )
    }

    async fn find_all(&self) -> Result<Vec<Product>, Error> {
        select_all!(
            table: "products",
            pool: &self.pool,
            mapper: |row| map_row_to_product(&row)
        )
    }

    async fn count(&self) -> Result<i64, Error> {
        count!(table: "products", pool: &self.pool)
    }
}

#[async_trait]
impl SoftDeletable<Product> for ProductRepository {
    async fn soft_delete(&self, id: &Uuid) -> Result<(), Error> {
        soft_delete!(
            table: "products",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool
        )
    }

    async fn restore(&self, id: &Uuid) -> Result<(), Error> {
        restore!(
            table: "products",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool
        )
    }

    async fn find_by_id_with_deleted(&self, id: &Uuid) -> Result<Option<Product>, Error> {
        select_by_id_with_deleted!(
            table: "products",
            id_column: "id",
            id: id.as_bytes(),
            pool: &self.pool,
            mapper: |row| map_row_to_product(&row)
        )
    }

    async fn find_deleted(&self) -> Result<Vec<Product>, Error> {
        select_deleted!(
            table: "products",
            pool: &self.pool,
            mapper: |row| map_row_to_product(&row)
        )
    }
}

// Row から Product への変換
fn map_row_to_product(row: &sqlx::postgres::PgRow) -> Result<Product, sqlx::Error> {
    let id_bytes: Vec<u8> = row.try_get("id")?;
    let id = Uuid::from_slice(&id_bytes).map_err(|e| {
        sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid UUID: {e}"),
        )))
    })?;

    Ok(Product {
        id,
        name: row.try_get("name")?,
        price: row.try_get("price")?,
        stock: row.try_get("stock")?,
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

// テストヘルパー関数
async fn setup_test_db() -> Result<PgPool, Box<dyn std::error::Error>> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/effect_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // テーブルを作成
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS products (
            id BYTEA PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            price INTEGER NOT NULL,
            stock INTEGER NOT NULL,
            version BIGINT NOT NULL DEFAULT 1,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            deleted_at TIMESTAMPTZ
        )
        ",
    )
    .execute(&pool)
    .await?;

    // 既存のデータをクリア
    sqlx::query("TRUNCATE TABLE products")
        .execute(&pool)
        .await?;

    Ok(pool)
}

async fn teardown_test_db(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("DROP TABLE IF EXISTS products")
        .execute(pool)
        .await?;
    Ok(())
}

#[tokio::test]
#[ignore = "TEST_DATABASE_URL が必要"]
async fn test_repository_crud_operations() {
    let pool = setup_test_db().await.unwrap();
    let repo = ProductRepository::new(pool.clone());

    // Create
    let product = Product::new("Test Product".to_string(), 1000, 10);
    let product_id = product.id;
    repo.save(&product).await.unwrap();

    // Read
    let found = repo.find_by_id(&product_id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name, "Test Product");
    assert_eq!(found.price, 1000);
    assert_eq!(found.stock, 10);
    assert_eq!(found.version, 1);

    // Update
    let mut updated_product = found;
    updated_product.price = 1500;
    updated_product.stock = 20;
    repo.save(&updated_product).await.unwrap();

    let found = repo.find_by_id(&product_id).await.unwrap().unwrap();
    assert_eq!(found.price, 1500);
    assert_eq!(found.stock, 20);
    assert_eq!(found.version, 2);

    // Delete
    repo.delete(&product_id).await.unwrap();
    let found = repo.find_by_id(&product_id).await.unwrap();
    assert!(found.is_none());

    teardown_test_db(&pool).await.unwrap();
}

#[tokio::test]
#[ignore = "TEST_DATABASE_URL が必要"]
async fn test_optimistic_locking() {
    let pool = setup_test_db().await.unwrap();
    let repo = ProductRepository::new(pool.clone());

    let product = Product::new("Test Product".to_string(), 1000, 10);
    let product_id = product.id;
    repo.save(&product).await.unwrap();

    // 同じ製品を2回取得
    let product1 = repo.find_by_id(&product_id).await.unwrap().unwrap();
    let mut product2 = repo.find_by_id(&product_id).await.unwrap().unwrap();

    // product2 を先に更新
    product2.price = 2000;
    repo.save(&product2).await.unwrap();

    // product1 を更新しようとするとエラーになるはず
    let mut product1 = product1;
    product1.price = 1500;
    let result = repo.save(&product1).await;
    assert!(matches!(result, Err(Error::OptimisticLockFailure { .. })));

    teardown_test_db(&pool).await.unwrap();
}

#[tokio::test]
#[ignore = "TEST_DATABASE_URL が必要"]
async fn test_soft_delete_operations() {
    let pool = setup_test_db().await.unwrap();
    let repo = ProductRepository::new(pool.clone());

    let product = Product::new("Test Product".to_string(), 1000, 10);
    let product_id = product.id;
    repo.save(&product).await.unwrap();

    // ソフトデリート
    repo.soft_delete(&product_id).await.unwrap();

    // 通常の検索では見つからない
    let found = repo.find_by_id(&product_id).await.unwrap();
    assert!(found.is_none());

    // 削除済みを含めた検索では見つかる
    let found = repo.find_by_id_with_deleted(&product_id).await.unwrap();
    assert!(found.is_some());
    assert!(found.unwrap().deleted_at.is_some());

    // 削除済みのみを検索
    let deleted = repo.find_deleted().await.unwrap();
    assert_eq!(deleted.len(), 1);

    // リストア
    repo.restore(&product_id).await.unwrap();

    // 通常の検索で再び見つかる
    let found = repo.find_by_id(&product_id).await.unwrap();
    assert!(found.is_some());
    assert!(found.unwrap().deleted_at.is_none());

    teardown_test_db(&pool).await.unwrap();
}

#[tokio::test]
#[ignore = "TEST_DATABASE_URL が必要"]
async fn test_batch_operations() {
    let pool = setup_test_db().await.unwrap();
    let repo = ProductRepository::new(pool.clone());

    // 複数の製品を作成
    let products: Vec<_> = (0..5)
        .map(|i| Product::new(format!("Product {i}"), 1000 * (i + 1), 10 * (i + 1)))
        .collect();

    for product in &products {
        repo.save(product).await.unwrap();
    }

    // find_all
    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 5);

    // count
    let count = repo.count().await.unwrap();
    assert_eq!(count, 5);

    // find_by_ids
    let ids: Vec<_> = products.iter().take(3).map(|p| p.id).collect();
    let found = repo.find_by_ids(&ids).await.unwrap();
    assert_eq!(found.len(), 3);

    // ソフトデリートして count を確認
    repo.soft_delete(&products[0].id).await.unwrap();
    let count = repo.count().await.unwrap();
    assert_eq!(count, 4); // ソフトデリートされたものは除外

    teardown_test_db(&pool).await.unwrap();
}

#[tokio::test]
#[ignore = "TEST_DATABASE_URL が必要"]
async fn test_exists_operation() {
    let pool = setup_test_db().await.unwrap();
    let repo = ProductRepository::new(pool.clone());

    let product = Product::new("Test Product".to_string(), 1000, 10);
    let product_id = product.id;

    // 存在しない
    assert!(!repo.exists(&product_id).await.unwrap());

    // 保存
    repo.save(&product).await.unwrap();

    // 存在する
    assert!(repo.exists(&product_id).await.unwrap());

    // ソフトデリート後も exists は false を返す
    repo.soft_delete(&product_id).await.unwrap();
    assert!(!repo.exists(&product_id).await.unwrap());

    teardown_test_db(&pool).await.unwrap();
}

#[tokio::test]
#[ignore = "TEST_DATABASE_URL が必要"]
async fn test_unique_constraint_violation() {
    let pool = setup_test_db().await.unwrap();

    // name にユニーク制約を追加
    sqlx::query("ALTER TABLE products ADD CONSTRAINT products_name_key UNIQUE (name)")
        .execute(&pool)
        .await
        .unwrap();

    let repo = ProductRepository::new(pool.clone());

    let product1 = Product::new("Unique Name".to_string(), 1000, 10);
    repo.save(&product1).await.unwrap();

    let product2 = Product::new("Unique Name".to_string(), 2000, 20);
    let result = repo.save(&product2).await;

    assert!(matches!(result, Err(Error::UniqueViolation { .. })));

    teardown_test_db(&pool).await.unwrap();
}
