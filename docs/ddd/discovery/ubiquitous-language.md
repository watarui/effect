# ユビキタス言語辞書

## 概要

このドキュメントでは、Effect プロジェクトで使用する共通言語を定義します。
すべてのステークホルダー（開発者、ドメインエキスパート）が同じ意味で用語を使用することを保証します。

## 語彙管理ドメイン（Vocabulary Management Domain）

### 項目（Item）

**定義**: 学習対象となる言語的単位。単一の単語、フレーズ、熟語、慣用表現を含む。

**構成要素**:

- **Spelling**: 綴り
- **Pronunciation**: 発音（音声ファイルURL等）
- **Phonetic Respelling**: 発音記号（IPA形式）
- **Definitions**: 意味のリスト（番号付き）
- **Parts of Speech**: 品詞情報
  - **Noun**（名詞）
    - Countable Noun（可算名詞）
    - Uncountable Noun（不可算名詞）
    - Countable and Uncountable Noun（可算・不可算名詞）
  - Article（冠詞）
  - Pronoun（代名詞）
  - **Verb**（動詞）
    - Transitive Verb（他動詞）
    - Intransitive Verb（自動詞）
    - Auxiliary Verb（助動詞）
  - Adjective（形容詞）
  - Adverb（副詞）
  - Preposition（前置詞）
  - Conjunction（接続詞）
  - Interjection（間投詞）
- **Example Sentences**: 各意味に対する例文（最低1つ）
- **Synonyms**: 類義語
- **Antonyms**: 対義語
- **Derived Words**: 派生語
- **Related Words**: 関連語
- **Dictionary Links**: 外部辞書へのリンク
  - Oxford Learner's Dictionaries
  - Cambridge Dictionary

**実装での表現**:

```rust
pub struct VocabularyItem {
    id: VocabularyId,
    spelling: String,
    pronunciation: Option<String>,
    phonetic_respelling: String,
    definitions: Vec<Definition>,
    parts_of_speech: Vec<PartOfSpeech>,
    example_sentences: Vec<ExampleSentence>,
    synonyms: Vec<String>,
    antonyms: Vec<String>,
    derived_words: Vec<String>,
    related_words: Vec<String>,
    dictionary_links: DictionaryLinks,
}
```

### グローバル辞書（Global Dictionary）

**定義**: アプリケーション全体で共有される語句情報のリポジトリ。
全ユーザーがアクセス可能で、AI によって生成された語句情報を格納する。

**特徴**:

- 語句の重複を防ぐ（同じ spelling は一度だけ登録）
- AI生成情報の一貫性を保証
- バージョン管理（将来的な情報更新に対応）

### 領域（Domain）

**定義**: 語句が関連する言語技能の分類。

**種類**:

- **R (Reading)**: 読解に重要な語句
- **W (Writing)**: 作文に重要な語句
- **L (Listening)**: 聞き取りに重要な語句
- **S (Speaking)**: 会話に重要な語句

**注**: 一つの語句は複数の領域に属することができる

## 学習ドメイン（Learning Domain）

### 学習リスト（Learning List）

**定義**: 個々のユーザーが学習対象として選択した語句の集合。

**特徴**:

- ユーザーごとに独立
- グローバル辞書の語句を参照
- 学習進捗情報を含む

### 学習セッション（Learning Session）

**定義**: 25分のポモドーロ単位で行われる学習活動の単位。

**種類**:

- **復習セッション**: 既習語句のテスト
- **新規学習セッション**: 新しい語句の学習
- **混合セッション**: 復習と新規の組み合わせ

**実装での表現**:

```rust
pub struct LearningSession {
    id: SessionId,
    user_id: UserId,
    session_type: SessionType,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    target_duration: Duration, // 25 minutes
}
```

### テスト問題（Test Question）

**定義**: 学習セッション中に提示される個別の問題。

**構成要素**:

- 語句の表示
- 制限時間（30秒）
- 反応選択肢（わかる/わからない/曖昧）

### 反応（Response）

**定義**: テスト問題に対するユーザーの回答。

**属性**:

- **反応タイプ**: わかる（Understood）/わからない（NotUnderstood）/曖昧（Ambiguous）
- **反応時間**: ミリ秒単位
- **タイムスタンプ**: 回答日時

### 「覚えた」状態（Mastered State / Sense of Mastery）

**定義**: 語句が十分に定着したと判定される状態。

**判定基準**:

- 「わかる」を3秒以内に選択
- 過去3回連続で上記条件を満たす

### 復習間隔（Review Interval）

**定義**: 「覚えた」項目を再度テストに出すまでの期間。

**アルゴリズム**:

- 初回: 7日後
- 2回目以降: 前回間隔 × 2.5

## ユーザードメイン（User Domain）

### ユーザー（User）

**定義**: システムを利用する個人。学習者。

**属性**:

- Googleアカウント情報
- 学習コース
- 目標スコア

### 学習コース（Learning Course）

**定義**: 特定の試験や目的に向けた学習カリキュラム。

**例**:

- IELTSコース
- TOEFLコース
- ビジネス英語コース

### 目標スコア（Target Score）

**定義**: 学習者が目指す試験のスコア。

**例**: IELTS 7.0

## 進捗ドメイン（Progress Domain）

### 学習統計（Learning Statistics）

**定義**: ユーザーの学習活動に関する集計データ。

**含まれる情報**:

- 総学習語句数
- 「覚えた」語句数
- 領域別進捗（R/W/L/S）
- 正答率
- 平均反応時間
- 学習時間

### IELTSスコア推定（IELTS Score Estimation）

**定義**: 学習統計と領域別進捗から推定される現在のIELTSスコア。

**計算要素**:

- 語彙カバー率（IELTS頻出語句に対する習得率）
- 領域別習熟度
- 正答率と反応時間

## AI統合ドメイン（AI Integration Domain）

### AI深掘りチャット（AI Deep Dive Chat）

**定義**: 特定の語句について、AIとの対話を通じて詳細な理解を深める機能。

**特徴**:

- 語句ごとに独立したセッション
- 文脈に応じた説明
- 使用例の提供

### AIテストカスタマイズ（AI Test Customization）

**定義**: AIへの指示によってテスト内容をカスタマイズする機能。

**例**: "Speaking語句多めで" → Speaking領域の語句を優先的に出題

## ドメインイベント

最新の設計では、各コンテキストのイベントを DomainEvent wrapper で管理します：

```rust
pub enum DomainEvent {
    Learning(LearningEvent),
    Algorithm(LearningAlgorithmEvent),
    Vocabulary(VocabularyEvent),
    AI(AIIntegrationEvent),
    User(UserEvent),
}
```

### 語彙管理イベント（VocabularyEvent）

- エントリーが作成された（EntryCreated）
- 項目が作成された（ItemCreated）
- AI情報生成がリクエストされた（AIGenerationRequested）
- AI情報が生成された（AIInfoGenerated）

### 学習イベント（LearningEvent）

- セッションが開始された（SessionStarted）
- 正誤が判定された（CorrectnessJudged）
- セッションが完了した（SessionCompleted）
- 項目マスタリーが更新された（ItemMasteryUpdated）

### 学習アルゴリズムイベント（LearningAlgorithmEvent）

- 復習スケジュールが更新された（ReviewScheduleUpdated）
- 統計が更新された（StatisticsUpdated）

### 進捗イベント

進捗コンテキストは純粋な Read Model のため、独自のイベントは発行しません。
他のコンテキストからのイベントを受信して集計します。

### AI統合イベント（AIIntegrationEvent）

- タスクが作成された（TaskCreated）
- タスクが開始された（TaskStarted）
- タスクが完了した（TaskCompleted）
- タスクが失敗した（TaskFailed）

### ユーザーイベント（UserEvent）

- アカウントが作成された（AccountCreated）
- アカウントが削除された（AccountDeleted）

## 使用上の注意

1. **一貫性**: 同じ概念には必ず同じ用語を使用する
2. **コード内**: 変数名、クラス名、メソッド名に反映する
3. **ドキュメント**: すべての文書で統一する
4. **日本語/英語**: ドメインエキスパートとの会話では日本語、コードでは英語を使用

## 更新履歴

- 2025-07-27: ドメインエキスパートとの対話に基づき作成
