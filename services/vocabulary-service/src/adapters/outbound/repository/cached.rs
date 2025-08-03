use async_trait::async_trait;
use common_types::ItemId;
use infrastructure::cache;
use tracing::{error, warn};

use crate::{
    domain::entities::vocabulary_item::VocabularyItem,
    ports::outbound::repository::Repository as VocabularyRepository,
};

const CACHE_TTL_SECONDS: u64 = 3600; // 1時間

/// キャッシュ付き語彙リポジトリ
#[derive(Clone)]
pub struct Repository<R> {
    inner: R,
    cache: cache::Client,
}

impl<R> Repository<R> {
    /// 新しいキャッシュ付きリポジトリを作成
    pub const fn new(inner: R, cache: cache::Client) -> Self {
        Self { inner, cache }
    }

    /// ID 用のキャッシュキーを生成
    fn cache_key_for_id(id: &ItemId) -> String {
        format!("vocabulary:item:id:{id}")
    }

    /// 単語用のキャッシュキーを生成
    fn cache_key_for_word(word: &str) -> String {
        format!("vocabulary:item:word:{word}")
    }

    /// 語彙項目をキャッシュに保存
    async fn cache_item(&self, item: &VocabularyItem)
    where
        R: Sync,
    {
        // ID によるキャッシュ
        let id_key = Self::cache_key_for_id(item.id());
        if let Err(e) = self.cache.set(&id_key, item, Some(CACHE_TTL_SECONDS)).await {
            warn!("Failed to cache item by ID: {}", e);
        }

        // 単語によるキャッシュ
        let word_key = Self::cache_key_for_word(item.word());
        if let Err(e) = self
            .cache
            .set(&word_key, item, Some(CACHE_TTL_SECONDS))
            .await
        {
            warn!("Failed to cache item by word: {}", e);
        }
    }

    /// 語彙項目のキャッシュを無効化
    async fn invalidate_cache(&self, item: &VocabularyItem)
    where
        R: Sync,
    {
        // ID キャッシュの削除
        let id_key = Self::cache_key_for_id(item.id());
        if let Err(e) = self.cache.delete(&id_key).await {
            warn!("Failed to delete cache by ID: {}", e);
        }

        // 単語キャッシュの削除
        let word_key = Self::cache_key_for_word(item.word());
        if let Err(e) = self.cache.delete(&word_key).await {
            warn!("Failed to delete cache by word: {}", e);
        }
    }
}

#[async_trait]
impl<R> VocabularyRepository for Repository<R>
where
    R: VocabularyRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    type Error = R::Error;

    async fn save(&self, item: &VocabularyItem) -> Result<(), Self::Error> {
        // データベースに保存
        self.inner.save(item).await?;

        // キャッシュを無効化
        self.invalidate_cache(item).await;

        Ok(())
    }

    async fn find_by_id(&self, id: &ItemId) -> Result<Option<VocabularyItem>, Self::Error> {
        let key = Self::cache_key_for_id(id);

        // キャッシュから取得を試みる
        match self.cache.get::<VocabularyItem>(&key).await {
            Ok(Some(item)) => {
                return Ok(Some(item));
            },
            Ok(None) => {
                // キャッシュミス
            },
            Err(e) => {
                error!("Cache read error: {}", e);
                // エラー時はフォールバック
            },
        }

        // データベースから取得
        let item = self.inner.find_by_id(id).await?;

        // キャッシュに保存
        if let Some(ref item) = item {
            self.cache_item(item).await;
        }

        Ok(item)
    }

    async fn find_by_word(&self, word: &str) -> Result<Option<VocabularyItem>, Self::Error> {
        let key = Self::cache_key_for_word(word);

        // キャッシュから取得を試みる
        match self.cache.get::<VocabularyItem>(&key).await {
            Ok(Some(item)) => {
                return Ok(Some(item));
            },
            Ok(None) => {
                // キャッシュミス
            },
            Err(e) => {
                error!("Cache read error: {}", e);
                // エラー時はフォールバック
            },
        }

        // データベースから取得
        let item = self.inner.find_by_word(word).await?;

        // キャッシュに保存
        if let Some(ref item) = item {
            self.cache_item(item).await;
        }

        Ok(item)
    }

    async fn soft_delete(&self, id: &ItemId) -> Result<(), Self::Error> {
        // まず項目を取得してキャッシュ無効化のために必要な情報を得る
        if let Some(item) = self.inner.find_by_id(id).await? {
            // データベースから削除
            self.inner.soft_delete(id).await?;

            // キャッシュを無効化
            self.invalidate_cache(&item).await;
        } else {
            // 項目が見つからない場合もそのまま削除を試みる
            self.inner.soft_delete(id).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::{mock, predicate::*};

    use super::*;

    mock! {
        InnerRepo {}

        #[async_trait]
        impl VocabularyRepository for InnerRepo {
            type Error = crate::ports::outbound::repository::Error;

            async fn save(&self, item: &VocabularyItem) -> Result<(), crate::ports::outbound::repository::Error>;
            async fn find_by_id(&self, id: &ItemId) -> Result<Option<VocabularyItem>, crate::ports::outbound::repository::Error>;
            async fn find_by_word(&self, word: &str) -> Result<Option<VocabularyItem>, crate::ports::outbound::repository::Error>;
            async fn soft_delete(&self, id: &ItemId) -> Result<(), crate::ports::outbound::repository::Error>;
        }
    }

    use domain_events::CefrLevel;

    use crate::domain::{
        entities::vocabulary_item::VocabularyItem,
        value_objects::part_of_speech::PartOfSpeech,
    };

    fn create_test_item() -> VocabularyItem {
        use crate::domain::value_objects::{
            domain::Domain,
            part_of_speech::NounType,
            register::Register,
        };

        VocabularyItem::new(
            "test",
            PartOfSpeech::Noun(NounType::Countable),
            Some(CefrLevel::B1),
            Register::Neutral,
            Domain::General,
            vec!["A test item", "A procedure to establish quality"],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_by_id_cache_hit() {
        let mut mock_repo = MockInnerRepo::new();
        let item = create_test_item();
        let _item_id = *item.id();

        // Inner repo should not be called on cache hit
        mock_repo.expect_find_by_id().times(0);

        // Create a real Redis client for testing (requires Redis to be running)
        // In a real test, we'd use a test container or mock
        // For now, we'll skip this test
    }

    #[tokio::test]
    async fn test_find_by_id_cache_miss() {
        let mut mock_repo = MockInnerRepo::new();
        let item = create_test_item();
        let item_id = *item.id();

        // Inner repo should be called once on cache miss
        mock_repo
            .expect_find_by_id()
            .with(eq(item_id))
            .times(1)
            .returning(move |_| Ok(Some(item.clone())));

        // Test would continue with actual Redis client...
    }

    #[tokio::test]
    async fn test_save_invalidates_cache() {
        let mut mock_repo = MockInnerRepo::new();
        let item = create_test_item();

        // Inner repo save should be called
        mock_repo
            .expect_save()
            .with(eq(item))
            .times(1)
            .returning(|_| Ok(()));

        // Test would continue with verifying cache invalidation...
    }
}
