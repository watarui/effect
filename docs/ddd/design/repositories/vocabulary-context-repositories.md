# Vocabulary Context - リポジトリインターフェース

## 概要

Vocabulary Context には 2 つの集約が存在し、それぞれに対応するリポジトリを定義します：

- `VocabularyEntry`：語彙エントリ（スペリング単位の管理）
- `VocabularyItem`：語彙項目（意味や品詞単位の詳細情報）

## VocabularyEntryRepository

語彙エントリの永続化を担当するリポジトリです。

### インターフェース定義

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// 語彙エントリのリポジトリ
#[async_trait]
pub trait VocabularyEntryRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    // ===== 基本的な CRUD 操作 =====

    /// ID でエントリを取得
    async fn find_by_id(&self, id: &EntryId) -> Result<Option<VocabularyEntry>, Self::Error>;

    /// スペリングでエントリを取得
    async fn find_by_spelling(&self, spelling: &str) -> Result<Option<VocabularyEntry>, Self::Error>;

    /// エントリを保存（新規作成または更新）
    async fn save(&self, entry: &VocabularyEntry) -> Result<(), Self::Error>;

    /// エントリを削除（論理削除推奨）
    async fn delete(&self, id: &EntryId) -> Result<(), Self::Error>;

    // ===== 検索関連のクエリ =====

    /// キーワードでエントリを検索
    async fn search(
        &self,
        keyword: &str,
        limit: u32,
    ) -> Result<Vec<VocabularyEntry>, Self::Error>;

    /// プレフィックス検索（オートコンプリート用）
    async fn find_by_prefix(
        &self,
        prefix: &str,
        limit: u32,
    ) -> Result<Vec<VocabularyEntry>, Self::Error>;

    /// 全エントリをページネーションで取得
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
    ) -> Result<Page<VocabularyEntry>, Self::Error>;

    // ===== AI 生成関連 =====

    /// AI 生成失敗のエントリを取得（リトライ用）
    async fn find_failed_generation_entries(
        &self,
        max_retries: u32,
        limit: u32,
    ) -> Result<Vec<VocabularyEntry>, Self::Error>;

    // ===== 統計関連 =====

    /// エントリ総数を取得
    async fn count_total(&self) -> Result<u64, Self::Error>;

    /// AI 生成済みエントリ数を取得
    async fn count_generated(&self) -> Result<u64, Self::Error>;
}
```

### 使用例

```rust
// アプリケーションサービスでの使用例
pub struct AddVocabularyUseCase<R: VocabularyEntryRepository> {
    repository: Arc<R>,
    event_bus: Arc<dyn EventBus>,
}

impl<R: VocabularyEntryRepository> AddVocabularyUseCase<R> {
    pub async fn execute(&self, spelling: String) -> Result<EntryId> {
        // 既存チェック
        if let Some(existing) = self.repository.find_by_spelling(&spelling).await? {
            return Ok(existing.id().clone());
        }

        // 新規作成
        let entry = VocabularyEntry::new(spelling)?;
        let entry_id = entry.id().clone();

        // 保存
        self.repository.save(&entry).await?;

        // イベント発行（非同期 AI 生成のトリガー）
        self.event_bus.publish(VocabularyEntryCreated {
            entry_id: entry_id.clone(),
            spelling: entry.spelling().to_string(),
        }).await?;

        Ok(entry_id)
    }
}

// バックグラウンドワーカーでの AI コンテンツ生成
pub struct AiContentGenerationWorker<R: VocabularyItemRepository> {
    item_repository: Arc<R>,
    ai_service: Arc<dyn AiService>,
}

impl<R: VocabularyItemRepository> AiContentGenerationWorker<R> {
    pub async fn process_ai_content_generation(&self) -> Result<()> {
        // 中身が空の項目を取得
        let empty_items = self.item_repository
            .find_items_for_ai_generation(100)
            .await?;

        for item in empty_items {
            match self.generate_content_for_item(&item).await {
                Ok(_) => info!("Generated content for item: {}", item.id()),
                Err(e) => error!("Failed to generate content: {}", e),
            }
        }

        Ok(())
    }

    async fn generate_content_for_item(&self, item: &VocabularyItem) -> Result<()> {
        // AI サービスを使用してコンテンツを生成
        let content = self.ai_service.generate_vocabulary_content(
            item.entry_spelling(),
            item.disambiguation(),
        ).await?;

        // 項目を更新
        let mut updated_item = item.clone();
        updated_item.update_with_ai_content(content)?;
        self.item_repository.save(&updated_item).await?;

        Ok(())
    }
}

// ユーザーによる手動再生成
pub struct RegenerateVocabularyUseCase<R: VocabularyItemRepository> {
    repository: Arc<R>,
    ai_service: Arc<dyn AiService>,
}

impl<R: VocabularyItemRepository> RegenerateVocabularyUseCase<R> {
    pub async fn execute(&self, item_id: ItemId, user_id: UserId) -> Result<()> {
        // 項目を取得
        let item = self.repository.find_by_id(&item_id).await?
            .ok_or(DomainError::NotFound)?;

        // 充実度に関係なく AI で再生成
        let new_content = self.ai_service.generate_vocabulary_content(
            item.entry_spelling(),
            item.disambiguation(),
        ).await?;

        // 更新（ユーザーによる明示的な要求なので上書き）
        let mut updated_item = item;
        updated_item.update_with_ai_content(new_content)?;
        updated_item.set_last_modified_by(ModifiedBy::User(user_id));

        self.repository.save(&updated_item).await?;

        Ok(())
    }
}
```

## VocabularyItemRepository

語彙項目の詳細情報を管理するリポジトリです。

### インターフェース定義

```rust
/// 語彙項目のリポジトリ
#[async_trait]
pub trait VocabularyItemRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    // ===== 基本的な CRUD 操作 =====

    /// ID で項目を取得
    async fn find_by_id(&self, id: &ItemId) -> Result<Option<VocabularyItem>, Self::Error>;

    /// エントリ ID で全項目を取得
    async fn find_by_entry_id(
        &self,
        entry_id: &EntryId,
    ) -> Result<Vec<VocabularyItem>, Self::Error>;

    /// 項目を保存（新規作成または更新）
    async fn save(&self, item: &VocabularyItem) -> Result<(), Self::Error>;

    /// 複数項目を一括保存（AI 生成結果の保存用）
    async fn save_batch(&self, items: &[VocabularyItem]) -> Result<(), Self::Error>;

    /// 項目を削除
    async fn delete(&self, id: &ItemId) -> Result<(), Self::Error>;

    // ===== フィルタリング関連 =====

    /// 品詞でフィルタリング
    async fn find_by_part_of_speech(
        &self,
        pos: &PartOfSpeech,
        page_request: &PageRequest,
    ) -> Result<Page<VocabularyItem>, Self::Error>;

    /// CEFR レベルでフィルタリング
    async fn find_by_cefr_level(
        &self,
        level: &CefrLevel,
        page_request: &PageRequest,
    ) -> Result<Page<VocabularyItem>, Self::Error>;

    /// スキルタイプでフィルタリング
    async fn find_by_skill_type(
        &self,
        skill: &SkillType,
        page_request: &PageRequest,
    ) -> Result<Page<VocabularyItem>, Self::Error>;

    // ===== 学習用クエリ =====

    /// ランダムに項目を取得（テスト出題用）
    async fn find_random(
        &self,
        count: u32,
        filter: Option<ItemFilter>,
    ) -> Result<Vec<VocabularyItem>, Self::Error>;

    /// 優先度順に項目を取得
    async fn find_by_priority(
        &self,
        limit: u32,
    ) -> Result<Vec<VocabularyItem>, Self::Error>;

    // ===== AI 関連 =====

    /// AI コンテンツ生成が必要な項目を取得（中身が完全に空の項目）
    async fn find_items_for_ai_generation(
        &self,
        limit: u32,
    ) -> Result<Vec<VocabularyItem>, Self::Error>;

    /// 画像生成が必要な項目を取得
    async fn find_items_needing_images(
        &self,
        limit: u32,
    ) -> Result<Vec<VocabularyItem>, Self::Error>;

    // ===== 統計関連 =====

    /// 品詞別の項目数を取得
    async fn count_by_part_of_speech(&self) -> Result<HashMap<PartOfSpeech, u64>, Self::Error>;

    /// CEFR レベル別の項目数を取得
    async fn count_by_cefr_level(&self) -> Result<HashMap<CefrLevel, u64>, Self::Error>;
}
```

### 補助的な型定義

```rust
/// 項目フィルター
#[derive(Debug, Clone)]
pub struct ItemFilter {
    pub part_of_speech: Option<Vec<PartOfSpeech>>,
    pub cefr_levels: Option<Vec<CefrLevel>>,
    pub skill_types: Option<Vec<SkillType>>,
    pub has_image: Option<bool>,
    pub exclude_ids: Option<Vec<ItemId>>,
}

/// 品詞
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Preposition,
    Conjunction,
    Pronoun,
    Interjection,
    Article,
    Other(String),
}

/// CEFR レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CefrLevel {
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
}

/// スキルタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    Reading,
    Writing,
    Listening,
    Speaking,
    Grammar,
    Vocabulary,
}
```

## 実装上の考慮事項

### 1. パフォーマンス最適化

```rust
// インデックスの推奨
// VocabularyEntry
// - (spelling) - UNIQUE, スペリング検索用
// - (spelling gin_trgm_ops) - 部分一致検索用（PostgreSQL）
// - (created_at) - ソート用
// - (generation_attempts) - リトライ管理用

// VocabularyItem
// - (entry_id) - エントリとの関連
// - (part_of_speech) - 品詞フィルタ用
// - (cefr_level) - レベルフィルタ用
// - (priority DESC) - 優先度順取得用
// - (entry_id, disambiguation) - UNIQUE, 重複防止
// - (definition IS NULL AND pronunciation IS NULL) - AI 生成対象の効率的な検索用
```

### 2. トランザクション境界

```rust
// AI 生成結果の保存例
pub async fn save_ai_generation_result(
    entry_repo: &dyn VocabularyEntryRepository,
    item_repo: &dyn VocabularyItemRepository,
    entry_id: EntryId,
    generated_items: Vec<GeneratedItem>,
) -> Result<()> {
    // 1. エントリのステータス更新（トランザクション 1）
    let mut entry = entry_repo.find_by_id(&entry_id).await?
        .ok_or(DomainError::NotFound)?;
    entry.mark_as_generated();
    entry_repo.save(&entry).await?;

    // 2. 項目の一括保存（トランザクション 2）
    let items: Vec<VocabularyItem> = generated_items
        .into_iter()
        .map(|g| VocabularyItem::from_generated(entry_id.clone(), g))
        .collect::<Result<Vec<_>>>()?;

    item_repo.save_batch(&items).await?;

    Ok(())
}
```

### 3. エラーハンドリング

```rust
/// Vocabulary Context 固有のリポジトリエラー
#[derive(Debug, thiserror::Error)]
pub enum VocabularyRepositoryError {
    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Invalid disambiguation: entry {0} already has item with disambiguation {1}")]
    DuplicateDisambiguation(EntryId, String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Search error: {0}")]
    SearchError(String),
}
```

### 4. AI 生成の非同期処理パターン

```rust
// AI 生成のトリガーパターン
pub enum AiGenerationTrigger {
    // 自動トリガー
    NewEntryCreated,        // 新規エントリ作成時
    EmptyItemDetected,      // 空の項目を検出時

    // 手動トリガー
    UserRequested {         // ユーザーによる明示的な要求
        item_id: ItemId,
        force_regenerate: bool,  // 既存内容を上書きするか
    },
}

// イベント駆動による非同期処理
impl VocabularyDomainEventHandler {
    pub async fn handle_vocabulary_entry_created(&self, event: VocabularyEntryCreated) {
        // 空の VocabularyItem を作成
        let item = VocabularyItem::new_empty(event.entry_id);
        self.item_repository.save(&item).await?;

        // AI 生成タスクをキューに追加
        self.task_queue.enqueue(AiGenerationTask {
            item_id: item.id(),
            trigger: AiGenerationTrigger::NewEntryCreated,
        }).await?;
    }
}
```

## AI 生成の設計思想

### 基本方針

1. **自動生成の制限**

   - 中身が完全に空の項目のみ自動で AI 生成
   - 少しでも手動編集された項目は自動生成から除外
   - ユーザーの作業を尊重し、勝手に上書きしない

2. **手動再生成の自由度**

   - ユーザーは明示的なボタン操作でいつでも AI 再生成可能
   - 充実度に関係なく、ユーザーの意思を優先

3. **非同期処理のメリット**
   - 大量インポート時のユーザー体験向上
   - AI サービスの遅延やエラーの影響を最小化
   - システム全体のスケーラビリティ向上

### 実装のポイント

```rust
impl VocabularyItem {
    /// 自動 AI 生成の対象かどうかを判定
    pub fn is_eligible_for_auto_generation(&self) -> bool {
        // すべてのコンテンツフィールドが空の場合のみ true
        self.definition.is_none()
            && self.examples.is_empty()
            && self.pronunciation.is_none()
            && self.synonyms.is_empty()
            && self.antonyms.is_empty()
    }

    /// AI で生成されたコンテンツで更新
    pub fn update_with_ai_content(&mut self, content: AiGeneratedContent) -> Result<()> {
        // 既存の内容がある場合の処理はユースケース層で判断
        self.definition = Some(content.definition);
        self.examples = content.examples;
        self.pronunciation = Some(content.pronunciation);
        self.last_ai_generated_at = Some(Utc::now());
        Ok(())
    }
}
```

## 更新履歴

- 2025-07-28: 初版作成（DDD 原則に基づいた設計）
- 2025-07-29: AI 生成機能の設計を追加（非同期処理、自動/手動生成の区別）
