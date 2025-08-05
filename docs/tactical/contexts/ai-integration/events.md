# AI Integration Context - ドメインイベント

## 概要

AI Integration Context で発生するドメインイベントのカタログです。AI タスクのライフサイクル、チャットセッション、外部サービスとの連携に関するイベントを管理します。

## イベント一覧

| イベント名 | 説明 | 発生タイミング |
|-----------|------|-------------|
| AITaskCreated | AI タスクが作成された | 新規タスクが受け付けられた時 |
| AITaskStarted | AI タスクの処理が開始された | タスクがキューから取り出された時 |
| AITaskCompleted | AI タスクが完了した | 正常に処理が完了した時 |
| AITaskFailed | AI タスクが失敗した | エラーが発生し、リトライも失敗した時 |
| AITaskRetried | AI タスクがリトライされた | エラー後の再試行時 |
| AITaskCancelled | AI タスクがキャンセルされた | ユーザーによるキャンセル時 |
| ChatSessionStarted | チャットセッションが開始された | 深掘りチャット開始時 |
| ChatMessageAdded | チャットメッセージが追加された | ユーザーまたは AI の発言時 |
| ChatSessionClosed | チャットセッションが終了した | セッション終了時 |

## イベント詳細

### 1. AITaskCreated

AI タスクが作成されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- task_id: タスク識別子
- task_type: タスクタイプ
- requested_by: 要求者
- request_content: リクエスト内容
- priority: 優先度

**発生条件**:

- Vocabulary Context から項目生成要求を受信した時
- Learning Context からテストカスタマイズ要求を受信した時
- ユーザーから画像生成要求があった時

### 2. AITaskStarted

AI タスクの処理が開始されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- task_id: タスク識別子
- provider: 使用する AI プロバイダー
- estimated_duration: 推定処理時間

**発生条件**:

- タスクがキューから取り出され、処理が開始された時
- 適切なプロバイダーが選択された時

### 3. AITaskCompleted

AI タスクが正常に完了したことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- task_id: タスク識別子
- response_content: 生成されたコンテンツ
- tokens_used: 使用トークン数
- processing_time: 処理時間
- cost_estimate: 推定コスト

**発生条件**:

- AI による生成が正常に完了した時
- レスポンスの検証が成功した時

### 4. AITaskFailed

AI タスクが失敗したことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- task_id: タスク識別子
- error_type: エラータイプ
- error_message: エラーメッセージ
- retry_count: 試行回数
- is_final: 最終失敗かどうか

**エラータイプ**:

- ProviderError: プロバイダー側のエラー
- RateLimitExceeded: レート制限超過
- InvalidRequest: 無効なリクエスト
- Timeout: タイムアウト
- InsufficientCredits: クレジット不足

### 5. ChatSessionStarted

チャットセッションが開始されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- session_id: セッション識別子
- user_id: ユーザー識別子
- item_id: 対象項目識別子
- initial_context: 初期コンテキスト

**発生条件**:

- ユーザーが項目の深掘りチャットを開始した時
- 必要なコンテキストが準備された時

### 6. ChatMessageAdded

チャットメッセージが追加されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- session_id: セッション識別子
- message_id: メッセージ識別子
- role: 発言者（User, Assistant）
- content: メッセージ内容
- tokens_used: 使用トークン数

**発生条件**:

- ユーザーが質問を投稿した時
- AI が回答を生成した時

## 他コンテキストへのイベント

### Vocabulary Context へ

**AIGenerationCompleted** (Integration Event として変換):

- 生成された項目情報の通知
- item_id と生成コンテンツを含む

### Learning Context へ

**TestCustomizationCompleted**:

- カスタマイズされたテスト内容の通知
- session_id と調整された項目リストを含む

### Progress Context へ

**AIUsageRecorded**:

- AI 機能の使用統計
- 使用回数、トークン数、コストなど

## イベント処理の考慮事項

### 非同期処理

- すべてのタスクは非同期で処理
- イベントによる進捗通知
- WebSocket/SSE でリアルタイム更新

### エラーハンドリング

- 自動リトライは最大3回
- Circuit Breaker による保護
- フォールバックプロバイダーへの切り替え

### コスト管理

- トークン使用量の追跡
- 月次制限のチェック
- 優先度に基づく処理順序
