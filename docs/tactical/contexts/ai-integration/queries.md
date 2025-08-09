# AI Integration Context - クエリ

## 概要

AI Integration Context のクエリは、タスク状態の取得、処理結果の照会、使用統計の提供を担当します。リアルタイムステータスと履歴データの両方をサポートします。

## 主要クエリ

### 1. GetTaskStatus

**目的**: タスクの現在の状態を取得

**パラメータ**:

- task_id: タスク識別子

**返却データ**:

- task_id: タスク識別子
- status: "pending" | "processing" | "completed" | "failed" | "cancelled"
- created_at: 作成日時
- started_at: 開始日時
- completed_at: 完了日時
- progress: 進捗率（0-100）
- estimated_completion: 推定完了時間

### 2. GetTaskResult

**目的**: 完了したタスクの結果を取得

**パラメータ**:

- task_id: タスク識別子

**返却データ**:

- task_id: タスク識別子
- result_type: 結果タイプ
- content: 生成されたコンテンツ
- metadata: メタデータ
- tokens_used: 使用トークン数
- cost_estimate: 推定コスト
- provider_used: 使用プロバイダー

### 3. GetUserTasks

**目的**: ユーザーのタスク一覧を取得

**パラメータ**:

- user_id: ユーザー識別子
- status_filter: ステータスフィルタ（オプション）
- date_range: 期間指定（オプション）
- limit: 取得件数
- offset: オフセット

**返却データ**:

- tasks: タスク配列
  - task_id: タスク識別子
  - task_type: タスクタイプ
  - status: ステータス
  - created_at: 作成日時
- total_count: 総件数
- has_more: 追加データの有無

### 4. GetChatSession

**目的**: チャットセッションの詳細を取得

**パラメータ**:

- session_id: セッション識別子

**返却データ**:

- session_id: セッション識別子
- user_id: ユーザー識別子
- item_id: 対象項目識別子
- status: セッションステータス
- started_at: 開始日時
- last_activity: 最終活動日時
- message_count: メッセージ数

### 5. GetChatHistory

**目的**: チャットセッションの会話履歴を取得

**パラメータ**:

- session_id: セッション識別子
- limit: 取得件数（デフォルト: 50）
- before: この日時より前のメッセージ

**返却データ**:

- messages: メッセージ配列
  - message_id: メッセージ識別子
  - role: "user" | "assistant"
  - content: メッセージ内容
  - timestamp: タイムスタンプ
  - tokens_used: 使用トークン数

### 6. GetQueueStatus

**目的**: タスクキューの状態を取得

**パラメータ**:

- queue_type: キュータイプ（オプション）

**返却データ**:

- pending_count: 待機中タスク数
- processing_count: 処理中タスク数
- average_wait_time: 平均待ち時間
- estimated_processing_time: 推定処理時間
- worker_count: アクティブワーカー数
- worker_utilization: ワーカー使用率

### 7. GetProviderStatus

**目的**: AI プロバイダーの状態を取得

**パラメータ**:

- provider: プロバイダー名（オプション、全プロバイダー）

**返却データ**:

- providers: プロバイダー状態の配列
  - name: プロバイダー名
  - status: "available" | "degraded" | "unavailable"
  - circuit_breaker: "closed" | "open" | "half_open"
  - success_rate: 成功率
  - average_latency: 平均レイテンシ
  - rate_limit_remaining: 残りレート制限

### 8. GetUsageStats

**目的**: AI 機能の使用統計を取得

**パラメータ**:

- user_id: ユーザー識別子（オプション）
- period: "day" | "week" | "month"
- group_by: "provider" | "task_type" | "date"

**返却データ**:

- total_tasks: 総タスク数
- successful_tasks: 成功タスク数
- failed_tasks: 失敗タスク数
- total_tokens: 総トークン数
- total_cost: 総コスト
- breakdown: グループ別の内訳
  - group_key: グループキー
  - count: 件数
  - tokens: トークン数
  - cost: コスト

### 9. GetCostEstimate

**目的**: タスクのコスト見積もりを取得

**パラメータ**:

- task_type: タスクタイプ
- estimated_tokens: 推定トークン数
- provider: プロバイダー（オプション）

**返却データ**:

- estimated_cost: 推定コスト
- cost_breakdown: コスト内訳
  - input_tokens: 入力トークン
  - output_tokens: 出力トークン
  - base_cost: 基本コスト
- provider_comparison: プロバイダー別比較

### 10. GetTaskErrors

**目的**: タスクのエラー情報を取得

**パラメータ**:

- task_id: タスク識別子（特定タスク）
- date_range: 期間指定（エラー一覧）
- error_type: エラータイプフィルタ

**返却データ**:

- errors: エラー配列
  - task_id: タスク識別子
  - error_type: エラータイプ
  - error_message: エラーメッセージ
  - occurred_at: 発生日時
  - retry_count: リトライ回数
  - is_resolved: 解決済みフラグ

## リアルタイムクエリ

### SubscribeToTaskUpdates

**目的**: タスクの更新をリアルタイムで購読

**パラメータ**:

- task_ids: 購読するタスク ID 配列
- event_types: 購読するイベントタイプ

**返却データ** (WebSocket/SSE):

- event_type: イベントタイプ
- task_id: タスク識別子
- data: イベントデータ
- timestamp: タイムスタンプ

## キャッシング戦略

| クエリ | キャッシュ期間 | 無効化タイミング |
|--------|--------------|----------------|
| GetTaskStatus (完了) | 24時間 | - |
| GetTaskResult | 7日間 | - |
| GetProviderStatus | 30秒 | 状態変更時 |
| GetUsageStats | 5分 | 新タスク完了時 |
| GetQueueStatus | リアルタイム | - |
