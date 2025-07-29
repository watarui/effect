# Effect プロジェクト - DDD 設計進捗サマリー

## 概要

このドキュメントは、Effect プロジェクトの戦略的 DDD 設計の現在の進捗状況と、今後の作業再開のためのガイドです。

作成日: 2025-07-27  
最終更新: 2025-07-29

## プロジェクト背景

- **目的**: 英語語彙学習プラットフォーム（IELTS 対策）
- **真の目的**: アーキテクチャ学習（DDD、マイクロサービス、イベントソーシング、CQRS、ヘキサゴナルアーキテクチャ）
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

### 直近の作業内容（2025-07-27 〜 2025-07-29）

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

## 今後の作業

### 直近のタスク

1. **Phase 4: Design - ドメインサービスの設計**（必要に応じて）
   - 各コンテキストのドメインサービス特定
   - サービスインターフェースの定義

2. **Phase 5: Implementation - 技術選定と設計**
   - 技術スタックの選定（Rust、データベース、メッセージブローカー等）
   - プロジェクト構造の設計（cargo workspace 構成）
   - インフラストラクチャ層の設計
   - API 設計（GraphQL/REST）

### 実装フェーズの検討事項

1. **マイクロサービス構成**
   - cargo workspace を使用したモノレポ構成
   - 各コンテキストを独立したクレートとして実装
   - 共通ライブラリの設計

2. **イベント駆動アーキテクチャ**
   - イベントバスの実装方針
   - イベントストアの選定（Progress Context 用）
   - 集約間通信の実装

3. **永続化戦略**
   - 各コンテキストのデータベース選定
   - リポジトリパターンの実装
   - トランザクション管理

## 会話・設計方針

### 基本方針

- **対話重視**: 勝手に進めず、必ず確認を取る
- **段階的進行**: 少しずつ確実に進める
- **学習目的優先**: 実用性よりアーキテクチャ学習を重視

### 技術的方針

- **マイクロサービス**: 最初から分離（cargo workspace monorepo）
- **イベント駆動**: 集約間は疎結合
- **CQRS/ES**: 適切な箇所で採用（Progress Context など）
- **シンプル優先**: 過度な抽象化を避ける

### 決定事項

- 認証: Firebase Auth + Google OAuth のみ
- 通知機能: 実装しない
- UI: 日本語固定、タイムゾーン JST 固定
- テスト: 1 セッション最大 100 問（設定可能）、約 25 分

## 再開時のチェックリスト

1. このドキュメントを読んで全体像を把握
2. `/docs/ddd/design/aggregate-identification.md` で集約設計を確認
3. `/docs/ddd/design/repositories/` でリポジトリ設計を確認
4. 次のステップ（ドメインサービス設計 or 実装フェーズ）を決定

## 関連ドキュメント

### 戦略的 DDD

- `/docs/ddd/strategic/domain-vision.md`
- `/docs/ddd/strategic/bounded-contexts.md`
- `/docs/ddd/strategic/context-map.md`

### EventStorming

- `/docs/ddd/discovery/event-storming/big-picture.md`
- `/docs/ddd/discovery/ubiquitous-language.md`

### 詳細設計

- `/docs/ddd/design/*/event-storming-design-level.md`（各コンテキスト）
- `/docs/ddd/design/aggregate-identification.md`

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
