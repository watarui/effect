# AI Integration Context Bounded Context Canvas

## 1. Name

AI Integration Context

## 2. Purpose

外部 AI サービス（OpenAI、Gemini など）への統合ゲートウェイとして機能し、Anti-Corruption Layer パターンで内部ドメインを保護する。
項目情報生成、テストカスタマイズ、深掘りチャット、画像生成などの AI 機能を提供する。

## 3. Strategic Classification

- **Domain Type**: Generic Subdomain
- **Business Model**: Revenue Enabler
- **Evolution Stage**: Commodity/Partner Supplied

### 分類の理由

- **Generic Subdomain**: AI サービスの統合は一般的な技術的課題であり、ビジネスの差別化要因ではない
- **Revenue Enabler**: AI による自動生成が学習コンテンツの質を向上させ、ユーザー価値を高める
- **Commodity/Partner Supplied**: 外部 AI サービスを利用し、自社で AI モデルは開発しない

## 4. Domain Roles

- **Service Gateway**: 外部 AI サービスへのゲートウェイ
- **Anti-Corruption Layer**: 外部 API の変更から内部を保護
- **Task Orchestrator**: AI タスクの管理と実行

### 役割の詳細

| 役割                  | 説明                                           |
| --------------------- | ---------------------------------------------- |
| Service Gateway       | 複数の AI プロバイダーへの統一インターフェース |
| Anti-Corruption Layer | 外部 API の詳細を内部ドメインから隠蔽          |
| Task Orchestrator     | リトライ、Circuit Breaker、レート制限の管理    |

## 5. Inbound Communication

### メッセージ/イベント

| 名前                    | 送信元             | 契約タイプ | 説明                                 |
| ----------------------- | ------------------ | ---------- | ------------------------------------ |
| GenerateItemInfoRequest | Vocabulary Context | 非同期     | 語彙項目の詳細情報生成要求           |
| CustomizeTestRequest    | Learning Context   | 非同期     | 自然言語指示によるテストカスタマイズ |
| StartChatSessionRequest | Learning Context   | 非同期     | 深掘りチャットセッション開始         |
| SendChatMessageRequest  | Learning Context   | 非同期     | チャットメッセージ送信               |
| GenerateImageRequest    | Vocabulary Context | 非同期     | 項目関連画像の生成要求               |

### 統合パターン

- Vocabulary Context: Event-Driven Partnership（非同期イベント連携）
- Learning Context: Event-Driven Customer-Supplier（Learning が Customer）
- イベント駆動アーキテクチャによる非同期処理

## 6. Outbound Communication

### メッセージ/イベント

| 名前                  | 宛先               | 契約タイプ | 説明                         |
| --------------------- | ------------------ | ---------- | ---------------------------- |
| TaskCreated           | Requesting Context | 同期       | タスク ID の即座の返却       |
| ItemInfoGenerated     | Vocabulary Context | 非同期     | 生成された項目情報の通知     |
| TestCustomized        | Learning Context   | 非同期     | カスタマイズされたテスト項目 |
| ChatResponseGenerated | Learning Context   | 非同期     | チャットレスポンス           |
| ImageGenerated        | Vocabulary Context | 非同期     | 生成された画像 URL           |
| TaskProgressUpdated   | WebSocket/SSE      | 非同期     | タスク進捗のリアルタイム通知 |
| TaskCompleted         | WebSocket/SSE      | 非同期     | タスク完了のリアルタイム通知 |
| TaskFailed            | WebSocket/SSE      | 非同期     | タスク失敗のリアルタイム通知 |
| ProviderUnavailable   | Progress Context   | 非同期     | プロバイダー障害の通知       |

### 統合パターン

- タスク ID の即座の返却（同期）
- 結果はイベント駆動で非同期通知
- WebSocket/SSE によるリアルタイム進捗更新

## 7. Ubiquitous Language

### 主要な用語

| 用語                   | 英語                   | 定義                                     |
| ---------------------- | ---------------------- | ---------------------------------------- |
| AI 生成タスク          | AI Generation Task     | 各種 AI 生成処理の管理単位               |
| タスクキュー           | Task Queue             | 非同期処理のためのタスク待ち行列         |
| タスク ID              | Task ID                | 非同期タスクの一意識別子                 |
| チャットセッション     | Chat Session           | 深掘りチャットの会話履歴                 |
| プロバイダー           | Provider               | AI サービス提供者（OpenAI、Gemini など） |
| Circuit Breaker        | Circuit Breaker        | 障害時に自動的にサービスを遮断する仕組み |
| レート制限             | Rate Limit             | API 呼び出し頻度の制限                   |
| 機能別インターフェース | Feature Interface      | 用途別に最適化された AI サービス抽象化   |
| リトライ               | Retry                  | 一時的エラー時の再試行                   |
| リアルタイム通知       | Real-time Notification | WebSocket/SSE による即時進捗通知         |
| ワーカー               | Worker                 | タスクキューから処理を実行するプロセス   |

### ドメインコンセプト

機能別インターフェース（ItemInfoGenerator、TestCustomizer など）により、各用途に最適な AI プロバイダーを選択可能にする。
非同期タスクキューとワーカーにより、大量の AI 生成要求を効率的に処理する。
WebSocket/SSE によるリアルタイム通知により、ユーザーは処理の進捗を即座に把握できる。
Circuit Breaker とリトライ機構により、外部サービスの不安定性に対する耐障害性を確保する。

## 8. Business Decisions

### 主要なビジネスルール

1. 非同期処理を基本とし、即座にタスク ID を返却
2. WebSocket/SSE でリアルタイム進捗通知を提供
3. タスクは FIFO キューで処理（優先度付きキューも可能）
4. リトライ可能なエラーは最大 3 回まで再試行
5. Circuit Breaker は 5 回失敗で開き、1 分後に半開状態へ
6. プロバイダー選択は優先順位ベース（将来的にコスト/品質で切り替え）
7. チャットセッションは最大 100 メッセージまで
8. タスクの最大処理時間は 5 分（それ以上は失敗とする）

### ポリシー

- **コスト管理**: 月間予算の 80% で警告、100% で停止
- **品質優先**: デフォルトは高品質プロバイダーを選択
- **プライバシー**: PII（個人情報）の検出と除去

### エラー処理

```
リトライ可能:
- RateLimit
- Timeout
- NetworkError

リトライ不可:
- InvalidRequest
- InsufficientCredits
- ContentFiltered
```

## 9. Assumptions

### 技術的前提

- AI プロバイダーの API が利用可能
- API キーが有効で十分なクレジットがある
- ネットワーク遅延を考慮したタイムアウト設定
- JSON 形式のレスポンスが返される

### ビジネス的前提

- AI 生成品質はユーザーが許容するレベル
- プロバイダーのコストは予算内に収まる
- 非同期処理により複数タスクの並列処理が可能
- リアルタイム通知によりユーザーエンゲージメントが向上
- WebSocket/SSE 接続が安定している

## 10. Verification Metrics

### 定量的指標

| メトリクス           | 目標値      | 測定方法                         |
| -------------------- | ----------- | -------------------------------- |
| タスク受付時間       | 95% < 100ms | 要求からタスク ID 返却までの時間 |
| タスク完了時間       | 95% < 30 秒 | タスク作成から完了までの時間     |
| 並列処理数           | 100 以上    | 同時処理可能なタスク数           |
| 通知遅延             | 95% < 500ms | イベント発生から通知までの遅延   |
| 成功率               | 95% 以上    | 成功タスク数 / 全タスク数        |
| Circuit Breaker 発動 | 月 5 回以下 | プロバイダー別の遮断回数         |
| コスト効率           | 予算内      | 月間 API 利用料金                |

### 定性的指標

- AI 生成コンテンツの品質
- エラーメッセージの分かりやすさ
- プロバイダー切り替えの透明性
- 新プロバイダー追加の容易さ

## 11. Open Questions

### 設計上の疑問

- [ ] タスクキューの永続化戦略は？（Redis、PostgreSQL、など）
- [ ] ワーカーのスケーリング戦略は？（水平スケール、垂直スケール）
- [ ] ストリーミングレスポンス（ChatGPT スタイル）は必要か？
- [ ] AI 生成結果のキャッシュ戦略は？
- [ ] プロンプトエンジニアリングの管理方法は？
- [ ] WebSocket 接続管理の最適な方法は？

### 実装上の課題

- [ ] 複数プロバイダーの同時利用（フォールバック）は必要か？
- [ ] プロバイダー別の最適なモデル選択ロジックは？
- [ ] コスト予測と警告の実装方法は？
- [ ] AI 生成コンテンツの品質評価指標は？

---

## 改訂履歴

- 2025-07-30: 初版作成
- 2025-07-30: 完全非同期処理への変更（イベント駆動アーキテクチャ、WebSocket/SSE によるリアルタイム通知）
