# Vocabulary Context - コマンド定義

## 概要

Vocabulary Context で処理されるコマンドの定義です。これらのコマンドは Command Service で受け付けられ、ドメインロジックを通じてイベントに変換されます。

## コマンド一覧

### CreateVocabularyItem

新しい語彙項目を作成するコマンド。

**必須フィールド**:

- spelling: 綴り
- definitions: 定義のリスト（最低1つ）
- part_of_speech: 品詞
- created_by: 作成者ID（認証から取得）

**オプションフィールド**:

- pronunciation: 発音記号
- register: レジスター（formal/informal など）
- domain: 分野（medical/computing など）

**バリデーション**:

- spelling は空文字不可
- definitions は最低1つ必要
- 同じ spelling + part_of_speech の組み合わせは重複チェック

### UpdateVocabularyItem

既存の語彙項目を更新するコマンド。

**必須フィールド**:

- item_id: 更新対象の項目ID
- updated_by: 更新者ID（認証から取得）
- version: 楽観的ロック用のバージョン

**更新可能フィールド**:

- definitions: 定義の追加・修正・削除
- pronunciation: 発音記号
- examples: 例文の追加・修正・削除
- synonyms: 類義語
- antonyms: 対義語
- register: レジスター
- domain: 分野

**バリデーション**:

- version が現在のバージョンと一致すること
- definitions を空にすることは不可
- 更新者は認証済みユーザーであること

### PublishVocabularyItem

語彙項目を公開状態にするコマンド。

**必須フィールド**:

- item_id: 公開する項目ID
- published_by: 公開者ID（認証から取得）

**バリデーション**:

- ステータスが Draft であること
- 必要な情報が揃っていること（定義、品詞など）

### RequestAIEnrichment

AI による項目情報の生成を要求するコマンド。

**必須フィールド**:

- item_id: 対象項目ID
- requested_by: 要求者ID

**生成内容**:

- 定義の補強
- 例文の生成
- 発音情報
- 類義語・対義語
- コロケーション

※ すべての情報を一括で生成（個別選択は不可）

**バリデーション**:

- 項目が存在すること
- 同じタイプの生成が進行中でないこと

## コマンド処理の流れ

```
1. API Gateway でコマンドを受信
2. 認証・認可のチェック
3. Command Service でバリデーション
4. ドメインモデルにコマンドを適用
5. イベントを生成
6. Event Store に保存
7. Event Bus に発行
```

## エラーハンドリング

**バリデーションエラー**:

- 400 Bad Request として返却
- エラーメッセージは具体的に

**競合エラー**:

- 409 Conflict として返却
- 最新バージョンを含めて返却

**権限エラー**:

- 401 Unauthorized（未認証）
- 403 Forbidden（権限不足）

## 冪等性

- CreateVocabularyItem: spelling + part_of_speech で重複チェック
- UpdateVocabularyItem: version によるチェック
- PublishVocabularyItem: 既に Published なら成功として扱う
