# Vocabulary Context - 集約設計

## 概要

Vocabulary Context は、Effect プロジェクトにおける語彙コンテンツ管理の中核です。全ユーザーが共有するグローバル辞書を管理し、AI と連携して豊富な語彙情報を提供します。

### 主要な設計方針

1. **Wikipedia 方式**: 1つの綴り（spelling）に対して複数の意味（disambiguation）を持つ項目を管理
2. **楽観的ロック + 自動マージ**: 並行編集に対して可能な限り自動マージを試みる
3. **イベントソーシング**: すべての変更を記録し、完全な履歴とバージョン管理を実現
4. **AI との非同期連携**: 項目情報の生成を AI に委譲し、非同期で処理
5. **CQRS アーキテクチャ**: 書き込み（Command）と読み取り（Query）を分離し、それぞれ最適化

### 主要な責務

- グローバル辞書の管理（全ユーザー共有）
- 項目（単語、フレーズ、熟語）の CRUD 操作
- AI を活用した項目情報の生成と管理
- 並行編集の処理と競合解決
- 完全な変更履歴の保持

## 集約の設計

### 1. VocabularyEntry（見出し語）- 軽量な集約

**概念モデル**:

- entry_id: 見出し語の一意識別子
- spelling: 綴り（例: "apple"）
- items: 項目のサマリー情報リスト
- created_at: 作成日時

**ItemSummary（値オブジェクト）**:

- item_id: 項目の識別子
- disambiguation: 意味の区別（例: "(fruit)", "(company)"）
- is_primary: 最も一般的な意味かどうか

### 2. VocabularyItem（語彙項目）- メイン集約ルート

**主要な属性**:

- item_id: 項目の一意識別子
- entry_id: 所属する見出し語の識別子
- spelling: 綴り
- disambiguation: 意味の区別

**詳細情報**:

- pronunciation: 発音記号
- phonetic_respelling: 音声表記
- definitions: 定義のリスト（Definition 値オブジェクトの配列）
- synonyms: 類義語のリスト
- antonyms: 対義語のリスト
- collocations: コロケーション
- register: レジスター（formal, informal など）
- cefr_level: CEFR レベル

**Definition（定義）- 値オブジェクト**:

- definition_id: 定義の識別子
- part_of_speech: 品詞（noun, verb, adjective など）
- meaning: 英語の定義
- meaning_translation: 日本語訳
- domain: 分野（medical, computing, legal など、NULL = general）
- register: この定義特有の使用域（項目レベルを上書き）
- examples: 例文のリスト（Example 値オブジェクトの配列）

**Example（例文）- 値オブジェクト**:

- example_text: 例文
- example_translation: 例文の日本語訳
- source: 出典（オプション）

**メタ情報**:

- created_by: 作成者（ユーザー、システム、インポート）
- created_at: 作成日時
- last_modified_at: 最終更新日時
- last_modified_by: 最終更新者
- version: バージョン番号（楽観的ロック用）
- status: ステータス（Draft, PendingAI, Published）

### 3. FieldChange（フィールド変更）- 値オブジェクト

並行編集の競合解決のために、個別フィールドレベルでの変更を追跡：

- field_path: 変更されたフィールドのパス（例: "definitions[0].meaning"）
- old_value: 変更前の値
- new_value: 変更後の値

## 不変条件

1. **spelling の不変性**: VocabularyItem の spelling は一度設定されたら変更不可
2. **EntryId の整合性**: VocabularyItem は必ず VocabularyEntry に所属する
3. **バージョン管理**: 更新時は必ずバージョンチェックを行う
4. **ステータス遷移**: Draft → PendingAI → Published の順序を守る

## CQRS による実装

### Write Model（Command Service）

Event Store にイベントとして保存される：

- **VocabularyEntryCreated**: 新しい見出し語の作成
- **VocabularyItemAdded**: 見出し語に新しい項目（意味）を追加
- **DefinitionAdded**: 項目に新しい定義を追加
- **ExampleAdded**: 定義に新しい例文を追加
- **ItemPublished**: 項目を公開状態に変更

各イベントは集約の状態変更を表し、Event Store に追記される。

### Read Model（Query Service）

非正規化された投影として保存される：

```json
{
  "item_id": "uuid",
  "spelling": "apple",
  "definitions": [
    {
      "definition_id": "uuid",
      "part_of_speech": "noun",
      "meaning": "A round fruit...",
      "domain": null,
      "examples": [
        {
          "example_text": "I ate an apple for lunch.",
          "example_translation": "昼食にりんごを食べました。"
        }
      ]
    }
  ],
  "synonyms": {...},
  "antonyms": {...}
}
```

この構造により、1回のクエリで画面表示に必要なすべての情報を取得できる。
