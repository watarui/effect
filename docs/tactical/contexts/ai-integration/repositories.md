# AI Integration Context - リポジトリ設計

## 概要

AI Integration Context には 3 つの主要な集約が存在します：

- `AIGenerationTask`：AI による各種生成タスクの管理
- `ChatSession`：深掘りチャット機能のセッション管理
- `TaskQueue`：非同期タスクキューの管理

このコンテキストは Anti-Corruption Layer パターンを実装し、外部 AI サービスとの統合を担当します。
完全非同期処理により、大量の AI 要求を効率的に処理し、WebSocket/SSE によるリアルタイム通知を提供します。

## AIGenerationTaskRepository

AI 生成タスクの永続化を担当するリポジトリです。

### 主要な責務

- タスクの基本的な CRUD 操作
- ステータス別のタスク検索
- タスクキューの管理
- リトライ対象の特定
- 使用統計の集計

### インターフェース設計

**基本的な CRUD 操作**:

- `find_by_id`: ID でタスクを取得
- `save`: タスクを保存（新規作成または更新）
- `delete`: タスクを削除（通常は論理削除）

**ステータス別クエリ**:

- `find_pending_tasks`: 未処理のタスクを取得
- `find_processing_tasks`: 処理中のタスクを取得
- `find_failed_tasks`: 失敗したタスクを取得（リトライ用）
- `find_timed_out_tasks`: タイムアウトしたタスクを取得

**ユーザー関連クエリ**:

- `find_by_user`: ユーザーのタスクを取得
- `count_active_by_user`: ユーザーのアクティブなタスク数を取得

**タスクタイプ別クエリ**:

- `find_by_type`: タスクタイプ別に取得
- `find_by_related_entity`: 関連エンティティで取得

**キューイング関連**:

- `claim_next_pending_task`: 次の未処理タスクを取得してロック
- `release_task`: タスクのロックを解放

**統計関連**:

- `count_by_status`: ステータス別のタスク数を取得
- `calculate_average_processing_time`: 平均処理時間を計算
- `get_provider_statistics`: プロバイダー別の統計を取得

## ChatSessionRepository

チャットセッションの永続化を担当するリポジトリです。

### 主要な責務

- セッションの基本的な CRUD 操作
- アクティブセッションの管理
- メッセージ履歴の保存
- セッションタイムアウトの処理
- 使用統計の追跡

### インターフェース設計

**基本的な CRUD 操作**:

- `find_by_id`: ID でセッションを取得
- `save`: セッションを保存
- `delete`: セッションを削除

**セッション管理**:

- `find_active_by_user`: ユーザーのアクティブセッションを取得
- `find_by_user_and_item`: ユーザーと項目でセッションを取得
- `find_inactive_sessions`: 非アクティブセッションを取得（タイムアウト処理用）

**メッセージ操作**:

- `add_message`: メッセージを追加
- `get_message_history`: メッセージ履歴を取得
- `count_messages`: メッセージ数を取得

**統計関連**:

- `count_active_sessions`: アクティブセッション数を取得
- `calculate_average_session_length`: 平均セッション長を計算
- `get_token_usage_by_user`: ユーザー別トークン使用量を取得

## TaskQueueRepository

非同期タスクキューの管理を担当するリポジトリです。

### 主要な責務

- タスクの優先度管理
- プロバイダー別のキュー管理
- デッドレターキューの処理
- バックプレッシャーの実装

### インターフェース設計

**キュー操作**:

- `enqueue`: タスクをキューに追加
- `dequeue`: 次のタスクを取得
- `peek`: キューの先頭を確認（取得せず）
- `remove`: 特定のタスクをキューから削除

**優先度管理**:

- `enqueue_with_priority`: 優先度付きでエンキュー
- `reorder_by_priority`: 優先度で再順序付け

**キューステータス**:

- `get_queue_length`: キューの長さを取得
- `get_queue_stats`: キューの統計情報を取得
- `is_queue_full`: キューが満杯かチェック

**デッドレターキュー**:

- `move_to_dlq`: デッドレターキューに移動
- `get_dlq_items`: DLQ のアイテムを取得
- `retry_from_dlq`: DLQ から再試行

## 実装上の考慮事項

### 非同期処理の実装

- タスクの atomic な claim 操作
- 分散ロックによる競合回避
- タイムアウトによる自動ロック解放

### プロバイダー管理

- プロバイダー別のレート制限追跡
- Circuit Breaker の状態管理
- フォールバック戦略の実装

### パフォーマンス最適化

- 頻繁なステータス更新のバッチ化
- インデックスの適切な設定
- 長時間実行タスクの分離

### コスト追跡

- トークン使用量の記録
- プロバイダー別コストの集計
- 月次制限のリアルタイム追跡
