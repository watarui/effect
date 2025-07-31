//! キャッシュ管理
//!
//! このモジュールは Redis を使用したキャッシュ機能を提供します。

use redis::{Client as RedisClient, RedisError, aio::ConnectionManager};
use thiserror::Error;

/// キャッシュ関連のエラー
#[derive(Debug, Error)]
pub enum Error {
    /// 接続エラー
    #[error("Cache connection error: {0}")]
    Connection(String),

    /// Redis エラー
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    /// シリアライゼーションエラー
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Redis キャッシュクライアント
#[derive(Clone)]
pub struct Client {
    connection: ConnectionManager,
}

impl Client {
    /// 新しいキャッシュクライアントを作成
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis 接続 URL (例: `<redis://localhost:6379>`)
    ///
    /// # Errors
    ///
    /// Redis への接続に失敗した場合はエラーを返す
    pub async fn new(redis_url: &str) -> Result<Self, Error> {
        let client = RedisClient::open(redis_url)
            .map_err(|e| Error::Connection(format!("Failed to create Redis client: {e}")))?;

        let connection = ConnectionManager::new(client)
            .await
            .map_err(|e| Error::Connection(format!("Failed to connect to Redis: {e}")))?;

        Ok(Self { connection })
    }

    /// 値を取得
    ///
    /// # Errors
    ///
    /// Redis 操作またはデシリアライゼーションに失敗した場合はエラーを返す
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        use redis::AsyncCommands;

        let mut conn = self.connection.clone();
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(json) => {
                let result = serde_json::from_str(&json)
                    .map_err(|e| Error::Serialization(format!("Failed to deserialize: {e}")))?;
                Ok(Some(result))
            },
            None => Ok(None),
        }
    }

    /// 値を設定
    ///
    /// # Arguments
    ///
    /// * `key` - キャッシュキー
    /// * `value` - 保存する値
    /// * `ttl_seconds` - TTL（秒）。None の場合は永続化
    ///
    /// # Errors
    ///
    /// Redis 操作またはシリアライゼーションに失敗した場合はエラーを返す
    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<(), Error>
    where
        T: serde::Serialize + Sync,
    {
        use redis::AsyncCommands;

        let json = serde_json::to_string(value)
            .map_err(|e| Error::Serialization(format!("Failed to serialize: {e}")))?;

        let mut conn = self.connection.clone();
        match ttl_seconds {
            Some(ttl) => conn.set_ex(key, json, ttl).await?,
            None => conn.set(key, json).await?,
        }

        Ok(())
    }

    /// 値を削除
    ///
    /// # Errors
    ///
    /// Redis 操作に失敗した場合はエラーを返す
    pub async fn delete(&self, key: &str) -> Result<bool, Error> {
        use redis::AsyncCommands;

        let mut conn = self.connection.clone();
        let deleted: i32 = conn.del(key).await?;
        Ok(deleted > 0)
    }

    /// キーの存在確認
    ///
    /// # Errors
    ///
    /// Redis 操作に失敗した場合はエラーを返す
    pub async fn exists(&self, key: &str) -> Result<bool, Error> {
        use redis::AsyncCommands;

        let mut conn = self.connection.clone();
        Ok(conn.exists(key).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_error_should_display_correctly() {
        let error = Error::Connection("test error".to_string());
        assert_eq!(error.to_string(), "Cache connection error: test error");
    }

    // 実際の Redis 接続テストは integration tests で実施
}
