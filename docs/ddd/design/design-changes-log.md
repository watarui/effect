# DDD 設計変更記録

## 概要

このドキュメントは、Effect プロジェクトの DDD 設計プロセスにおける変更内容と、各設計文書間の不整合を管理するための記録です。
Canvas 作成など新しい設計作業を進めながら、過去の成果物との整合性を保つために使用します。

作成日: 2025-07-29

## 変更履歴

### 2025-07-29: ItemsSelected の同期化

- **変更内容**: ItemsSelected を非同期から同期に変更
- **理由**: UX の一貫性を優先（学習開始時に即座に項目リストが必要）
- **決定場所**: Learning Context Bounded Context Canvas
- **影響範囲**:
  - Learning Context と Learning Algorithm Context の統合パターン
  - イベントフローの設計
  - 実装時の通信方式

### 2025-07-27: Learning Algorithm Context の追加

- **変更内容**: 6 つ目のコンテキストとして Learning Algorithm Context を追加
- **理由**: 学習アルゴリズムの責務を明確に分離
- **影響範囲**:
  - 全体のコンテキストマップ
  - Learning Context の責務範囲

### 2025-07-xx: Progress Context の責務簡素化

- **変更内容**: Progress Context を純粋なイベントソーシングに特化
- **理由**: 統計計算は各コンテキストに委譲し、責務を明確化
- **影響範囲**:
  - Progress Context の設計
  - 統計情報の算出責任の移譲

## 未反映の変更リスト

### 高優先度

1. **ItemsSelected の同期化**
   - 対象: `/docs/ddd/design/event-storming-design-level/learning-context.md`
   - 対象: `/docs/ddd/design/event-storming-design-level/learning-algorithm-context.md`
   - 内容: 非同期イベントから同期 API 呼び出しに変更

2. **コンテキスト間の関係パターン**
   - 対象: `/docs/ddd/strategic/context-map.md`
   - 内容: Learning Context と Learning Algorithm Context の関係を Partnership に更新

3. **イベント名の統一**
   - 対象: 全 event-storming-design-level ドキュメント
   - 内容: 命名規則の統一（例: SessionStarted → LearningSessionStarted）

### 中優先度

4. **集約の責務説明の明確化**
   - 対象: `/docs/ddd/design/aggregate-identification.md`
   - 内容: UserItemRecord と ItemLearningRecord の責務の違いを明確に説明

5. **Progress Context の責務範囲**
   - 対象: `/docs/ddd/strategic/bounded-contexts.md`
   - 内容: 最新の設計（純粋なイベントソーシング）に合わせて更新

6. **AI Integration Context の戦略的分類**
   - 対象: `/docs/ddd/strategic/context-map.md`
   - 内容: Generic Subdomain として統一

### 低優先度

7. **ドメイン用語の統一**
   - 対象: 全ドキュメント
   - 内容:
     - 「項目」→「Item」に統一
     - 「覚えた感」→「Sense of Mastery」に統一
     - MasteryStatus の値を統一

8. **ユビキタス言語の更新**
   - 対象: `/docs/ddd/discovery/ubiquitous-language.md`
   - 内容: 最新の用語定義に更新

## 追加で発見された不整合（2025-07-29）

### 高優先度

9. **Progress Context の設計不整合**
   - bounded-contexts.md：統計計算の責務あり（line 101-104）
   - progress-context.md：純粋なイベントソーシング、集約なし（line 26-28）
   - 内容：Progress Context は集約を持たない純粋な Read Model として統一すべき

10. **Vocabulary Context の設計アプローチ不整合**

- bounded-contexts.md：単純な CRUD 操作（line 36）
- vocabulary-context.md：Wikipedia 方式、楽観的ロック、イベントソーシング（line 9-12）
- 内容：最新の Wikipedia 方式の設計に統一

11. **domain-types.md の存在**

- 対象：`/docs/ddd/strategic/domain-types.md`
- 内容：このファイルは使用されていない（空またはテンプレート）
- アクション：削除または内容を追加

### 中優先度

12. **イベント名の不統一（詳細）**

- ubiquitous-language.md：`VocabularyItemRegistered`（line 238）
- context-map.md：`ItemRegistered`（line 257）
- vocabulary-context.md：異なるイベント体系
- 内容：プレフィックスルールを決定（例：`{Context}_{Action}`）

13. **セッション時間の不整合**

- ubiquitous-language.md：25分のポモドーロ単位（line 101）
- learning-context.md：最大100問（設定可能）、約25分（line 18）
- 内容：時間ベースか問題数ベースかを明確化

14. **共有カーネルの定義場所**

- context-map.md：Shared Kernel セクションあり（line 210-221）
- 他のドキュメント：参照なし
- 内容：共有型の定義場所を統一、各コンテキストから参照

15. **統合パターンの表記不統一**

- context-map.md：Customer-Supplier、Publisher-Subscriber など
- Canvas：同期/非同期の観点も含む
- 内容：統合パターンの表記方法を統一

### 低優先度

16. **CreatedBy の型定義不整合**

- ubiquitous-language.md：概念のみ（line 44）
- vocabulary-context.md：詳細な enum 定義（line 77-81）
- 内容：実装詳細をどこまで設計文書に含めるか統一

17. **認証方式の表記**

- User Context：Firebase/Google OAuth（複数箇所）
- 一部：Firebase Auth + Google OAuth
- 内容：表記を統一

18. **ドメインイベントのグルーピング**

- ubiquitous-language.md：ドメインごとにグループ化（line 231-256）
- context-map.md：コンテキストごとにグループ化（line 225-267）
- 内容：イベントの整理方法を統一

19. **更新履歴の記載方法**

- 一部：詳細な更新内容
- 一部：日付のみ
- 内容：更新履歴の記載レベルを統一

20. **マークダウンのコードブロック言語指定**

- 一部：`rust`
- 一部：言語指定なし
- 内容：コードブロックの言語指定を統一

## 更新対象ドキュメント一覧

| ドキュメント | 更新内容 | 優先度 | 備考 |
|------------|---------|--------|------|
| `/docs/ddd/strategic/context-map.md` | コンテキスト関係、AI の分類 | 高 | 全体像に影響 |
| `/docs/ddd/design/event-storming-design-level/learning-context.md` | ItemsSelected の同期化 | 高 | 実装に直接影響 |
| `/docs/ddd/design/event-storming-design-level/learning-algorithm-context.md` | ItemsSelected の同期化 | 高 | 実装に直接影響 |
| `/docs/ddd/strategic/bounded-contexts.md` | Progress Context の責務 | 中 | 概念理解に影響 |
| `/docs/ddd/design/aggregate-identification.md` | 集約の責務説明 | 中 | 設計理解に影響 |
| `/docs/ddd/discovery/ubiquitous-language.md` | 用語の統一 | 低 | 可読性向上 |
| 各 event-storming ドキュメント | イベント名の統一 | 低 | 一貫性向上 |

## Canvas 完成後の更新計画

### Phase 1: 重要な概念の更新（1-2時間）

1. context-map.md の更新
   - コンテキスト間の関係を最新化
   - 戦略的分類を統一

2. ItemsSelected の同期化
   - 関連する event-storming ドキュメントを更新
   - 統合パターンの説明を修正

### Phase 2: 責務と境界の明確化（1-2時間）

3. bounded-contexts.md の更新
   - 各コンテキストの責務を最新化
   - 特に Progress Context の簡素化を反映

4. aggregate-identification.md の更新
   - 集約間の責務の違いを明確に説明

### Phase 3: 詳細の統一（1時間）

5. イベント名の統一
   - 命名規則を決定
   - 全ドキュメントで統一

6. ドメイン用語の統一
   - ユビキタス言語の更新
   - 全ドキュメントで表記統一

## 設計原則の確認

この更新作業を通じて、以下の DDD 原則を維持します：

- **ユビキタス言語の一貫性**: チーム全体で同じ用語を使用
- **境界づけられたコンテキストの独立性**: 各コンテキストの自律性を保持
- **継続的な改善**: 設計は進化するものとして受け入れる
- **ドキュメントは生きている**: 実装と設計の乖離を防ぐ

## メモ

- Canvas 作成中に発見した変更は、このドキュメントに随時追記する
- 大きな設計変更が発生した場合は、即座に影響範囲を評価する
- 実装フェーズに入る前に、必ず全ての高優先度項目を更新する
