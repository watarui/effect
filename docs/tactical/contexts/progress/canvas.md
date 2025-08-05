# Progress Context Bounded Context Canvas

## 1. Name

Progress Context

## 2. Purpose

学習活動の完全な履歴を保持し、あらゆる角度から学習進捗を可視化する「学習活動の鏡」として機能する。
純粋な CQRS/Event Sourcing の実装により、過去の任意時点の状態再現と柔軟な統計分析を実現する。
Progress Context は 3 つのマイクロサービスに分解され、書き込みと読み取りの独立したスケーリングを実現する。

## 3. Strategic Classification

- **Domain Type**: Supporting Domain
- **Business Model**: Information Provider
- **Evolution Stage**: Custom Built

### 分類の理由

- **Supporting Domain**: 学習の中核機能ではないが、モチベーション維持と学習効果の可視化に不可欠
- **Information Provider**: 統計情報とインサイトを提供し、ユーザーエンゲージメントを向上
- **Custom Built**: 純粋な CQRS/イベントソーシングという高度な実装パターンを採用

## 4. Domain Roles

- **Event Aggregator**: 複数コンテキストからのイベントを集約
- **Read Model Provider**: 様々な切り口でのデータ投影を提供
- **Analytics Engine**: 統計計算と傾向分析

### 役割の詳細

| 役割                | 説明                                           |
| ------------------- | ---------------------------------------------- |
| Event Aggregator    | 他の全コンテキストからイベントを収集・永続化   |
| Read Model Provider | GraphQL クエリに最適化された投影を生成・提供   |
| Analytics Engine    | 日別・カテゴリ別・レベル別の統計を計算        |

## 5. Inbound Communication

### メッセージ/イベント

| 名前                    | 送信元                     | 契約タイプ | 説明                                 |
| ----------------------- | -------------------------- | ---------- | ------------------------------------ |
| SessionStarted          | Learning Context           | 非同期     | 学習セッション開始の通知             |
| SessionCompleted        | Learning Context           | 非同期     | セッション完了と結果の通知           |
| CorrectnessJudged       | Learning Context           | 非同期     | 個別項目の正誤判定結果               |
| ReviewScheduleUpdated   | Learning Algorithm Context | 非同期     | 復習スケジュール更新の通知           |
| DifficultyAdjusted      | Learning Algorithm Context | 非同期     | 難易度係数調整の通知                 |
| VocabularyItemUpdated   | Vocabulary Context         | 非同期     | 語彙項目更新の通知                   |
| UserCreated             | User Context               | 非同期     | 新規ユーザー作成の通知               |

### 統合パターン

- すべて Published Language（イベント購読）パターン
- 他のコンテキストが発行するイベントを受動的に収集
- 集約は持たず、Read Model のみを管理

## 6. Outbound Communication

### メッセージ/イベント

| 名前                     | 宛先         | 契約タイプ | 説明                               |
| ------------------------ | ------------ | ---------- | ---------------------------------- |
| DailyStatsResponse       | GraphQL API  | 同期       | 日別統計データの提供               |
| CategoryProgressResponse | GraphQL API  | 同期       | カテゴリ別進捗データ（CEFR レベル分布含む） |
| UserSummaryResponse      | GraphQL API  | 同期       | 全体サマリー（進捗スコア 0-100 含む）     |
| LearningStreakResponse   | GraphQL API  | 同期       | 連続学習記録の提供                 |

### 統合パターン

- GraphQL Resolver を通じた同期的なデータ提供
- キャッシュを活用した高速レスポンス
- 複雑なクエリに対する最適化された投影

## 7. Ubiquitous Language

### 主要な用語

| 用語                   | 英語                  | 定義                                      |
| ---------------------- | --------------------- | ----------------------------------------- |
| 投影                   | Projection            | イベントから導出される特定視点のデータ     |
| イベントストア         | Event Store           | すべてのドメインイベントを保持する永続化層 |
| 日別統計               | Daily Stats           | 1日単位で集計された学習活動               |
| カテゴリ別進捗         | Category Progress     | スキル・レベル別の習熟度                  |
| ストリーク             | Learning Streak       | 連続学習日数                              |
| チェックポイント       | Checkpoint            | イベント処理の進行状況記録                |
| 遅延評価               | Lazy Evaluation       | アクセス時に初めて計算する戦略            |

### ドメインコンセプト

すべての学習活動をイベントとして記録し、様々な視点（日別、カテゴリ別、レベル別）から学習進捗を可視化する。集約を持たない純粋な Read Model として、柔軟で拡張可能な統計分析基盤を提供する。

## 8. Business Decisions

### 主要なビジネスルール

1. 日別統計はリアルタイム更新（即座に反映）
2. カテゴリ統計は 5 分ごとのバッチ更新
3. ストリークは 1 日の猶予を許可（時差考慮）
4. 統計データは 1 年間保持、それ以降はアーカイブ
5. 進捗スコアは習熟率・正答率・カバー率から算出（0-100）

### ポリシー

- **結果整合性**: リアルタイム性より正確性を優先
- **スケーラビリティ**: 読み取り負荷に応じたキャッシュ戦略
- **拡張性**: 新しい統計軸の追加が容易な設計

### 更新戦略

```
リアルタイム更新:
- DailyStats（今日の統計）
- LearningStreak（連続記録）

バッチ更新（5分間隔）:
- CategoryProgress
- SkillDistribution

遅延評価（アクセス時）:
- MonthlyTrends
```

## 9. Assumptions

### 技術的前提

- イベントストアは追記のみ（イミュータブル）
- イベントの順序は保証される
- スナップショットによる状態復元が可能
- 大量のイベントを効率的に処理できる

### ビジネス的前提

- ユーザーは統計の若干の遅延を許容する
- 過去の学習履歴の完全性が重要
- 統計の正確性がモチベーションに影響
- 新しい分析軸が将来追加される

## 10. Verification Metrics

### 定量的指標

| メトリクス           | 目標値      | 測定方法                         |
| -------------------- | ----------- | -------------------------------- |
| イベント処理遅延     | 5 秒以内    | 最新イベントと処理済み位置の差   |
| クエリ応答時間       | 100ms 以内  | GraphQL クエリの 95 パーセンタイル |
| プロジェクション精度 | 99.9% 以上  | 再計算による検証                 |
| キャッシュヒット率   | 80% 以上    | キャッシュアクセス統計           |

### 定性的指標

- 統計データの信頼性
- 新しい分析軸の追加容易性
- イベント再生による監査可能性
- システム全体の可観測性向上

## 11. Service Architecture

### マイクロサービス構成

Progress Context は以下の 3 つのサービスに分解される：

1. **progress-command-service**
   - イベント受信と Event Store への永続化
   - イベント順序保証
   - Google Pub/Sub へのイベント発行

2. **progress-query-service**
   - Read Model からの基本的な読み取り操作
   - GraphQL API の提供
   - Redis によるキャッシング

3. **progress-projection-service**
   - Event Store からのイベント消費
   - Read Model の構築・更新
   - 統計集計とマテリアライズドビューの管理

### 技術選定

- **Event Store**: PostgreSQL（イベント永続化）
- **Event Bus**: Google Pub/Sub（サービス間通信）
- **Read Model**: PostgreSQL（統計データ）
- **Cache**: Redis（クエリ結果キャッシュ）
- **Deployment**: Google Cloud Run

## 12. Open Questions

### 設計上の疑問

- [x] イベントストアの永続化技術は何を使うか？→ PostgreSQL に決定
- [x] 投影の再構築時のダウンタイムをどう回避するか？→ Blue-Green デプロイメント
- [ ] スナップショットの作成頻度とタイミングは？
- [ ] 古いイベントのアーカイブ戦略は？
- [ ] IELTS/TOEFL スコア推定機能は必要か？
- [ ] CEFR レベル表示と進捗スコアで十分か？

### 実装上の課題

- [x] 大量イベントの効率的な処理方法は？→ バッチ処理とストリーミング
- [x] 複数の投影を並行更新する際の一貫性保証は？→ 楽観的ロック
- [ ] GraphQL のネストしたクエリの最適化は？
- [ ] GDPR 対応（ユーザーデータ削除）はどう実装するか？

---

## 改訂履歴

- 2025-08-03: CQRS/Event Sourcing 実装詳細を追加
- 2025-07-30: 初版作成
