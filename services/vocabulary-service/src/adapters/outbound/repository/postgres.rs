//! `PostgreSQL` vocabulary repository implementation
//!
//! 新しい設計に基づく語彙リポジトリの実装

use std::str::FromStr;

use async_trait::async_trait;
use common_types::{EntryId, ItemId};
use domain_events::CefrLevel;
use infrastructure::repository::entity::Entity;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{
        entities::{vocabulary_entry::VocabularyEntry, vocabulary_item::VocabularyItem},
        value_objects::{
            cefr_level::Ext as CefrLevelExt,
            definition::{Collocation, Definition, Example},
            domain::Domain,
            part_of_speech::PartOfSpeech,
            register::Register,
        },
    },
    ports::outbound::repository::{
        Error as RepositoryError,
        Repository as VocabularyItemRepository,
    },
};

/// `VocabularyEntry` 構築用のパラメータ
struct EntryBuildParams {
    entry_id:         Uuid,
    spelling:         String,
    meaning:          String,
    part_of_speech:   PartOfSpeech,
    cefr_level:       Option<CefrLevel>,
    register:         Register,
    domain:           Domain,
    created_at:       chrono::DateTime<chrono::Utc>,
    last_modified_at: chrono::DateTime<chrono::Utc>,
    deleted_at:       Option<chrono::DateTime<chrono::Utc>>,
}

/// `VocabularyItem` 構築用のパラメータ
struct ItemBuildParams {
    item_id:          ItemId,
    entry:            VocabularyEntry,
    definitions:      Vec<Definition>,
    examples:         Vec<Example>,
    collocations:     Vec<Collocation>,
    pronunciation:    Option<String>,
    synonyms:         Vec<String>,
    antonyms:         Vec<String>,
    version:          u64,
    created_at:       chrono::DateTime<chrono::Utc>,
    last_modified_at: chrono::DateTime<chrono::Utc>,
    deleted_at:       Option<chrono::DateTime<chrono::Utc>>,
}

/// `PostgreSQL` vocabulary repository
#[derive(Clone)]
pub struct Repository {
    pool: PgPool,
}

impl Repository {
    /// 新しいインスタンスを作成
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 定義と関連データを取得するヘルパー関数
    async fn fetch_definitions_with_related(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        item_id: &Uuid,
    ) -> Result<
        (
            Vec<Definition>,
            Vec<Example>,
            Vec<String>, // synonyms
            Vec<String>, // antonyms
            Vec<Collocation>,
            PartOfSpeech,
            Option<Domain>,
            Option<String>, // definition_meaning for entry
        ),
        RepositoryError,
    > {
        let definitions_rows = sqlx::query!(
            r#"
            SELECT id, part_of_speech, meaning, meaning_translation,
                   domain, register, display_order
            FROM vocabulary_definitions
            WHERE item_id = $1
            ORDER BY display_order
            "#,
            item_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        let mut definitions = Vec::new();
        let mut all_examples = Vec::new();
        let mut all_synonyms = Vec::new();
        let mut all_antonyms = Vec::new();
        let mut all_collocations = Vec::new();

        for def_row in &definitions_rows {
            definitions.push(
                Definition::new(&def_row.meaning)
                    .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?,
            );

            let (examples, synonyms, antonyms, collocations) =
                self.fetch_definition_related_data(tx, &def_row.id).await?;

            all_examples.extend(examples);
            all_synonyms.extend(synonyms);
            all_antonyms.extend(antonyms);
            all_collocations.extend(collocations);
        }

        let part_of_speech = PartOfSpeech::from_str(&definitions_rows[0].part_of_speech)
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;
        let domain = definitions_rows[0]
            .domain
            .as_ref()
            .and_then(|d| Domain::from_str(d).ok());
        let definition_meaning = Some(definitions_rows[0].meaning.clone());

        Ok((
            definitions,
            all_examples,
            all_synonyms,
            all_antonyms,
            all_collocations,
            part_of_speech,
            domain,
            definition_meaning,
        ))
    }

    /// 単一定義の関連データを取得
    async fn fetch_definition_related_data(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        definition_id: &Uuid,
    ) -> Result<(Vec<Example>, Vec<String>, Vec<String>, Vec<Collocation>), RepositoryError> {
        // examples を取得
        let example_rows = sqlx::query!(
            r#"
            SELECT example_text, example_translation
            FROM vocabulary_examples
            WHERE definition_id = $1
            ORDER BY display_order
            "#,
            definition_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        let examples: Result<Vec<_>, _> = example_rows
            .iter()
            .map(|ex_row| Example::new(&ex_row.example_text, ex_row.example_translation.as_deref()))
            .collect();
        let examples = examples.map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        // synonyms を取得
        let synonym_rows = sqlx::query!(
            r#"
            SELECT synonym
            FROM vocabulary_synonyms
            WHERE definition_id = $1
            ORDER BY display_order
            "#,
            definition_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        let synonyms: Vec<_> = synonym_rows.iter().map(|r| r.synonym.clone()).collect();

        // antonyms を取得
        let antonym_rows = sqlx::query!(
            r#"
            SELECT antonym
            FROM vocabulary_antonyms
            WHERE definition_id = $1
            ORDER BY display_order
            "#,
            definition_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        let antonyms: Vec<_> = antonym_rows.iter().map(|r| r.antonym.clone()).collect();

        // collocations を取得
        let collocation_rows = sqlx::query!(
            r#"
            SELECT collocation_type, pattern, example
            FROM vocabulary_collocations
            WHERE definition_id = $1
            ORDER BY display_order
            "#,
            definition_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        let collocations: Vec<_> = collocation_rows
            .iter()
            .map(|col_row| {
                let examples = col_row
                    .example
                    .as_ref()
                    .map_or_else(Vec::new, |ex| vec![ex.as_str()]);
                Collocation::new(&col_row.pattern, examples)
            })
            .collect();

        Ok((examples, synonyms, antonyms, collocations))
    }

    /// `vocabulary_entry` を保存または取得
    async fn save_or_get_entry(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        entry: &VocabularyEntry,
    ) -> Result<Uuid, RepositoryError> {
        let entry_exists = sqlx::query!(
            "SELECT id FROM vocabulary_entries WHERE spelling = $1",
            entry.word()
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        if let Some(existing) = entry_exists {
            Ok(existing.id)
        } else {
            Ok(sqlx::query!(
                "INSERT INTO vocabulary_entries (id, spelling) VALUES ($1, $2) RETURNING id",
                entry.id().as_uuid(),
                entry.word()
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?
            .id)
        }
    }

    /// `vocabulary_item` を保存
    async fn save_vocabulary_item(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        item: &VocabularyItem,
        entry_id: Uuid,
    ) -> Result<(), RepositoryError> {
        let entry = item.entry();
        let disambiguation = "(default)"; // TODO: 実際の値を使用
        let status = "published";
        let created_by_type = "user";
        let created_by_id = Uuid::nil(); // TODO: 実際のユーザーIDを使用
        let last_modified_by = Uuid::nil(); // TODO: 実際のユーザーIDを使用

        sqlx::query!(
            r#"
            INSERT INTO vocabulary_items (
                id, entry_id, spelling, disambiguation, pronunciation,
                phonetic_respelling, audio_url, register, cefr_level,
                status, created_by_type, created_by_id, created_at,
                last_modified_at, last_modified_by, version
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (spelling, disambiguation) WHERE deleted_at IS NULL
            DO UPDATE SET
                pronunciation = EXCLUDED.pronunciation,
                phonetic_respelling = EXCLUDED.phonetic_respelling,
                audio_url = EXCLUDED.audio_url,
                register = EXCLUDED.register,
                cefr_level = EXCLUDED.cefr_level,
                last_modified_at = EXCLUDED.last_modified_at,
                last_modified_by = EXCLUDED.last_modified_by,
                version = vocabulary_items.version + 1
            "#,
            item.id().as_uuid(),
            entry_id,
            entry.word(),
            disambiguation,
            item.pronunciation(),
            None::<String>, // phonetic_respelling
            None::<String>, // audio_url
            entry.register().to_string(),
            entry.cefr_level().map(|l| l.as_str().to_string()),
            status,
            created_by_type,
            created_by_id,
            Entity::created_at(item),
            Entity::updated_at(item),
            last_modified_by,
            i32::try_from(Entity::version(item)).map_err(|_| RepositoryError::OperationFailed(
                "Version conversion failed".to_string()
            ))?
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        Ok(())
    }

    /// 定義と関連データを保存
    async fn save_definitions_and_related(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        item: &VocabularyItem,
        entry: &VocabularyEntry,
    ) -> Result<(), RepositoryError> {
        for (idx, definition) in item.definitions().iter().enumerate() {
            let def_id = Uuid::new_v4();
            sqlx::query!(
                r#"
                INSERT INTO vocabulary_definitions (
                    id, item_id, part_of_speech, meaning, meaning_translation,
                    domain, register, display_order
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                def_id,
                item.id().as_uuid(),
                entry.part_of_speech().to_string(),
                definition.text(),
                None::<String>, // meaning_translation
                entry.domain().to_string(),
                entry.register().to_string(),
                i32::try_from(idx).map_err(|_| RepositoryError::OperationFailed(
                    "Index conversion failed".to_string()
                ))?
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

            // 各定義に関連するデータを保存
            self.save_definition_related_data(tx, item, def_id).await?;
        }
        Ok(())
    }

    /// 単一定義の関連データを保存
    async fn save_definition_related_data(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        item: &VocabularyItem,
        def_id: Uuid,
    ) -> Result<(), RepositoryError> {
        // examples を保存
        for (ex_idx, example) in item.examples().iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO vocabulary_examples (
                    id, definition_id, example_text, example_translation, display_order
                ) VALUES ($1, $2, $3, $4, $5)
                "#,
                Uuid::new_v4(),
                def_id,
                example.sentence(),
                example.translation(),
                i32::try_from(ex_idx).map_err(|_| RepositoryError::OperationFailed(
                    "Index conversion failed".to_string()
                ))?
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;
        }

        // synonyms を保存
        for (syn_idx, synonym) in item.synonyms().iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO vocabulary_synonyms (
                    definition_id, synonym, display_order
                ) VALUES ($1, $2, $3)
                ON CONFLICT (definition_id, synonym) DO NOTHING
                "#,
                def_id,
                synonym,
                i32::try_from(syn_idx).map_err(|_| RepositoryError::OperationFailed(
                    "Index conversion failed".to_string()
                ))?
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;
        }

        // antonyms を保存
        for (ant_idx, antonym) in item.antonyms().iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO vocabulary_antonyms (
                    definition_id, antonym, display_order
                ) VALUES ($1, $2, $3)
                ON CONFLICT (definition_id, antonym) DO NOTHING
                "#,
                def_id,
                antonym,
                i32::try_from(ant_idx).map_err(|_| RepositoryError::OperationFailed(
                    "Index conversion failed".to_string()
                ))?
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;
        }

        // collocations を保存
        for (col_idx, collocation) in item.collocations().iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO vocabulary_collocations (
                    id, definition_id, collocation_type, pattern, example, display_order
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                Uuid::new_v4(),
                def_id,
                "verb_noun", // TODO: collocation_type を実装
                collocation.pattern(),
                collocation
                    .examples()
                    .first()
                    .map(std::string::String::as_str),
                i32::try_from(col_idx).map_err(|_| RepositoryError::OperationFailed(
                    "Index conversion failed".to_string()
                ))?
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;
        }

        Ok(())
    }

    /// `VocabularyEntry` を構築するヘルパー関数
    fn build_vocabulary_entry(params: EntryBuildParams) -> VocabularyEntry {
        VocabularyEntry::from_database(
            EntryId::from(params.entry_id),
            params.spelling,
            params.meaning,
            params.part_of_speech,
            params.cefr_level,
            params.register,
            params.domain,
            1, // version
            params.created_at,
            params.last_modified_at,
            params.deleted_at,
        )
    }

    /// `VocabularyItem` を構築するヘルパー関数
    fn build_vocabulary_item(params: ItemBuildParams) -> VocabularyItem {
        VocabularyItem::from_database(
            params.item_id,
            params.entry,
            params.definitions,
            params.examples,
            params.collocations,
            params.pronunciation,
            params.synonyms,
            params.antonyms,
            std::collections::HashMap::new(), // TODO: metadata
            params.version,
            params.created_at,
            params.last_modified_at,
            params.deleted_at,
        )
    }
}

#[async_trait]
impl VocabularyItemRepository for Repository {
    type Error = RepositoryError;

    async fn save(&self, item: &VocabularyItem) -> Result<(), Self::Error> {
        // トランザクション内で複数テーブルに保存
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        // 1. vocabulary_entries テーブルに保存または更新
        let entry = item.entry();
        let entry_id = self.save_or_get_entry(&mut tx, entry).await?;

        // 2. vocabulary_items テーブルに保存
        self.save_vocabulary_item(&mut tx, item, entry_id).await?;

        // 3. 定義と関連データを保存
        self.save_definitions_and_related(&mut tx, item, entry)
            .await?;

        tx.commit()
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &ItemId) -> Result<Option<VocabularyItem>, Self::Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        // vocabulary_items と関連テーブルを結合して取得
        let item_row = sqlx::query!(
            r#"
            SELECT 
                vi.id, vi.entry_id, vi.spelling, vi.disambiguation,
                vi.pronunciation, vi.phonetic_respelling, vi.audio_url,
                vi.register, vi.cefr_level, vi.status,
                vi.created_by_type, vi.created_by_id,
                vi.created_at, vi.last_modified_at, vi.last_modified_by,
                vi.version, vi.deleted_at,
                ve.id as entry_id_2, ve.spelling as entry_spelling
            FROM vocabulary_items vi
            JOIN vocabulary_entries ve ON vi.entry_id = ve.id
            WHERE vi.id = $1 AND vi.deleted_at IS NULL
            "#,
            id.as_uuid()
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        let Some(item_data) = item_row else {
            return Ok(None);
        };

        // ヘルパー関数を使って定義と関連データを取得
        let (
            definitions,
            examples,
            synonyms,
            antonyms,
            collocations,
            part_of_speech,
            domain,
            definition_meaning,
        ) = self
            .fetch_definitions_with_related(&mut tx, id.as_uuid())
            .await?;

        // VocabularyEntry を構築
        let cefr_level = item_data
            .cefr_level
            .as_ref()
            .and_then(|l| CefrLevel::from_str(l));
        let register = item_data
            .register
            .as_ref()
            .and_then(|r| Register::from_str(r).ok())
            .unwrap_or(Register::Neutral);
        let domain = domain.unwrap_or(Domain::General);
        let meaning = definition_meaning.unwrap_or_else(String::new);

        let entry = Self::build_vocabulary_entry(EntryBuildParams {
            entry_id: item_data.entry_id,
            spelling: item_data.spelling.clone(),
            meaning,
            part_of_speech,
            cefr_level,
            register,
            domain,
            created_at: item_data.created_at,
            last_modified_at: item_data.last_modified_at,
            deleted_at: item_data.deleted_at,
        });

        // VocabularyItem を構築
        let vocabulary_item = Self::build_vocabulary_item(ItemBuildParams {
            item_id: *id,
            entry,
            definitions,
            examples,
            collocations,
            pronunciation: item_data.pronunciation,
            synonyms,
            antonyms,
            version: u64::try_from(item_data.version).unwrap_or(0),
            created_at: item_data.created_at,
            last_modified_at: item_data.last_modified_at,
            deleted_at: item_data.deleted_at,
        });

        tx.commit()
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        Ok(Some(vocabulary_item))
    }

    async fn find_by_word(&self, word: &str) -> Result<Option<VocabularyItem>, Self::Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        // vocabulary_items と関連テーブルを結合して取得
        let item_row = sqlx::query!(
            r#"
            SELECT 
                vi.id, vi.entry_id, vi.spelling, vi.disambiguation,
                vi.pronunciation, vi.phonetic_respelling, vi.audio_url,
                vi.register, vi.cefr_level, vi.status,
                vi.created_by_type, vi.created_by_id,
                vi.created_at, vi.last_modified_at, vi.last_modified_by,
                vi.version, vi.deleted_at,
                ve.id as entry_id_2, ve.spelling as entry_spelling
            FROM vocabulary_items vi
            JOIN vocabulary_entries ve ON vi.entry_id = ve.id
            WHERE vi.spelling = $1 AND vi.deleted_at IS NULL
            ORDER BY vi.created_at DESC
            LIMIT 1
            "#,
            word
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        let Some(item_data) = item_row else {
            return Ok(None);
        };

        let item_id = ItemId::from(item_data.id);

        // ヘルパー関数を使って定義と関連データを取得
        let (
            definitions,
            examples,
            synonyms,
            antonyms,
            collocations,
            part_of_speech,
            domain,
            definition_meaning,
        ) = self
            .fetch_definitions_with_related(&mut tx, item_id.as_uuid())
            .await?;

        // VocabularyEntry を構築
        let cefr_level = item_data
            .cefr_level
            .as_ref()
            .and_then(|l| CefrLevel::from_str(l));
        let register = item_data
            .register
            .as_ref()
            .and_then(|r| Register::from_str(r).ok())
            .unwrap_or(Register::Neutral);
        let domain = domain.unwrap_or(Domain::General);
        let meaning = definition_meaning.unwrap_or_else(String::new);

        let entry = Self::build_vocabulary_entry(EntryBuildParams {
            entry_id: item_data.entry_id,
            spelling: item_data.spelling.clone(),
            meaning,
            part_of_speech,
            cefr_level,
            register,
            domain,
            created_at: item_data.created_at,
            last_modified_at: item_data.last_modified_at,
            deleted_at: item_data.deleted_at,
        });

        // VocabularyItem を構築
        let vocabulary_item = Self::build_vocabulary_item(ItemBuildParams {
            item_id,
            entry,
            definitions,
            examples,
            collocations,
            pronunciation: item_data.pronunciation,
            synonyms,
            antonyms,
            version: u64::try_from(item_data.version).unwrap_or(0),
            created_at: item_data.created_at,
            last_modified_at: item_data.last_modified_at,
            deleted_at: item_data.deleted_at,
        });

        tx.commit()
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        Ok(Some(vocabulary_item))
    }

    async fn soft_delete(&self, id: &ItemId) -> Result<(), Self::Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        // vocabulary_items テーブルの deleted_at を更新
        let result = sqlx::query!(
            r#"
            UPDATE vocabulary_items
            SET deleted_at = $1
            WHERE id = $2 AND deleted_at IS NULL
            "#,
            chrono::Utc::now(),
            id.as_uuid()
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(id.to_string()));
        }

        tx.commit()
            .await
            .map_err(|e| RepositoryError::OperationFailed(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use domain_events::CefrLevel;

    use super::*;
    use crate::domain::value_objects::{
        domain::Domain,
        part_of_speech::{NounType, PartOfSpeech},
        register::Register,
    };

    #[tokio::test]
    async fn repository_should_implement_trait() {
        // 接続が失敗する場合はテストをスキップ
        let Ok(pool) = PgPool::connect("postgresql://test").await else {
            eprintln!("Database connection failed, skipping test");
            return;
        };

        let repo = Repository::new(pool);

        // トレイトが実装されていることを確認
        let item = VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            Some(CefrLevel::B1),
            Register::Neutral,
            Domain::General,
            vec!["test definition"],
        )
        .unwrap();

        // メソッドが呼び出せることを確認
        let _result = repo.save(&item).await;
        let _result = repo.find_by_id(item.id()).await;
    }
}
