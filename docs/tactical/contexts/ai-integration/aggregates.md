# AI Integration Context - 集約設計

## 概要

AI Integration Context は、Effect プロジェクトにおける外部 AI サービスへのゲートウェイとして機能します。
OpenAI、Gemini などの AI サービスを抽象化し、Anti-Corruption Layer パターンを実装して、内部ドメインを外部 API の変更から保護します。

### 主要な責務

- **項目情報生成**: 語彙項目の意味、例文、発音などを AI で生成
- **テストカスタマイズ**: 自然言語による指示を解釈してテスト内容を調整
- **深掘りチャット**: 特定の項目について詳細な説明を提供
- **画像生成/検索**: 項目に関連するイメージ画像の取得（AI 生成または画像素材サービス）

### 設計方針

- **機能別インターフェース**: 各機能に最適なプロバイダーを選択可能
- **非同期処理**: イベント駆動アーキテクチャによる完全非同期処理
- **リアルタイム通知**: WebSocket/SSE による進捗・完了通知
- **エラー回復**: Circuit Breaker とリトライによる安定性確保
- **Anti-Corruption Layer**: 外部 API の詳細を内部ドメインから隠蔽

## 集約の設計

### 1. AIGenerationTask（AI 生成タスク）- 集約ルート

AI による各種生成タスクを管理します。

**主要な属性**:

- task_id: タスク識別子
- task_type: タスクタイプ（ItemInfoGeneration、TestCustomization、ImageGeneration）
- status: タスクステータス（Pending、Processing、Completed、Failed、Cancelled）
- requested_by: 要求者
- requested_at: 要求日時
- request_content: リクエスト内容
- response: レスポンス情報
- completed_at: 完了日時
- error: エラー情報
- retry_count: リトライ回数

**タスクタイプ**:

- ItemInfoGeneration: 語彙項目情報の生成
- TestCustomization: テスト内容のカスタマイズ
- ImageGeneration: 画像の生成/検索

**不変条件**:

- 完了後のタスクは変更不可
- リトライ回数は最大3回まで
- キャンセルは Processing 状態でのみ可能

### 2. ChatSession（チャットセッション）- 集約ルート

深掘りチャット機能のセッションを管理します。

**主要な属性**:

- session_id: セッション識別子
- user_id: ユーザー識別子
- item_id: 対象項目識別子
- messages: 会話履歴
- started_at: 開始日時
- last_activity: 最終活動日時
- status: セッションステータス
- context: チャットコンテキスト

**ChatMessage（値オブジェクト）**:

- message_id: メッセージ識別子
- role: 役割（User、Assistant、System）
- content: メッセージ内容
- timestamp: タイムスタンプ
- tokens_used: 使用トークン数

**セッション管理**:

- 30分間操作がない場合は自動的にクローズ
- 会話履歴は最大50メッセージまで保持
- コンテキストは項目情報と学習履歴を含む

## ドメインサービス

### AIServiceAdapter

外部 AI サービスとの統合を抽象化するアダプター。

**責務**:

- プロバイダーの選択と切り替え
- リクエスト/レスポンスの変換
- エラーハンドリングとリトライ
- レート制限の管理

**サポートするプロバイダー**:

- OpenAI (GPT-4, DALL-E)
- Google Gemini
- Anthropic Claude
- 画像素材サービス（Unsplash等）

## 設計上の重要な決定

### 完全非同期処理

すべての AI 処理は非同期で実行：

1. タスクの受付と即座の TaskId 返却
2. バックグラウンドでの処理実行
3. 完了時のイベント発行とコールバック

### Anti-Corruption Layer の実装

外部 API の詳細を隠蔽：

- プロバイダー固有のデータ構造を内部モデルに変換
- API の変更が内部ドメインに影響しない設計
- 統一されたエラーハンドリング

### エラー回復戦略

1. **Circuit Breaker**: 連続失敗時の自動遮断
2. **指数バックオフ**: リトライ間隔の段階的増加
3. **フォールバック**: 代替プロバイダーへの切り替え
4. **部分的成功**: 一部の処理失敗を許容

### コスト管理

- ユーザー/月ごとの使用量制限
- トークン数の事前計算と警告
- 優先度に基づくリクエスト処理
