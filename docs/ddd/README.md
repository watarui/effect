# Domain-Driven Design (DDD) ドキュメント

## 概要

このディレクトリには、Effect プロジェクトの Domain-Driven Design プロセスに関するドキュメントが含まれています。
[DDD-Crew の DDD Starter Modelling Process](https://ddd-crew.github.io/ddd-starter-modelling-process/) に基づいて進めています。

## プロジェクトの背景

Effect は、英語語彙学習プラットフォームです（IELTS 対策）。主な目的：

- **真の目的**: アーキテクチャ学習（DDD、マイクロサービス、イベントソーシング、CQRS、ヘキサゴナルアーキテクチャ）
- **ユーザー**: 開発者本人と家族数名（学習デモアプリ）
- **方針**: 対話を通じて少しずつ確実に進める

## 現在のドキュメント構造

### [進捗サマリー](./progress-summary.md) ⭐

現在の進捗状況と再開時のガイド（最初に読むべきドキュメント）

### 戦略的設計 (Strategic Design)

- [ドメインビジョン](./strategic/domain-vision.md) - ビジネス目標と成功の定義
- [境界づけられたコンテキスト](./strategic/bounded-contexts.md) - 6 つのコンテキストの定義
- [コンテキストマップ](./strategic/context-map.md) - コンテキスト間の関係
- [統合パターン](./strategic/integration-patterns.md) - コンテキスト間の統合方法

### 発見フェーズ (Discovery)

- [イベントストーミング - ビッグピクチャー](./discovery/event-storming/big-picture.md) - 全体像の把握
- [ユビキタス言語](./discovery/ubiquitous-language.md) - ドメイン用語辞書

### 詳細設計 (Design)

#### EventStorming Design Level（各コンテキスト）

- [Learning Context](./design/learning-context/event-storming-design-level.md)
- [Vocabulary Context](./design/vocabulary-context/event-storming-design-level.md)
- [Learning Algorithm Context](./design/learning-algorithm-context/event-storming-design-level.md)
- [Progress Context](./design/progress-context/event-storming-design-level.md)
- [AI Integration Context](./design/ai-integration-context/event-storming-design-level.md)
- [User Context](./design/user-context/event-storming-design-level.md)

#### 集約設計

- [集約の特定と境界](./design/aggregate-identification.md) - 全集約の整理と分析

## 確定した 6 つのコンテキスト

1. **Learning Context（学習）** - 学習セッション管理
2. **Vocabulary Context（語彙管理）** - 語彙エントリの管理
3. **Learning Algorithm Context（学習アルゴリズム）** - SM-2 アルゴリズム実装
4. **Progress Context（進捗）** - CQRS/イベントソーシング
5. **AI Integration Context（AI 統合）** - AI 生成と深掘り
6. **User Context（ユーザー）** - 認証と権限管理

## DDD プロセスの進捗

```mermaid
graph TD
    A[Align - 調整 ✅] --> B[Discover - 発見 ✅]
    B --> C[Decompose - 分解 ✅]
    C --> D[Design - 設計 ⚡進行中]
    D --> E[Implement - 実装]
    E --> F[Evolve - 進化]
    F --> B
```

### 完了フェーズ

- ✅ Phase 1: Align - ビジネス目標の明確化
- ✅ Phase 2: Discover - EventStorming Big Picture、ユビキタス言語
- ✅ Phase 3: Decompose - 境界づけられたコンテキストの特定
- ✅ Phase 4: Design - EventStorming Design Level（全コンテキスト）
- ✅ Phase 4: Design - 集約の特定（テキスト整理）

### 現在の作業

- ⚡ Phase 4: Design - 集約の PlantUML 図作成
- 📋 Phase 4: Design - リポジトリとサービスの設計

## 重要な決定事項

- **用語統一**: 「語句」→「項目（Item）」
- **認証**: Firebase Auth + Google OAuth のみ
- **通知機能**: 実装しない
- **UI**: 日本語固定、タイムゾーン JST 固定
- **テスト**: 1 セッション最大 100 問（設定可能）、約 25 分
- **マイクロサービス**: 最初から分離（cargo workspace monorepo）

## アーカイブ

古いドキュメントは [`docs/archive/ddd/`](./../archive/ddd/) フォルダに移動しました。

## 参考資料

- [DDD-Crew: DDD Starter Modelling Process](https://ddd-crew.github.io/ddd-starter-modelling-process/)
- [Domain-Driven Design (Eric Evans)](https://www.dddcommunity.org/book/evans_2003/)
- [Implementing Domain-Driven Design (Vaughn Vernon)](https://www.amazon.com/dp/0321834577)
- [Event Storming (Alberto Brandolini)](https://www.eventstorming.com/)
