# Vocabulary Context - クエリ定義

## 概要

Vocabulary Context の Query Service が提供するクエリパターンの定義です。すべてのクエリは Read Model から高速に取得されます。

## 基本的なクエリ

### GetVocabularyItem

単一の語彙項目を取得。

**パラメータ**:

- item_id: 項目ID

**レスポンス**:

```typescript
{
  item_id: string
  spelling: string
  definitions: Definition[]
  synonyms: string[]
  antonyms: string[]
  status: string
  created_at: string
  updated_at: string
}
```

### GetVocabularyEntry

見出し語とそれに紐づく全項目を取得。

**パラメータ**:

- entry_id: エントリID

**レスポンス**:

```typescript
{
  entry_id: string
  spelling: string
  items: VocabularyItemSummary[]
}
```

## 検索クエリ

### SearchVocabularyItems

語彙項目を検索（Query Service 経由）。

**パラメータ**:

- query: 検索文字列
- filters:
  - status: ステータスでフィルタ
  - cefr_level: CEFR レベルでフィルタ
  - part_of_speech: 品詞でフィルタ
- first: 取得件数（最大100）
- after: カーソル（ページネーション用）

**レスポンス**:

```typescript
{
  edges: {
    node: VocabularyItem
    cursor: string
  }[]
  pageInfo: {
    hasNextPage: boolean
    endCursor: string
  }
  totalCount: number
}
```

### SearchWithMeilisearch

全文検索（Search Service 経由）。

**パラメータ**:

- query: 検索文字列
- limit: 取得件数（デフォルト20）

**特徴**:

- Typo 許容
- 部分一致
- 関連度順でソート

## 統計クエリ

### GetVocabularyStats

語彙の統計情報を取得。

**レスポンス**:

```typescript
{
  total_entries: number
  total_items: number
  total_definitions: number
  items_by_status: {
    draft: number
    published: number
  }
  items_by_cefr: {
    A1: number
    A2: number
    // ...
  }
}
```

## ページネーション

### Cursor ベースページネーション

**利点**:

- データ追加時もページがずれない
- 大規模データセットで効率的

**使用例**:

```graphql
# 最初のページ
query {
  searchItems(query: "test", first: 20) {
    edges {
      node { id, spelling }
      cursor
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}

# 次のページ
query {
  searchItems(query: "test", first: 20, after: "cursor_xyz") {
    # ...
  }
}
```

## キャッシング戦略

### Redis キャッシュ

**キャッシュされるクエリ**:

- GetVocabularyItem: 5分間
- GetVocabularyEntry: 5分間
- GetVocabularyStats: 1分間

**キャッシュキー**:

```
vocabulary:item:{item_id}
vocabulary:entry:{entry_id}
vocabulary:stats
```

**キャッシュ無効化**:

- 項目更新時に関連キャッシュを削除
- TTL による自動期限切れ

## パフォーマンス考慮事項

### N+1 問題の回避

- JSONB による非正規化でJOIN不要
- 一度のクエリで必要な情報をすべて取得

### インデックス

以下のフィールドにインデックス:

- spelling
- status
- cefr_level
- created_at（ソート用）

### レート制限

- 認証なし: 60 req/min
- 認証あり: 600 req/min
- Search Service: 100 req/min
