# Shared Kernel（共有カーネル）

## 概要

Shared Kernel は、複数の境界づけられたコンテキスト間で共有される、小さく安定したモデルの集合です。これらは慎重に管理され、変更は全ての関係者の合意のもとで行われます。

## 共有される識別子

すべてのコンテキストで同じ意味と形式を持つ識別子です。

```rust
// ユーザーを一意に識別
pub struct UserId(Uuid);

// 学習項目を一意に識別
pub struct ItemId(Uuid);

// 学習セッションを一意に識別
pub struct SessionId(Uuid);

// AIタスクを一意に識別
pub struct TaskId(Uuid);
```

### 識別子の実装ガイドライン

1. **不変性**: 一度生成された識別子は変更不可
2. **一意性**: UUID v4 を使用してグローバルな一意性を保証
3. **型安全性**: プリミティブ型ではなく値オブジェクトとして実装
4. **シリアライズ**: JSON、文字列形式での相互変換をサポート

## 共有される基本的な値オブジェクト

### コースタイプ

```rust
pub enum CourseType {
    Ielts,
    Toefl,
    Toeic,
    Eiken,
    GeneralEnglish,
}
```

### CEFR レベル

```rust
pub enum CefrLevel {
    A1,  // Beginner
    A2,  // Elementary
    B1,  // Intermediate
    B2,  // Upper Intermediate
    C1,  // Advanced
    C2,  // Proficient
}
```

### 言語コード

```rust
pub struct LanguageCode(String); // ISO 639-1 形式（例: "en", "ja"）
```

### タイムスタンプ

```rust
use chrono::{DateTime, Utc};

pub type Timestamp = DateTime<Utc>;
```

## 共有される列挙型

### 反応タイプ

```rust
pub enum ResponseType {
    Correct,
    Incorrect,
    Skipped,
}
```

### マスタリーステータス

```rust
pub enum MasteryStatus {
    Unknown,      // 未学習
    Tested,       // テスト済み（1回以上正解）
    ShortTerm,    // 短期記憶に定着
    LongTerm,     // 長期記憶に定着
}
```

## 使用上の注意事項

### 1. 変更管理

- Shared Kernel の変更は、影響を受けるすべてのコンテキストの担当者による合意が必要
- 後方互換性を維持し、破壊的変更は避ける
- 変更履歴を明確に記録する

### 2. 最小限の共有

- 本当に共有が必要なものだけを含める
- コンテキスト固有のロジックは含めない
- ビジネスルールではなく、データ構造のみを共有

### 3. 実装の独立性

- 各コンテキストは Shared Kernel を参照するが、依存を最小限に保つ
- 可能な限り、コンテキスト内部での変換を通じて独立性を維持

## バージョニング戦略

```rust
// バージョン情報の埋め込み
pub const SHARED_KERNEL_VERSION: &str = "1.0.0";

// 互換性チェック
pub fn is_compatible(required_version: &str) -> bool {
    // セマンティックバージョニングに基づく互換性チェック
}
```

## 例: コンテキスト間でのデータ交換

```rust
// Learning Context から Progress Context へのイベント
pub struct LearningSessionCompleted {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub course_type: CourseType,
    pub completed_at: Timestamp,
    pub items_studied: Vec<ItemId>,
    pub correct_count: u32,
    pub total_count: u32,
}
```

## アンチパターン

### ❌ 避けるべきこと

1. **ビジネスロジックの共有**

   ```rust
   // Bad: ビジネスロジックは各コンテキストで実装すべき
   pub fn calculate_next_review_date(item: &Item) -> DateTime<Utc> {
       // このような計算ロジックは共有しない
   }
   ```

2. **頻繁な変更**

   - 実験的な機能や頻繁に変更される要素は含めない

3. **大きすぎる共有**
   - エンティティ全体やアグリゲートは共有しない

### ✅ 推奨されること

1. **シンプルなデータ構造**

   - 識別子、列挙型、基本的な値オブジェクトのみ

2. **明確な命名**

   - すべてのコンテキストで同じ意味を持つ名前を使用

3. **ドキュメント化**
   - 各要素の意味と使用方法を明確に文書化

## 関連ドキュメント

- ユビキタス言語: `/docs/ddd/discovery/ubiquitous-language.md`
- コンテキストマップ: `/docs/ddd/strategic/context-map.md`
- 各コンテキストの詳細: `/docs/ddd/design/bounded-context-canvas/*.md`

## 更新履歴

- 2025-07-30: 初版作成
