# Vocabulary Context - 集約設計

## 概要

Vocabulary Context は、Effect プロジェクトにおける語彙コンテンツ管理の中核です。全ユーザーが共有するグローバル辞書を管理し、AI と連携して豊富な語彙情報を提供します。

### 主要な設計方針

1. **Wikipedia 方式**: 1つの綴り（spelling）に対して複数の意味（disambiguation）を持つ項目を管理
2. **楽観的ロック + 自動マージ**: 並行編集に対して可能な限り自動マージを試みる
3. **イベントソーシング**: すべての変更を記録し、完全な履歴とバージョン管理を実現
4. **AI との非同期連携**: 項目情報の生成を AI に委譲し、非同期で処理

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
- definitions: 定義のリスト
- parts_of_speech: 品詞のリスト
- examples: 例文のリスト
- synonyms: 類義語のリスト
- antonyms: 対義語のリスト
- collocations: コロケーション
- register: レジスター（formal, informal など）
- cefr_level: CEFR レベル
- tags: タグのリスト

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
