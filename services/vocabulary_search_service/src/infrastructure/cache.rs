//! キャッシュ実装

#![allow(clippy::type_complexity)]
#![allow(clippy::needless_borrows_for_generic_args)]

use std::time::Duration;

use async_trait::async_trait;
use redis::{AsyncCommands, aio::ConnectionManager};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{domain::error::SearchError, ports::outbound::CacheService};

/// Redis キャッシュサービス
pub struct RedisCacheService {
    conn:        ConnectionManager,
    default_ttl: Duration,
    key_prefix:  String,
}

impl RedisCacheService {
    /// 新しい Redis キャッシュサービスを作成
    pub fn new(
        conn: ConnectionManager,
        default_ttl: Duration,
        key_prefix: impl Into<String>,
    ) -> Self {
        Self {
            conn,
            default_ttl,
            key_prefix: key_prefix.into(),
        }
    }

    /// キーにプレフィックスを付ける
    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }
}

#[async_trait]
impl CacheService for RedisCacheService {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, SearchError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let full_key = self.make_key(key);

        debug!("Getting cache key: {}", full_key);

        let mut conn = self.conn.clone();
        let value: Option<String> = conn.get(&full_key).await?;

        match value {
            Some(json) => {
                let result = serde_json::from_str(&json)?;
                debug!("Cache hit for key: {}", full_key);
                Ok(Some(result))
            },
            None => {
                debug!("Cache miss for key: {}", full_key);
                Ok(None)
            },
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), SearchError>
    where
        T: Serialize + Sync,
    {
        let full_key = self.make_key(key);
        let ttl = ttl.unwrap_or(self.default_ttl);

        debug!("Setting cache key: {} with TTL: {:?}", full_key, ttl);

        // Serialize before the async block to avoid Send issues
        let json = serde_json::to_string(value)?;
        let mut conn = self.conn.clone();

        let _: () = conn.set_ex(&full_key, json, ttl.as_secs()).await?;

        debug!("Cache set for key: {}", full_key);

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), SearchError> {
        let full_key = self.make_key(key);

        debug!("Deleting cache key: {}", full_key);

        let mut conn = self.conn.clone();
        let _: () = conn.del(&full_key).await?;

        debug!("Cache deleted for key: {}", full_key);

        Ok(())
    }

    async fn clear_pattern(&self, pattern: &str) -> Result<(), SearchError> {
        let full_pattern = self.make_key(pattern);

        debug!("Clearing cache pattern: {}", full_pattern);

        let mut conn = self.conn.clone();

        // SCAN を使用してパターンに一致するキーを取得
        let mut cursor = 0;
        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&full_pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await?;

            if !keys.is_empty() {
                let _: () = conn.del(keys).await?;
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        debug!("Cache cleared for pattern: {}", full_pattern);

        Ok(())
    }
}

/// インメモリキャッシュサービス（テスト用）
#[cfg(test)]
pub mod mock {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use super::*;

    pub struct InMemoryCacheService {
        store:       Arc<Mutex<HashMap<String, (String, std::time::Instant, Duration)>>>,
        default_ttl: Duration,
        key_prefix:  String,
    }

    impl InMemoryCacheService {
        pub fn new(default_ttl: Duration, key_prefix: impl Into<String>) -> Self {
            Self {
                store: Arc::new(Mutex::new(HashMap::new())),
                default_ttl,
                key_prefix: key_prefix.into(),
            }
        }

        fn make_key(&self, key: &str) -> String {
            format!("{}:{}", self.key_prefix, key)
        }

        fn is_expired(&self, created: std::time::Instant, ttl: Duration) -> bool {
            created.elapsed() > ttl
        }
    }

    #[async_trait]
    impl CacheService for InMemoryCacheService {
        async fn get<T>(&self, key: &str) -> Result<Option<T>, SearchError>
        where
            T: for<'de> Deserialize<'de>,
        {
            let full_key = self.make_key(key);
            let mut store = self.store.lock().unwrap();

            if let Some((json, created, ttl)) = store.get(&full_key) {
                if self.is_expired(*created, *ttl) {
                    store.remove(&full_key);
                    Ok(None)
                } else {
                    let result = serde_json::from_str(json)?;
                    Ok(Some(result))
                }
            } else {
                Ok(None)
            }
        }

        async fn set<T>(
            &self,
            key: &str,
            value: &T,
            ttl: Option<Duration>,
        ) -> Result<(), SearchError>
        where
            T: Serialize + Sync,
        {
            let full_key = self.make_key(key);
            let ttl = ttl.unwrap_or(self.default_ttl);
            let json = serde_json::to_string(value)?;

            let mut store = self.store.lock().unwrap();
            store.insert(full_key, (json, std::time::Instant::now(), ttl));

            Ok(())
        }

        async fn delete(&self, key: &str) -> Result<(), SearchError> {
            let full_key = self.make_key(key);
            let mut store = self.store.lock().unwrap();
            store.remove(&full_key);
            Ok(())
        }

        async fn clear_pattern(&self, pattern: &str) -> Result<(), SearchError> {
            let full_pattern = self.make_key(pattern);
            // Simple pattern matching without regex
            let pattern_prefix = full_pattern.trim_end_matches('*');

            let mut store = self.store.lock().unwrap();
            store.retain(|k, _| !k.starts_with(&pattern_prefix));

            Ok(())
        }
    }
}
