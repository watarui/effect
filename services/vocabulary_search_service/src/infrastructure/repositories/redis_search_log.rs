//! Redis 検索ログリポジトリ

use async_trait::async_trait;
use redis::AsyncCommands;

use crate::{
    error::{Result, SearchError},
    ports::outbound::{SearchLogRepository, SearchStatistics},
};

/// Redis 検索ログリポジトリ
pub struct RedisSearchLogRepository {
    client: redis::Client,
}

impl RedisSearchLogRepository {
    pub fn new(redis_url: String) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| SearchError::Configuration(format!("Redis connection error: {}", e)))?;
        Ok(Self { client })
    }

    /// 検索クエリのキー
    fn search_query_key(&self, query: &str) -> String {
        format!("search:query:{}", query)
    }

    /// 人気検索クエリのキー
    fn popular_queries_key(&self) -> String {
        "search:popular".to_string()
    }

    /// 日次統計のキー
    fn daily_stats_key(&self) -> String {
        let today = chrono::Utc::now().format("%Y-%m-%d");
        format!("search:stats:daily:{}", today)
    }
}

#[async_trait]
impl SearchLogRepository for RedisSearchLogRepository {
    async fn log_search(&self, query: &str, results_count: usize) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SearchError::Network(e.to_string()))?;

        // クエリの検索回数をインクリメント
        let query_key = self.search_query_key(query);
        let _: () = conn.incr(&query_key, 1).await.unwrap_or(());

        // 人気検索クエリの更新（ZSET）
        let popular_key = self.popular_queries_key();
        let _: () = conn.zadd(&popular_key, query, 1).await.unwrap_or(());

        // 日次統計の更新
        let daily_key = self.daily_stats_key();
        let _: () = conn
            .hincr(&daily_key, "total_searches", 1)
            .await
            .unwrap_or(());
        let _: () = conn
            .hincr(&daily_key, "total_results", results_count as i64)
            .await
            .unwrap_or(());

        // TTL を設定（30日間保持）
        let _: () = conn.expire(&query_key, 2592000).await.unwrap_or(());
        let _: () = conn.expire(&daily_key, 2592000).await.unwrap_or(());

        Ok(())
    }

    async fn get_popular_queries(&self, limit: usize) -> Result<Vec<String>> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SearchError::Network(e.to_string()))?;

        let popular_key = self.popular_queries_key();

        // スコアの高い順に取得
        let queries: Vec<(String, f64)> = conn
            .zrevrange_withscores(&popular_key, 0, (limit - 1) as isize)
            .await
            .map_err(|e| SearchError::Network(e.to_string()))?;

        Ok(queries.into_iter().map(|(query, _)| query).collect())
    }

    async fn get_search_statistics(&self) -> Result<SearchStatistics> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SearchError::Network(e.to_string()))?;

        // 今日の統計を取得
        let daily_key = self.daily_stats_key();
        let total_searches: u64 = conn.hget(&daily_key, "total_searches").await.unwrap_or(0);
        let total_results: u64 = conn.hget(&daily_key, "total_results").await.unwrap_or(0);

        // 人気クエリトップ10を取得
        let popular_key = self.popular_queries_key();
        let top_queries: Vec<(String, f64)> = conn
            .zrevrange_withscores(&popular_key, 0, 9)
            .await
            .unwrap_or_default();

        // ユニーククエリ数を取得
        let unique_queries: u64 = conn.zcard(&popular_key).await.unwrap_or(0);

        let avg_results_per_query = if total_searches > 0 {
            total_results as f64 / total_searches as f64
        } else {
            0.0
        };

        Ok(SearchStatistics {
            total_searches,
            unique_queries,
            avg_results_per_query,
            top_queries: top_queries
                .into_iter()
                .map(|(q, s)| (q, s as u64))
                .collect(),
            search_trends: Vec::new(), // TODO: トレンド分析の実装
        })
    }
}
