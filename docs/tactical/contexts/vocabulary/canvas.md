# Vocabulary Context Bounded Context Canvas

## 1. Name

Vocabulary Context

## 2. Purpose

Wikipedia スタイルの語彙データベースを管理し、同一スペリングで複数の意味を持つ項目を効率的に管理する。
AI と連携して豊富な語彙情報を自動生成し、全ユーザーが共有するグローバル辞書を提供する。
CQRS/Event Sourcing パターンを採用し、4 つのマイクロサービス（Command、Query、Projection、Search）に分解されることで、
高い可用性とスケーラビリティを実現する。

## 3. Strategic Classification

- **Domain Type**: Supporting Subdomain
- **Business Model**: Revenue Enabler
- **Evolution Stage**: Custom Built

### 分類の理由

- **Supporting Subdomain**: 学習機能を支える重要な基盤だが、ビジネスの差別化要因は Learning/Algorithm Context にある
- **Revenue Enabler**: 豊富で質の高い語彙データが学習効果を向上させ、ユーザー満足度と継続率に貢献
- **Custom Built**: 既存の辞書 API では実現できない、学習に特化した詳細情報と編集可能性を実装

## 4. Domain Roles

- **Content Management**: 語彙データの作成・更新・削除
- **Repository Context**: 全ユーザーが共有するグローバル辞書
- **Integration Hub**: AI サービスとの連携による自動コンテンツ生成

### 役割の詳細

| 役割               | 説明                                         |
| ------------------ | -------------------------------------------- |
| Content Management | 語彙項目の CRUD 操作、バージョン管理         |
| Repository Context | グローバル辞書として検索・参照機能を提供     |
| Integration Hub    | AI Integration Context と連携してコンテンツ生成 |

## 5. Inbound Communication

### メッセージ/イベント

| 名前                    | 送信元                 | 契約タイプ | 説明                             |
| ----------------------- | ---------------------- | ---------- | -------------------------------- |
| ItemDetailsRequested    | Learning Context       | 同期       | 学習項目の詳細情報要求           |
| CreateVocabularyEntry   | User Context           | 同期       | 新規語彙エントリの作成           |
| UpdateVocabularyItem    | User Context           | 同期       | 語彙項目の更新（楽観的ロック）   |
| AIGenerationCompleted   | AI Integration Context | 非同期     | AI 生成コンテンツの受信          |

### 統合パターン

- Learning Context: Partnership（双方向の密接な協調）
- User Context: Customer-Supplier（User が Supplier）
- AI Integration Context: Partnership（非同期連携）

## 6. Outbound Communication

### メッセージ/イベント

| 名前                     | 宛先                   | 契約タイプ | 説明                          |
| ------------------------ | ---------------------- | ---------- | ----------------------------- |
| VocabularyEntryCreated   | AI Integration Context | 非同期     | 新規エントリの AI 生成要求     |
| VocabularyItemUpdated    | Progress Context       | 非同期     | 語彙項目の更新通知            |
| RequestAIGeneration      | AI Integration Context | 非同期     | AI コンテンツ生成要求         |
| ItemDetails              | Learning Context       | 同期       | 語彙項目の詳細情報レスポンス   |
| TaskCreated              | AI Integration Context | 非同期     | AI タスク作成通知（Task ID は AI Context が管理） |

### 統合パターン

- AI Integration Context: Partnership（AI 生成の連携）
- Progress Context: Published Language（イベント公開）
- Learning Context: Partnership（ItemDetails は同期レスポンス）

## 7. Ubiquitous Language

### 主要な用語

| 用語                | 英語                  | 定義                                      |
| ------------------- | --------------------- | ----------------------------------------- |
| 語彙エントリ        | Vocabulary Entry      | 一つのスペリングを表す（例："apple"）     |
| 語彙項目            | Vocabulary Item       | 特定の意味・用法（例："apple (fruit)"）   |
| 曖昧さ回避          | Disambiguation        | 同じスペリングの異なる意味を区別する記号  |
| 楽観的ロック        | Optimistic Lock       | バージョン番号による並行編集制御          |
| AI 生成コンテンツ   | AI Generated Content  | AI が自動生成した語彙情報                 |
| グローバル辞書      | Global Dictionary     | 全ユーザーが共有する語彙データベース      |
| 自動マージ          | Auto Merge            | 競合しない変更を自動的に統合             |

### ドメインコンセプト

Wikipedia 方式により、1 つのスペリングに複数の意味を持たせることができる。AI による自動生成と人手による編集を組み合わせ、豊富で正確な語彙情報を提供する。楽観的ロックと自動マージにより、複数ユーザーによる並行編集を効率的に処理する。

## 8. Business Decisions

### 主要なビジネスルール

1. 同一スペリング・同一曖昧さ回避の項目は作成不可
2. 中身が完全に空の項目のみ自動 AI 生成の対象
3. ユーザーによる手動編集を優先（AI は上書きしない）
4. 異なるフィールドへの変更は自動マージ可能
5. バージョン不一致でも競合しない場合は更新を許可

### ポリシー

- **コンテンツの質**: AI 生成と人手編集のベストミックス
- **編集の自由度**: ユーザーはいつでも再生成・編集可能
- **協調編集**: 可能な限り自動マージで編集を継続

### AI 生成ルール

```
自動生成の条件:
- 新規作成された空の項目
- すべてのコンテンツフィールドが空

手動再生成:
- ユーザーの明示的な要求があれば常に実行
- 既存コンテンツの充実度に関わらず上書き
```

## 9. Assumptions

### 技術的前提

- AI サービスは非同期で動作し、遅延やエラーが発生しうる
- AI タスク管理（ID、状態追跡）は AI Integration Context が担当
- 楽観的ロックのためのバージョン管理が可能
- 全文検索インデックスの構築が可能
- 大規模データ（数万〜数十万項目）への対応

### ビジネス的前提

- ユーザーは語彙情報の正確性を重視する
- AI 生成の品質は学習に十分なレベル
- 複数ユーザーによる協調編集のニーズがある
- Wikipedia スタイルの意味管理が直感的

## 10. Verification Metrics

### 定量的指標

| メトリクス         | 目標値      | 測定方法                        |
| ------------------ | ----------- | ------------------------------- |
| AI 生成成功率      | 95% 以上    | 成功生成数 / 生成要求数         |
| 自動マージ成功率   | 80% 以上    | 自動マージ数 / 競合発生数       |
| 検索応答時間       | 50ms 以内   | 95 パーセンタイル応答時間       |
| コンテンツ充実度   | 90% 以上    | 主要フィールド入力済み項目の割合 |

### 定性的指標

- 語彙情報の正確性と有用性
- 編集インターフェースの使いやすさ
- AI 生成コンテンツの品質
- 検索結果の関連性

## 11. Service Architecture

### マイクロサービス構成

Vocabulary Context は以下の 4 つのサービスに分解される：

1. **vocabulary-command-service**
   - VocabularyEntry/VocabularyItem 集約の管理
   - コマンドの検証と実行
   - ドメインイベントの生成
   - Event Store への永続化

2. **vocabulary-query-service**
   - 基本的な読み取り操作
   - 項目詳細の取得
   - エントリー一覧の取得
   - シンプルなフィルタリング

3. **vocabulary-search-service**
   - Meilisearch を活用した全文検索
   - Typo 許容、部分一致検索
   - オートコンプリート
   - フィルタリング機能（status, CEFR レベル等）

4. **vocabulary-projection-service**
   - イベントの消費と処理
   - Read Model の更新
   - Search Index の更新
   - イベント順序の保証

### 技術選定

- **Event Store**: PostgreSQL（イベント永続化）
- **Event Bus**: Google Pub/Sub（サービス間通信）
- **Read Model**: PostgreSQL（基本データ）
- **Search Engine**: Meilisearch（高度な検索）
- **Cache**: Redis（クエリ結果キャッシュ、5分 TTL）
- **Asset Storage**: Google Cloud Storage（画像・音声ファイル）
- **Deployment**: Google Cloud Run

## 12. Open Questions

### 設計上の疑問

- [x] 大規模データでの検索性能の最適化方法は？→ Meilisearch 採用
- [x] 変更履歴の保持期間とアーカイブ戦略は？→ Event Sourcing で全履歴保持
- [x] 画像（イラスト）の管理をどうするか？→ Google Cloud Storage
- [x] 発音音声ファイルの管理方法は？→ Google Cloud Storage
- [x] 例文の難易度判定は必要か？→ 不要（CEFR レベルで十分）
- [x] コロケーションの自動抽出は可能か？→ 実装しない（AI 生成に含める）

### 実装上の課題

- [x] AI タスク ID と生成状態の永続化方法は？→ Event Store に記録
- [x] AI 生成の優先順位付けアルゴリズムは？→ FIFO（デフォルト）
- [x] 外部辞書 API との連携は必要か？→ 不要（AI 生成で十分）

---

## 改訂履歴

- 2025-08-05: インフラストラクチャ設計決定を反映（GCS、AI 生成設計）
- 2025-08-03: CQRS/Event Sourcing 実装詳細を追加
- 2025-07-30: AI Integration Context の非同期化に伴う更新（TaskCreatedAck 追加、AI タスク ID 管理を明記）
- 2025-07-30: 初版作成
