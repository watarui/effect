# Vocabulary Context - API スキーマ

## 概要

Vocabulary Context の GraphQL API スキーマ定義です。API Gateway を通じて提供されます。

## GraphQL スキーマ

※ 実装では ID 型は値オブジェクト（UserId, ItemId, EntryId など）を使用しますが、ドキュメントでは理解しやすさのため UUID と表記しています。

### 基本型定義

```graphql
scalar DateTime
scalar UUID

enum VocabularyStatus {
  DRAFT
  PENDING_AI
  PUBLISHED
}

enum PartOfSpeech {
  NOUN
  VERB
  ADJECTIVE
  ADVERB
  PREPOSITION
  CONJUNCTION
  INTERJECTION
}

enum Register {
  FORMAL
  INFORMAL
  SLANG
  TECHNICAL
  LITERARY
}

enum CEFRLevel {
  A1
  A2
  B1
  B2
  C1
  C2
}
```

### オブジェクト型

```graphql
type Definition {
  id: UUID!
  partOfSpeech: PartOfSpeech!
  meaning: String!
  meaningTranslation: String
  domain: String
  register: Register
  examples: [Example!]!
}

type Example {
  text: String!
  translation: String
  source: String
}

type VocabularyItem {
  id: UUID!
  spelling: String!
  pronunciation: String
  phoneticRespelling: String
  definitions: [Definition!]!
  synonyms: [String!]!
  antonyms: [String!]!
  collocations: [Collocation!]!
  register: Register
  cefrLevel: CEFRLevel
  status: VocabularyStatus!
  createdAt: DateTime!
  updatedAt: DateTime!
  version: Int!
}

type Collocation {
  type: String!
  pattern: String!
  example: String
}

type VocabularyEntry {
  id: UUID!
  spelling: String!
  items: [VocabularyItem!]!
  createdAt: DateTime!
}

type VocabularyItemEdge {
  node: VocabularyItem!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

type VocabularyItemConnection {
  edges: [VocabularyItemEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type VocabularyStats {
  totalEntries: Int!
  totalItems: Int!
  totalDefinitions: Int!
  itemsByStatus: StatusCounts!
  itemsByCEFR: CEFRCounts!
}

type StatusCounts {
  draft: Int!
  pendingAI: Int!
  published: Int!
}

type CEFRCounts {
  A1: Int!
  A2: Int!
  B1: Int!
  B2: Int!
  C1: Int!
  C2: Int!
}
```

### クエリ

```graphql
type Query {
  # 単一項目の取得
  vocabularyItem(id: UUID!): VocabularyItem
  
  # エントリの取得
  vocabularyEntry(id: UUID!): VocabularyEntry
  vocabularyEntryBySpelling(spelling: String!): VocabularyEntry
  
  # 検索・一覧
  searchVocabularyItems(
    query: String
    status: VocabularyStatus
    cefrLevel: CEFRLevel
    partOfSpeech: PartOfSpeech
    first: Int = 20
    after: String
  ): VocabularyItemConnection!
  
  # 全文検索（Meilisearch）
  searchFullText(
    query: String!
    limit: Int = 20
  ): [VocabularyItem!]!
  
  # 統計情報
  vocabularyStats: VocabularyStats!
}
```

### ミューテーション

```graphql
input CreateVocabularyItemInput {
  spelling: String!
  definitions: [DefinitionInput!]!
  partOfSpeech: PartOfSpeech!
  pronunciation: String
  register: Register
  domain: String
}

input DefinitionInput {
  meaning: String!
  meaningTranslation: String
  domain: String
  register: Register
}

input UpdateVocabularyItemInput {
  definitions: [DefinitionInput!]
  pronunciation: String
  phoneticRespelling: String
  register: Register
  cefrLevel: CEFRLevel
}

input AddExampleInput {
  definitionId: UUID!
  text: String!
  translation: String
}

type Mutation {
  # 項目の作成
  createVocabularyItem(
    input: CreateVocabularyItemInput!
  ): UUID! @auth(requires: USER)
  
  # 項目の更新
  updateVocabularyItem(
    id: UUID!
    input: UpdateVocabularyItemInput!
    version: Int!
  ): Boolean! @auth(requires: USER)
  
  # 例文の追加
  addExample(
    itemId: UUID!
    input: AddExampleInput!
  ): Boolean! @auth(requires: USER)
  
  # 項目の公開
  publishVocabularyItem(
    id: UUID!
  ): Boolean! @auth(requires: ADMIN)
  
  # AI エンリッチメントの要求（全情報を一括生成）
  requestAIEnrichment(
    itemId: UUID!
  ): UUID! @auth(requires: USER)
}
```

## 認証・認可

### ディレクティブ

```graphql
directive @auth(requires: Role = USER) on FIELD_DEFINITION

enum Role {
  USER
  ADMIN
}
```

### 認証フロー

1. クライアントは Firebase Auth でトークンを取得
2. GraphQL リクエストの Authorization ヘッダーにトークンを含める
3. API Gateway でトークンを検証
4. ユーザー情報をコンテキストに追加

### 認可ルール

- **読み取り**: 認証不要（公開情報）
- **作成・更新**: 認証必須（USER ロール）
- **公開・削除**: 管理者のみ（ADMIN ロール）

## エラーハンドリング

### エラー型

```graphql
interface Error {
  message: String!
  code: String!
}

type ValidationError implements Error {
  message: String!
  code: String!
  field: String!
}

type ConflictError implements Error {
  message: String!
  code: String!
  currentVersion: Int!
}

type NotFoundError implements Error {
  message: String!
  code: String!
  resourceType: String!
  resourceId: String!
}
```

### エラーレスポンス例

```json
{
  "errors": [
    {
      "message": "Version conflict",
      "extensions": {
        "code": "CONFLICT",
        "currentVersion": 5
      }
    }
  ]
}
```

## サブスクリプション（将来実装・優先度低）

```graphql
type Subscription {
  # 項目の更新を購読
  vocabularyItemUpdated(itemId: UUID!): VocabularyItem!
  
  # 新規項目の作成を購読
  vocabularyItemCreated(spelling: String!): VocabularyItem!
}
```

※ リアルタイム更新は学習アプリとして優先度が低いため、当面は実装しない
