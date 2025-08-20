//! PostgreSQL Read Model リポジトリ実装

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{debug, error};
use uuid::Uuid;

use crate::{
    domain::{
        Cursor,
        PageInfo,
        PageSize,
        PagedResult,
        SortField,
        SortOptions,
        SortOrder,
        VocabularyEntry,
        VocabularyEntryRow,
        VocabularyExample,
        VocabularyFilter,
        VocabularyItem,
        VocabularyItemRow,
        VocabularyStatistics,
    },
    error::{QueryError, Result},
    ports::outbound::ReadModelRepository,
};

/// PostgreSQL Read Model リポジトリ
#[derive(Clone)]
pub struct PostgresReadModelRepository {
    pool: PgPool,
}

impl PostgresReadModelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// ソート句を生成
    #[allow(dead_code)]
    fn build_sort_clause(&self, sort: Option<SortOptions>) -> String {
        match sort {
            Some(opts) => {
                let field = match opts.field {
                    SortField::Spelling => "spelling",
                    SortField::FrequencyRank => "frequency_rank",
                    SortField::CefrLevel => "cefr_level",
                    SortField::CreatedAt => "created_at",
                    SortField::UpdatedAt => "updated_at",
                    SortField::ExampleCount => "example_count",
                };
                let order = match opts.order {
                    SortOrder::Ascending => "ASC",
                    SortOrder::Descending => "DESC",
                };
                format!("{} {}", field, order)
            },
            None => "created_at DESC".to_string(),
        }
    }

    /// WHERE句を生成
    #[allow(dead_code)]
    fn build_where_clause(&self, filter: &VocabularyFilter) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        if let Some(ref term) = filter.search_term {
            conditions.push("(spelling ILIKE $1 OR definition ILIKE $1)".to_string());
            params.push(format!("%{}%", term));
        }

        if let Some(ref pos) = filter.part_of_speech {
            conditions.push(format!("part_of_speech = ${}", params.len() + 1));
            params.push(pos.clone());
        }

        if let Some(ref cefr) = filter.cefr_level {
            conditions.push(format!("cefr_level = ${}", params.len() + 1));
            params.push(cefr.clone());
        }

        if let Some(is_published) = filter.is_published {
            conditions.push(format!("is_published = {}", is_published));
        }

        if let Some(has_definition) = filter.has_definition {
            if has_definition {
                conditions.push("definition IS NOT NULL".to_string());
            } else {
                conditions.push("definition IS NULL".to_string());
            }
        }

        if let Some(min_freq) = filter.min_frequency {
            conditions.push(format!("frequency_rank >= {}", min_freq));
        }

        if let Some(max_freq) = filter.max_frequency {
            conditions.push(format!("frequency_rank <= {}", max_freq));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        (where_clause, params)
    }
}

#[async_trait]
impl ReadModelRepository for PostgresReadModelRepository {
    async fn find_entry_by_id(&self, entry_id: Uuid) -> Result<Option<VocabularyEntry>> {
        let row = sqlx::query_as!(
            VocabularyEntryRow,
            r#"
            SELECT 
                entry_id,
                spelling,
                primary_item_id,
                item_count,
                created_at,
                updated_at
            FROM vocabulary_entries_read
            WHERE entry_id = $1
            "#,
            entry_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(VocabularyEntry::from))
    }

    async fn find_entry_by_spelling(&self, spelling: &str) -> Result<Option<VocabularyEntry>> {
        let row = sqlx::query_as!(
            VocabularyEntryRow,
            r#"
            SELECT 
                entry_id,
                spelling,
                primary_item_id,
                item_count,
                created_at,
                updated_at
            FROM vocabulary_entries_read
            WHERE spelling = $1
            "#,
            spelling
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(VocabularyEntry::from))
    }

    async fn find_entries(
        &self,
        _filter: Option<VocabularyFilter>,
        _sort: Option<SortOptions>,
        cursor: Option<Cursor>,
        limit: PageSize,
    ) -> Result<PagedResult<VocabularyEntry>> {
        let limit_val = limit.value() as i64;

        // カーソルベースのページネーション
        let has_cursor = cursor.is_some();
        let cursor_id = cursor
            .as_ref()
            .map(|c| Uuid::parse_str(c.value()).unwrap_or(Uuid::nil()));

        let entries: Vec<VocabularyEntry> = if let Some(cursor_id) = cursor_id {
            sqlx::query_as!(
                VocabularyEntryRow,
                r#"
                SELECT 
                    entry_id,
                    spelling,
                    primary_item_id,
                    item_count,
                    created_at,
                    updated_at
                FROM vocabulary_entries_read
                WHERE entry_id > $1
                ORDER BY created_at DESC
                LIMIT $2
                "#,
                cursor_id,
                limit_val
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(VocabularyEntry::from)
            .collect()
        } else {
            sqlx::query_as!(
                VocabularyEntryRow,
                r#"
                SELECT 
                    entry_id,
                    spelling,
                    primary_item_id,
                    item_count,
                    created_at,
                    updated_at
                FROM vocabulary_entries_read
                ORDER BY created_at DESC
                LIMIT $1
                "#,
                limit_val
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(VocabularyEntry::from)
            .collect()
        };

        let has_more = entries.len() as u32 == limit.value();
        let end_cursor = entries.last().map(|e| e.entry_id.to_string());
        let start_cursor = cursor.map(|c| c.value().to_string());

        Ok(PagedResult {
            items:     entries,
            page_info: PageInfo {
                has_next_page: has_more,
                has_previous_page: has_cursor,
                start_cursor,
                end_cursor,
                total_count: None,
            },
        })
    }

    async fn find_item_by_id(&self, item_id: Uuid) -> Result<Option<VocabularyItem>> {
        let row = sqlx::query_as!(
            VocabularyItemRow,
            r#"
            SELECT 
                item_id,
                entry_id,
                spelling,
                disambiguation,
                part_of_speech,
                definition,
                ipa_pronunciation,
                cefr_level,
                frequency_rank,
                is_published,
                is_deleted,
                example_count,
                created_at,
                updated_at
            FROM vocabulary_items_read
            WHERE item_id = $1
            "#,
            item_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(VocabularyItem::from))
    }

    async fn find_items_by_entry_id(
        &self,
        entry_id: Uuid,
        include_deleted: bool,
    ) -> Result<Vec<VocabularyItem>> {
        let items: Vec<VocabularyItem> = if include_deleted {
            sqlx::query_as!(
                VocabularyItemRow,
                r#"
                SELECT 
                    item_id,
                    entry_id,
                    spelling,
                    disambiguation,
                    part_of_speech,
                    definition,
                    ipa_pronunciation,
                    cefr_level,
                    frequency_rank,
                    is_published,
                    is_deleted,
                    example_count,
                    created_at,
                    updated_at
                FROM vocabulary_items_read
                WHERE entry_id = $1
                ORDER BY created_at
                "#,
                entry_id
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(VocabularyItem::from)
            .collect()
        } else {
            sqlx::query_as!(
                VocabularyItemRow,
                r#"
                SELECT 
                    item_id,
                    entry_id,
                    spelling,
                    disambiguation,
                    part_of_speech,
                    definition,
                    ipa_pronunciation,
                    cefr_level,
                    frequency_rank,
                    is_published,
                    is_deleted,
                    example_count,
                    created_at,
                    updated_at
                FROM vocabulary_items_read
                WHERE entry_id = $1 AND NOT is_deleted
                ORDER BY created_at
                "#,
                entry_id
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(VocabularyItem::from)
            .collect()
        };

        Ok(items)
    }

    async fn find_items(
        &self,
        _filter: Option<VocabularyFilter>,
        _sort: Option<SortOptions>,
        _cursor: Option<Cursor>,
        _limit: PageSize,
    ) -> Result<PagedResult<VocabularyItem>> {
        // TODO: フィルターとソートの実装
        let items = Vec::new();

        Ok(PagedResult {
            items,
            page_info: PageInfo {
                has_next_page:     false,
                has_previous_page: false,
                start_cursor:      None,
                end_cursor:        None,
                total_count:       Some(0),
            },
        })
    }

    async fn find_examples_by_item_id(&self, item_id: Uuid) -> Result<Vec<VocabularyExample>> {
        let examples = sqlx::query_as!(
            VocabularyExample,
            r#"
            SELECT 
                example_id,
                item_id,
                example,
                translation,
                added_by,
                created_at
            FROM vocabulary_examples_read
            WHERE item_id = $1
            ORDER BY created_at
            "#,
            item_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(examples)
    }

    async fn search_items(
        &self,
        search_term: &str,
        _filter: Option<VocabularyFilter>,
        cursor: Option<Cursor>,
        limit: PageSize,
    ) -> Result<PagedResult<VocabularyItem>> {
        debug!("Searching for term: {}", search_term);

        // 基本的な LIKE 検索（後で Meilisearch に置き換え）
        let search_pattern = format!("%{}%", search_term);
        let limit_val = limit.value() as i64;

        let items: Vec<VocabularyItem> = sqlx::query_as!(
            VocabularyItemRow,
            r#"
            SELECT 
                item_id,
                entry_id,
                spelling,
                disambiguation,
                part_of_speech,
                definition,
                ipa_pronunciation,
                cefr_level,
                frequency_rank,
                is_published,
                is_deleted,
                example_count,
                created_at,
                updated_at
            FROM vocabulary_items_read
            WHERE (spelling ILIKE $1 OR definition ILIKE $1)
                AND NOT is_deleted
            ORDER BY 
                CASE WHEN spelling ILIKE $2 THEN 0 ELSE 1 END,
                frequency_rank DESC NULLS LAST
            LIMIT $3
            "#,
            search_pattern,
            format!("{}%", search_term),
            limit_val
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(VocabularyItem::from)
        .collect();

        let has_more = items.len() as u32 == limit.value();

        Ok(PagedResult {
            items,
            page_info: PageInfo {
                has_next_page:     has_more,
                has_previous_page: cursor.is_some(),
                start_cursor:      None,
                end_cursor:        None,
                total_count:       None,
            },
        })
    }

    async fn get_statistics(&self) -> Result<VocabularyStatistics> {
        let row = sqlx::query!(
            r#"
            SELECT 
                COUNT(DISTINCT e.entry_id) as total_entries,
                COUNT(DISTINCT i.item_id) as total_items,
                COUNT(DISTINCT ex.example_id) as total_examples,
                COUNT(DISTINCT i.item_id) FILTER (WHERE i.is_published) as published_items
            FROM vocabulary_entries_read e
            LEFT JOIN vocabulary_items_read i ON e.entry_id = i.entry_id
            LEFT JOIN vocabulary_examples_read ex ON i.item_id = ex.item_id
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(VocabularyStatistics {
            total_entries:   row.total_entries.unwrap_or(0),
            total_items:     row.total_items.unwrap_or(0),
            total_examples:  row.total_examples.unwrap_or(0),
            published_items: row.published_items.unwrap_or(0),
            items_by_pos:    std::collections::HashMap::new(),
            items_by_cefr:   std::collections::HashMap::new(),
        })
    }

    async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                error!("Database health check failed: {}", e);
                QueryError::Database(e)
            })?;
        Ok(())
    }
}
