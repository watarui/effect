# Effect プロジェクト - 次の実装ステップ

## Step 1: ローカル開発環境の確認（30分）

1. Docker インフラの起動

   ```bash
   make up-infra
   ```

2. 接続確認
   - 各 PostgreSQL への接続テスト
   - Redis の動作確認
   - Pub/Sub エミュレータの確認

3. ビルド環境の確認

   ```bash
   make build
   make test
   ```

## Step 2: 共通ライブラリの実装（Phase 1: 基盤構築）

### 2.1 common-types の実装

1. 強型 ID の実装
   - UserId, ItemId, SessionId, EventId
   - UUID ベースの実装
   - シリアライゼーション対応

2. エラー型の定義
   - DomainError, DomainResult
   - thiserror による実装

3. タイムスタンプ関連
   - Timestamp 型の実装
   - タイムゾーン処理（JST固定）

### 2.2 domain-events の実装

1. 基本トレイトの定義
   - DomainEvent トレイト
   - EventMetadata 構造体
   - EventHandler, EventBus, EventStore トレイト

2. 各コンテキストのイベント定義
   - learning.rs: SessionStarted, SessionCompleted など
   - vocabulary.rs: ItemCreated, ItemUpdated など
   - 他のコンテキストのイベント

### 2.3 infrastructure の実装

1. データベース接続
   - PostgreSQL 接続プール（sqlx）
   - 接続設定の管理

2. イベントバス実装
   - Pub/Sub クライアントの実装
   - トピック管理とサブスクリプション

## Step 3: Learning Service の実装（TDD）

1. ドメイン層から開始
   - LearningSession 集約のテスト作成
   - ビジネスロジックの実装

2. リポジトリ層
   - PostgreSQL リポジトリ実装
   - イベントストアへの保存

3. gRPC サービス
   - Proto ファイル定義
   - サービス実装

## Step 4: 動作確認とイベントフロー

1. 単体での動作確認
   - Learning Service の起動
   - 基本的な API テスト

2. イベント発行の確認
   - Event Processor の実装
   - Pub/Sub でのイベント送受信テスト

## 完了したらチェック

- [ ] Step 1: ローカル環境の動作確認
- [ ] Step 2.1: common-types 実装
- [ ] Step 2.2: domain-events 実装
- [ ] Step 2.3: infrastructure 実装
- [ ] Step 3: Learning Service 実装
- [ ] Step 4: イベントフロー確認
