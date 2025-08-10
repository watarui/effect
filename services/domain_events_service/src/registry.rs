//! スキーマレジストリの実装

use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::config::RegistryConfig;

/// スキーマ情報
#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub id:          Uuid,
    pub event_type:  String,
    pub version:     i32,
    pub definition:  String,
    pub description: String,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}

/// イベントタイプ情報
#[derive(Debug, Clone)]
pub struct EventTypeInfo {
    pub event_type:      String,
    pub context:         String,
    pub description:     String,
    pub current_version: i32,
    pub is_deprecated:   bool,
}

/// スキーマレジストリ
#[derive(Clone)]
pub struct SchemaRegistry {
    pool:   PgPool,
    config: RegistryConfig,
    cache:  Arc<RwLock<HashMap<String, SchemaInfo>>>,
}

impl SchemaRegistry {
    /// 新しいスキーマレジストリを作成
    #[must_use]
    pub fn new(pool: PgPool, config: RegistryConfig) -> Self {
        Self {
            pool,
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// スキーマを取得
    pub async fn get_schema(
        &self,
        event_type: &str,
        version: Option<i32>,
    ) -> Result<SchemaInfo, SchemaRegistryError> {
        // キャッシュから取得を試みる
        let cache_key = format!("{}-{}", event_type, version.unwrap_or(-1));

        {
            let cache = self.cache.read().await;
            if let Some(schema) = cache.get(&cache_key) {
                return Ok(schema.clone());
            }
        }

        // データベースから取得
        let schema = if let Some(v) = version {
            sqlx::query_as!(
                SchemaRow,
                r#"
                SELECT 
                    id, event_type, version, definition, description,
                    created_at, updated_at
                FROM event_schemas
                WHERE event_type = $1 AND version = $2
                "#,
                event_type,
                v
            )
            .fetch_optional(&self.pool)
            .await?
        } else {
            // 最新バージョンを取得
            sqlx::query_as!(
                SchemaRow,
                r#"
                SELECT 
                    id, event_type, version, definition, description,
                    created_at, updated_at
                FROM event_schemas
                WHERE event_type = $1
                ORDER BY version DESC
                LIMIT 1
                "#,
                event_type
            )
            .fetch_optional(&self.pool)
            .await?
        };

        match schema {
            Some(row) => {
                let schema_info = SchemaInfo {
                    id:          row.id,
                    event_type:  row.event_type,
                    version:     row.version,
                    definition:  row.definition,
                    description: row.description,
                    created_at:  row.created_at,
                    updated_at:  row.updated_at,
                };

                // キャッシュに保存
                {
                    let mut cache = self.cache.write().await;
                    cache.insert(cache_key.clone(), schema_info.clone());
                }

                // キャッシュ有効期限後にクリア
                let cache = Arc::clone(&self.cache);
                let cache_key_clone = cache_key.clone();
                let ttl = self.config.cache_ttl_seconds;
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(ttl)).await;
                    let mut cache = cache.write().await;
                    cache.remove(&cache_key_clone);
                });

                Ok(schema_info)
            },
            None => Err(SchemaRegistryError::SchemaNotFound {
                event_type: event_type.to_string(),
                version,
            }),
        }
    }

    /// スキーマを登録
    pub async fn register_schema(
        &self,
        event_type: &str,
        definition: &str,
        description: &str,
    ) -> Result<(Uuid, i32), SchemaRegistryError> {
        // 現在の最大バージョンを取得
        let current_max_version: Option<i32> = sqlx::query_scalar!(
            r#"
            SELECT MAX(version)
            FROM event_schemas
            WHERE event_type = $1
            "#,
            event_type
        )
        .fetch_one(&self.pool)
        .await?;

        let new_version = current_max_version.unwrap_or(0) + 1;

        // 最大バージョン数をチェック
        let max_versions_i32 = i32::try_from(self.config.max_versions).unwrap_or(i32::MAX);
        if new_version > max_versions_i32 {
            return Err(SchemaRegistryError::MaxVersionsExceeded {
                event_type:   event_type.to_string(),
                max_versions: self.config.max_versions,
            });
        }

        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO event_schemas (
                id, event_type, version, definition, description,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            id,
            event_type,
            new_version,
            definition,
            description,
            now,
            now
        )
        .execute(&self.pool)
        .await?;

        // キャッシュをクリア
        {
            let mut cache = self.cache.write().await;
            cache.clear();
        }

        Ok((id, new_version))
    }

    /// イベントタイプ一覧を取得
    pub async fn list_event_types(
        &self,
        context: Option<&str>,
    ) -> Result<Vec<EventTypeInfo>, SchemaRegistryError> {
        #[derive(Debug, sqlx::FromRow)]
        struct EventTypeRow {
            event_type:  String,
            version:     i32,
            description: String,
            #[allow(dead_code)]
            created_at:  DateTime<Utc>,
        }

        let rows = if let Some(ctx) = context {
            sqlx::query_as::<_, EventTypeRow>(
                r"
                SELECT DISTINCT ON (event_type)
                    event_type, version, description, created_at
                FROM event_schemas
                WHERE event_type LIKE $1
                ORDER BY event_type, version DESC
                ",
            )
            .bind(format!("{ctx}.%"))
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, EventTypeRow>(
                r"
                SELECT DISTINCT ON (event_type)
                    event_type, version, description, created_at
                FROM event_schemas
                ORDER BY event_type, version DESC
                ",
            )
            .fetch_all(&self.pool)
            .await?
        };

        let event_types = rows
            .into_iter()
            .map(|row| {
                let parts: Vec<&str> = row.event_type.split('.').collect();
                let context = (*parts.first().unwrap_or(&"unknown")).to_string();

                EventTypeInfo {
                    event_type: row.event_type,
                    context,
                    description: row.description,
                    current_version: row.version,
                    is_deprecated: false, // TODO: 実装
                }
            })
            .collect();

        Ok(event_types)
    }

    /// スキーマバージョン情報を取得
    pub async fn get_schema_versions(
        &self,
        event_type: &str,
    ) -> Result<(i32, Vec<i32>), SchemaRegistryError> {
        let versions: Vec<i32> = sqlx::query_scalar!(
            r#"
            SELECT version
            FROM event_schemas
            WHERE event_type = $1
            ORDER BY version DESC
            "#,
            event_type
        )
        .fetch_all(&self.pool)
        .await?;

        match versions.first() {
            Some(&current) => Ok((current, versions)),
            None => Err(SchemaRegistryError::SchemaNotFound {
                event_type: event_type.to_string(),
                version:    None,
            }),
        }
    }
}

// 内部用の行構造体
#[derive(Debug)]
struct SchemaRow {
    id:          Uuid,
    event_type:  String,
    version:     i32,
    definition:  String,
    description: String,
    created_at:  DateTime<Utc>,
    updated_at:  DateTime<Utc>,
}

/// スキーマレジストリのエラー
#[derive(Debug, thiserror::Error)]
pub enum SchemaRegistryError {
    #[error("Schema not found: {event_type} (version: {version:?})")]
    SchemaNotFound {
        event_type: String,
        version:    Option<i32>,
    },

    #[error("Max versions exceeded for {event_type}: {max_versions}")]
    MaxVersionsExceeded {
        event_type:   String,
        max_versions: usize,
    },

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
