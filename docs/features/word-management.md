# 単語管理機能仕様

> **ステータス**: 📝 仮案 - 協調編集や AI 支援機能は将来構想を含む

## 概要

Effect の単語管理機能は、協調的な単語学習環境を提供します。ユーザーは単語の登録・編集を行い、AI 支援により効率的に学習情報を充実させることができます。

## 単語データモデル

### Word エンティティ

```rust
pub struct Word {
    pub id: Uuid,
    pub text: String,
    pub phonetic_ipa: String,  // IPA 発音記号
    pub phonetic_spelling: Option<String>,  // カタカナ表記など
    pub audio_source: AudioSource,
    pub cefr_level: CefrLevel,  // A1-C2
    pub difficulty: u8,  // 1-10
    pub categories: Vec<TestCategory>,  // 複数選択可能
    pub tags: Vec<String>,
    pub image_url: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub version: u32,  // 楽観的ロック用
}

pub enum AudioSource {
    GoogleTts {
        voice_id: String,
    },
    Recorded {
        url: String,
        source: String,
    },
}

pub enum TestCategory {
    IELTS,
    TOEIC,
    TOEFL,
    GRE,
    Academic,
    Business,
    General,
}

pub enum CefrLevel {
    A1,  // レベル 1-2
    A2,  // レベル 3-4
    B1,  // レベル 5-6
    B2,  // レベル 7-8
    C1,  // レベル 9
    C2,  // レベル 10
}
```

### WordMeaning エンティティ

```rust
pub struct WordMeaning {
    pub id: Uuid,
    pub word_id: Uuid,
    pub meaning: String,
    pub part_of_speech: PartOfSpeech,
    pub usage_notes: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

pub enum PartOfSpeech {
    Noun {
        countable: bool,
        uncountable: bool,
    },
    Verb {
        transitive: bool,
        intransitive: bool,
        auxiliary: bool,
    },
    Adjective,
    Adverb,
    Preposition,
    Conjunction,
    Pronoun,
    Interjection,
}
```

### Example エンティティ

```rust
pub struct Example {
    pub id: Uuid,
    pub meaning_id: Uuid,
    pub sentence: String,
    pub translation: String,
    pub context: Option<String>,  // ビジネス、日常会話など
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}
```

### WordRelation エンティティ

```rust
pub struct WordRelation {
    pub id: Uuid,
    pub word_id: Uuid,
    pub related_word_id: Uuid,
    pub relation_type: RelationType,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

pub enum RelationType {
    Synonym,           // 同義語
    Antonym,          // 反義語
    Collocation,      // コロケーション
    SpellingVariant,  // color ↔ colour
    RegionalVariant,  // elevator ↔ lift
    Etymology,        // 語源関係
}
```

## 単語登録機能

### 手動登録

基本的な単語情報を手動で入力：

```rust
pub struct CreateWordInput {
    pub text: String,
    pub meaning: String,
    pub categories: Vec<TestCategory>,
    pub tags: Vec<String>,
}
```

### AI 支援登録

単語入力時に AI が自動生成：

```rust
pub struct AiGeneratedContent {
    pub phonetic_ipa: String,
    pub example_sentences: Vec<ExampleSuggestion>,
    pub collocations: Vec<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    pub related_words: Vec<String>,
    pub image_suggestions: Vec<ImageSuggestion>,
}
```

### 音声生成

Google Text-to-Speech API を使用：

```rust
pub async fn generate_audio(word: &str) -> Result<AudioSource> {
    let client = GoogleTtsClient::new();
    let voice_id = "en-US-Standard-A";

    client.synthesize_speech(word, voice_id).await?;

    Ok(AudioSource::GoogleTts { voice_id })
}
```

## 協調編集機能

### 楽観的ロック

```rust
pub struct UpdateWordCommand {
    pub word_id: Uuid,
    pub version: u32,  // 現在のバージョン
    pub changes: HashMap<String, Value>,
    pub updated_by: Uuid,
}

pub enum UpdateResult {
    Success {
        new_version: u32,
    },
    VersionConflict {
        current_version: u32,
        your_changes: HashMap<String, Value>,
        latest_changes: HashMap<String, Value>,
    },
}
```

### 編集履歴

```rust
pub struct EditHistory {
    pub id: Uuid,
    pub word_id: Uuid,
    pub version: u32,
    pub editor: Uuid,
    pub changes: Vec<FieldChange>,
    pub timestamp: DateTime<Utc>,
}

pub struct FieldChange {
    pub field_name: String,
    pub old_value: Option<Value>,
    pub new_value: Value,
}
```

## お気に入り機能

```rust
pub struct UserFavorite {
    pub user_id: Uuid,
    pub word_id: Uuid,
    pub favorited_at: DateTime<Utc>,
    pub notes: Option<String>,  // 個人的なメモ
}
```

## 権限管理

- ログインユーザーは誰でも単語の追加・編集が可能
- すべての変更は編集者情報と共に記録
- 不適切な編集は管理者がロールバック可能

## API エンドポイント

### GraphQL スキーマ

```graphql
type Word {
    id: ID!
    text: String!
    phoneticIpa: String!
    meanings: [WordMeaning!]!
    examples: [Example!]!
    relations: [WordRelation!]!
    editHistory: [EditHistory!]!
    isFavorite: Boolean!
    userProgress: UserWordProgress
}

type Mutation {
    createWord(input: CreateWordInput!): Word!
    updateWord(input: UpdateWordInput!): UpdateResult!
    addWordMeaning(input: AddWordMeaningInput!): WordMeaning!
    addExample(input: AddExampleInput!): Example!
    toggleFavorite(wordId: ID!): Boolean!
}
```

## イベント

- `WordCreated`
- `WordUpdated`
- `WordMeaningAdded`
- `ExampleAdded`
- `WordRelationCreated`
- `WordFavorited`
- `WordUnfavorited`

## 実装の優先順位

### Phase 1（MVP）

- 基本的な単語登録・編集
- Google TTS による音声生成
- お気に入り機能
- 最終更新が勝つ協調編集

### Phase 2

- AI 支援による情報自動生成
- 編集履歴の詳細表示
- スペルバリエーション管理

### Phase 3

- 画像の自動推薦
- 高度な協調編集（差分マージ）
- 貢献度スコアリング
