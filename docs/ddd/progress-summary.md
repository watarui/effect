# Effect プロジェクト - DDD 設計進捗サマリー

## 概要

このドキュメントは、Effect プロジェクトの戦略的 DDD 設計の現在の進捗状況と、今後の作業再開のためのガイドです。

作成日: 2025-07-27  
最終更新: 2025-07-31

## プロジェクト背景

- **目的**: 英語語彙学習プラットフォーム（試験対策：IELTS、TOEFL 等）
- **真の目的**: アーキテクチャ学習（DDD、マイクロサービス、イベントソーシング、CQRS、ヘキサゴナルアーキテクチャ、Saga パターン）
- **ユーザー**: 開発者本人と家族数名（学習デモアプリ）
- **方針**: 対話を通じて少しずつ確実に進める

## 完了した作業

### Phase 1: Align ✅ 完了

- ビジネス目標の明確化
- ステークホルダーの特定
- 成功の定義

**主な成果**:

- アーキテクチャ学習が主目的であることを明確化
- 「覚えた感」を重視する学習体験の定義

### Phase 2: Discover ✅ 完了

- EventStorming Big Picture
- ユビキタス言語の構築

**主な成果**:

- 「語句」→「項目（Item）」への用語統一
- 6 つのコンテキストの発見（Learning Algorithm Context を追加）

### Phase 3: Decompose ✅ 完了

- 境界づけられたコンテキストの特定
- コンテキストマップの作成
- 統合パターンの選択

**確定した 6 つのコンテキスト**:

1. Learning Context（学習）
2. Vocabulary Context（語彙管理）
3. Learning Algorithm Context（学習アルゴリズム）
4. Progress Context（進捗）
5. AI Integration Context（AI 統合）
6. User Context（ユーザー）

### Phase 4: Design - EventStorming Design Level ✅ 完了

各コンテキストの詳細設計を完了：

1. **Learning Context**

   - ハイブリッド UI フロー（解答表示 → 3 秒自動確認）
   - 項目選定戦略の統合

2. **Vocabulary Context**

   - Wikipedia スタイル（同一スペリング複数意味）
   - オプティミスティックロック実装

3. **Learning Algorithm Context**

   - SM-2 アルゴリズム実装
   - 85% ルールに基づく動的調整

4. **Progress Context**

   - 純粋な CQRS/イベントソーシング
   - GraphQL 最適化設計

5. **AI Integration Context**

   - 機能別インターフェース
   - Anti-Corruption Layer パターン

6. **User Context**
   - Firebase Auth + Google OAuth
   - シンプルな権限管理（Admin/User）

### Phase 4: Design - 集約の特定 ✅ 完了

`/docs/ddd/design/aggregate-identification.md` を作成し、全集約を整理完了。

**特定した集約**:

- LearningSession, UserItemRecord（Learning Context）
- VocabularyEntry（Vocabulary Context）
- ItemLearningRecord（Learning Algorithm Context）
- AIGenerationTask, ChatSession（AI Integration Context）
- UserProfile（User Context）
- Progress Context は集約なし（純粋なイベントソーシング）

### Phase 4: Design - 集約の可視化 ✅ 完了

PlantUML 図を作成（`/docs/ddd/design/aggregates/`）:

- `aggregate-overview.puml` - 全集約の俯瞰図
- `aggregate-relationships.puml` - 集約間の参照関係
- `event-flow.puml` - イベントフロー図

### Phase 4: Design - 集約設計の検証 ✅ 完了

`/docs/ddd/design/aggregate-identification.md` で以下を分析:

- UserItemRecord（UI 表示用）と ItemLearningRecord（アルゴリズム用）の責務を明確化
- **現状維持を推奨**：DDD の原則に従い、各コンテキストが独自のモデルを持つ

### Phase 4: Design - リポジトリ設計 ✅ 完了

リポジトリインターフェースの設計を完了（`/docs/ddd/design/repositories/`）:

- ✅ `repository-design-overview.md` - 設計原則と共通インターフェース
- ✅ `learning-context-repositories.md` - Learning Context のリポジトリ
- ✅ `vocabulary-context-repositories.md` - Vocabulary Context のリポジトリ
- ✅ `learning-algorithm-context-repositories.md` - Learning Algorithm Context のリポジトリ
- ✅ `progress-context-repositories.md` - Progress Context のイベントストア
- ✅ `ai-integration-context-repositories.md` - AI Integration Context のリポジトリ
- ✅ `user-context-repositories.md` - User Context のリポジトリ

## 現在の状態

### 直近の作業内容（2025-07-27 〜 2025-07-31）

1. **ドキュメント整理**

   - 古いファイルを `docs/archive/` フォルダに移動
   - README.md を更新して現在の構成を明確化

2. **集約の可視化**

   - 3 つの PlantUML 図を作成（overview、relationships、event-flow）
   - 集約間の関係性を明確化

3. **設計上の課題解決**

   - UserItemRecord と ItemLearningRecord の責務分析
   - 現状維持（責務分離）を推奨する結論

4. **リポジトリ設計完了**

   - 設計原則の文書化
   - 全コンテキストのリポジトリインターフェース定義
   - Progress Context のプロジェクション簡素化（ストリーク追跡の統合）

5. **既存成果の改善**

   - 戦略的分類の詳細化（各コンテキストの分類理由を明確化）
   - イベント整合性分析（命名規則の統一案を提示）
   - Progress Context プロジェクション設計の詳細化（GraphQL マッピング）

6. **Bounded Context Canvas 作成開始**（2025-07-29）

   - Canvas テンプレート作成（DDD-Crew ベース、日本語説明付き）
   - Learning Context Canvas 作成
   - ItemsSelected を非同期から同期に変更（UX 優先の設計判断）

7. **Saga パターン分析**

   - 使用機会の分析と文書化（`/docs/ddd/design/saga-pattern-opportunities.md`）
   - AI 生成タスクの補償処理を最優先実装候補として選定

8. **設計変更管理**（2025-07-31）

   - design-changes-log.md で設計の不整合を管理
   - 学習セッションを 1 問刻みの問題数ベースに統一
   - ポモドーロ方式への言及を削除

9. **Bounded Context Canvas 完成**（2025-07-30）
   - 全 6 コンテキストの Canvas 作成完了
   - 各コンテキストの責務と境界を明確化

### Phase 4: Design - 既存成果の改善 ✅ 完了

1. **戦略的分類の詳細化**

   - `/docs/ddd/strategic/context-map.md` に各コンテキストの分類理由を追記
   - Core Domain の判定基準と差別化要因を明確化

2. **イベント整合性の確認**

   - `/docs/ddd/design/event-consistency-analysis.md` でイベント一覧と命名規則を分析
   - 命名規則の統一案を提示

3. **Progress Context プロジェクション設計の詳細化**
   - `/docs/ddd/design/progress-context-projection-mapping.md` で GraphQL との対応を文書化
   - 各プロジェクションの責務とクエリマッピングを明確化

### Phase 4: Design - Bounded Context Canvas ✅ 完了

1. **各コンテキストの責務と境界の明確化**

   - ✅ Canvas テンプレート作成完了
   - ✅ Learning Context Canvas 作成完了（2025-07-29）
   - ✅ Learning Algorithm Context Canvas 作成完了（2025-07-29）
   - ✅ Progress Context Canvas 作成完了（2025-07-30）
   - ✅ AI Integration Context Canvas 作成完了（2025-07-30）
   - ✅ Vocabulary Context Canvas 作成完了（2025-07-30）
   - ✅ User Context Canvas 作成完了（2025-07-30）

2. **Canvas 作成での決定事項**
   - ItemsSelected を非同期から同期に変更（Learning Context）
   - アーキテクチャの適材適所を重視
   - AI Integration Context の完全非同期化（イベント駆動）
   - IELTS スコア推定の除外（Progress Context）

### Phase 4: Design - ドメインサービス設計 ✅ 完了

1. **識別・設計済みのドメインサービス**

   - Learning Algorithm Context: `SM2Calculator`, `PerformanceAnalyzer`
   - AI Integration Context: `AIServiceAdapter`

2. **設計状況**
   - event-storming-design-level で必要なドメインサービスは定義済み
   - 追加の設計作業は不要

### Phase 4: Design - Aggregate Design Canvas ❌ 未実施

1. **重要な集約の詳細設計**
   - `LearningSession`（Learning Context）
   - `VocabularyEntry`（Vocabulary Context）
   - State Transitions、Invariants の文書化

**実施しない理由**:

- aggregate-identification.md で各集約の不変条件が明確に定義済み
- event-storming-design-level でコマンド/イベントを通じた状態変化が明確
- 集約がシンプルで複雑な状態遷移がない（YAGNI 原則）
- アーキテクチャ学習が主目的なので、過度な事前設計は避ける
- 実装時に必要になった場合に詳細化する方が実践的

## 今後の作業

### Phase 5: Implementation - 技術選定 ✅ 決定済み（2025-07-31）

1. **基本技術スタック**

   - 言語: Rust
   - Web フレームワーク: **Axum**（高性能、Tokio エコシステム）
   - GraphQL: async-graphql
   - 認証: Firebase Auth

2. **データ永続化**

   - RDB: PostgreSQL（Vocabulary, User Context）
   - イベントストア: **PostgreSQL + カスタム実装**（シンプル、学習価値）
   - キャッシュ: Redis

3. **メッセージング**
   - イベントバス: **Redis Streams**
     - 永続化により学習・デバッグが容易
     - 実践的なイベント駆動アーキテクチャ
     - シンプルな実装で本質を学習可能
   - 非同期処理: Tokio
   - Saga 実行: saga-executor サービス（Orchestration パターン）

### Phase 5: Implementation - プロジェクト構造設計 ✅ 決定済み（2025-07-31）

1. **cargo workspace 構成**

   ```
   effect/
   ├── Cargo.toml (workspace)
   ├── contexts/
   │   ├── learning/
   │   ├── learning-algorithm/
   │   ├── vocabulary/
   │   ├── user/
   │   ├── progress/
   │   └── ai-integration/
   ├── applications/
   │   ├── api-gateway/      # GraphQL API
   │   └── event-processor/  # イベント処理
   ├── services/
   │   └── saga-executor/    # Saga パターン実装
   └── shared/
       ├── common-types/     # UserId, ItemId 等の共通型
       ├── domain-events/    # DomainEvent と基本実装
       └── infrastructure/   # DB 接続、イベントバス実装等
   ```

2. **共通ライブラリ設計**
   - ドメインイベント定義
   - 共通型（UserId, ItemId 等）
   - インフラストラクチャ抽象化

### 実装フェーズの優先順位

1. **Phase 1: 基盤構築**

   - プロジェクト構造のセットアップ
   - 共通ライブラリの実装
   - CI/CD パイプライン

2. **Phase 2: Core Domain 実装**

   - Learning Context
   - Vocabulary Context
   - 基本的な GraphQL API

3. **Phase 3: アルゴリズムと分析**

   - Learning Algorithm Context
   - Progress Context（イベントソーシング）

4. **Phase 4: 統合と拡張**
   - AI Integration Context
   - Saga パターン実装（AI 生成タスクから開始）
   - 全体統合テスト

### Phase 5 開始前のチェックリスト

1. **技術選定の最終決定**

   - [x] Web フレームワーク: Axum（高性能、Tokio エコシステム）
   - [x] イベントストア: PostgreSQL + カスタム実装（シンプル、学習価値）
   - [x] イベントバス: Redis Streams（実践的で学習価値が高い）

2. **環境構築の準備**
   - [ ] Rust の最新版インストール
   - [ ] Docker と Docker Compose のセットアップ
   - [ ] PostgreSQL ローカル環境
   - [ ] Redis ローカル環境

### Phase 1: 基盤構築の詳細タスク

1. **cargo workspace の初期化**

   ```bash
   cargo new effect --name effect
   cd effect
   # Cargo.toml を workspace 設定に変更
   ```

2. **基本的なディレクトリ構造の作成**

   ```bash
   mkdir -p contexts/{learning,learning-algorithm,vocabulary,user,progress,ai-integration}
   mkdir -p shared/{domain-events,common-types,infrastructure}
   mkdir -p applications/{api-gateway,event-processor}
   mkdir -p services/saga-executor
   ```

3. **共通型の定義**

   - UserId, ItemId, SessionId などの強型付け
   - タイムスタンプ、UUID の標準化

4. **DomainEvent の基本実装**

   - Event trait の定義
   - EventStore trait の定義
   - EventBus trait の定義

5. **GitHub Actions CI 設定**
   - cargo test, cargo fmt, cargo clippy
   - 各コンテキストの独立ビルド確認

### 実装時の注意事項

- **最初からマイクロサービスアーキテクチャ**: cargo workspace monorepo で各コンテキストを独立サービスとして開発
- **テスト駆動開発（TDD）**: ドメインロジックからテストを書く
- **ドキュメント駆動開発**: 既存の設計を忠実に実装に反映
- **インクリメンタルな進行**: 小さな動くものから始める

## 会話・設計方針

### 基本方針

- **対話重視**: 勝手に進めず、必ず確認を取る
- **段階的進行**: 少しずつ確実に進める
- **学習目的優先**: 実用性よりアーキテクチャ学習を重視

### 技術的方針

- **マイクロサービス**: 最初から分離（cargo workspace monorepo）
- **イベント駆動**: 集約間は疎結合
- **CQRS/ES**: 適切な箇所で採用（Progress Context など）
- **Saga パターン**: 分散トランザクション管理（AI 生成タスクなど）
- **シンプル優先**: 過度な抽象化を避ける
- **適材適所**: 全てを非同期にせず、UX を考慮して同期/非同期を選択

### 決定事項

- 認証: Firebase Auth + Google OAuth のみ
- 通知機能: 実装しない
- UI: 日本語固定、タイムゾーン JST 固定
- 対応試験: IELTS、TOEFL 等（IELTS だけに特化しない）
- ItemsSelected: 同期通信（UX 優先の設計判断）
- AI Integration Context: 完全非同期化（タスクキュー方式）
- イベント名: DomainEvent wrapper パターンで統一
- IELTS スコア推定: 除外決定（CEFR レベルと進捗スコアで代替）
- 学習セッション: 1-100 問（1 問単位で設定可能）、デフォルト 50 問
- ポモドーロ方式: 廃止（問題数ベースに統一）

## 再開時のチェックリスト

1. このドキュメントを読んで全体像を把握
2. `/docs/ddd/design/aggregate-identification.md` で集約設計を確認
3. `/docs/ddd/design/repositories/` でリポジトリ設計を確認
4. 改善成果を確認:
   - `/docs/ddd/strategic/context-map.md` - 戦略的分類の詳細
   - `/docs/ddd/design/event-consistency-analysis.md` - イベント命名規則
   - `/docs/ddd/design/projections/progress-context-projection-mapping.md` - プロジェクション設計
5. Bounded Context Canvas を確認:
   - `/docs/ddd/design/bounded-context-canvas/template.md` - Canvas テンプレート
   - `/docs/ddd/design/bounded-context-canvas/learning-context.md` - 作成済み Canvas
6. Saga パターンの使用機会を確認:
   - `/docs/ddd/design/saga-pattern-opportunities.md` - 実装機会の分析
7. 設計変更記録を確認:
   - `/docs/ddd/design/design-changes-log.md` - 設計変更と不整合管理
8. 次のステップ: Phase 5: Implementation の「Phase 1: 基盤構築」から開始

## 関連ドキュメント

### 戦略的 DDD

- `/docs/ddd/strategic/domain-vision.md`
- `/docs/ddd/strategic/bounded-contexts.md`
- `/docs/ddd/strategic/context-map.md`

### EventStorming

- `/docs/ddd/discovery/event-storming/big-picture.md`
- `/docs/ddd/discovery/ubiquitous-language.md`

### 詳細設計

- `/docs/ddd/design/event-storming-design-level/`（各コンテキストの設計）
- `/docs/ddd/design/aggregate-identification.md`
- `/docs/ddd/design/repositories/`（リポジトリ設計）
- `/docs/ddd/design/projections/progress-context-projection-mapping.md`
- `/docs/ddd/design/bounded-context-canvas/`（Canvas 設計）
- `/docs/ddd/design/saga-pattern-opportunities.md`（Saga パターン分析）
- `/docs/ddd/design/design-changes-log.md`（設計変更記録）

## メモ

- ドメインエキスパートは開発者本人
- 「話を勝手に進めないでね」という要望あり
- 段階的アプローチは嫌い → フル機能を最初から設計
- 「覚えた感」が最重要価値

## ドキュメント整理の必要性

### 現状の問題

1. **重複したドキュメント**

   - 例：`bounded-context-contracts.md` と `strategic/bounded-contexts.md`
   - 同じ内容が複数箇所に存在し、どれが最新か不明

2. **古い設計と新しい設計の混在**

   - 最初の 5 コンテキスト設計 → 対話で 6 コンテキストに更新
   - 古いドキュメントが残っていて混乱の原因に

3. **フォルダ構造の不整合**
   - `tactical/` フォルダは現在使用していない
   - `architecture/` と `features/` は古い設計

### 整理案

- `docs/ddd/archive/` フォルダを作成し、古いドキュメントを移動
- 現在有効なドキュメントのみを残す
- README.md で現在の構成を明確化

### 対象ファイル

- アーカイブ対象：`architecture/`, `features/`, `tactical/`, 古い catalog 系ファイル
- 維持：今回の対話で作成/更新したドキュメント
