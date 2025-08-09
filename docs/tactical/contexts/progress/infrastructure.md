# Progress Context - インフラストラクチャ

## 概要

Progress Context は 3 つのマイクロサービスで構成され、CQRS/Event Sourcing パターンを実現します。各サービスは独立してスケーリング可能で、責務が明確に分離されています。

## サービス構成

### 1. progress-command-service

**責務**:

- イベントの受信と検証
- Event Store への永続化
- イベント順序の保証
- Pub/Sub へのイベント発行

**技術スタック**:

- Language: Rust
- Framework: Axum
- Database: PostgreSQL (Event Store)
- Message Bus: Google Pub/Sub

**デプロイ設定**:

```yaml
Service: progress-command-service
Platform: Google Cloud Run
CPU: 1
Memory: 512Mi
Min Instances: 1
Max Instances: 10
Concurrency: 100
```

### 2. progress-query-service

**責務**:

- GraphQL API の提供
- Read Model からのデータ取得
- Redis キャッシングの管理
- リアルタイムサブスクリプション

**技術スタック**:

- Language: Rust
- Framework: async-graphql + Axum
- Database: PostgreSQL (Read Model)
- Cache: Redis
- WebSocket: Tungstenite

**デプロイ設定**:

```yaml
Service: progress-query-service
Platform: Google Cloud Run
CPU: 2
Memory: 1Gi
Min Instances: 2
Max Instances: 20
Concurrency: 1000
```

### 3. progress-projection-service

**責務**:

- イベントストリームの消費
- 投影の更新とメンテナンス
- バッチ集計の実行
- スナップショット管理

**技術スタック**:

- Language: Rust
- Framework: Tokio
- Database: PostgreSQL (Read Model)
- Message Bus: Google Pub/Sub

**デプロイ設定**:

```yaml
Service: progress-projection-service
Platform: Google Cloud Run
CPU: 2
Memory: 2Gi
Min Instances: 1
Max Instances: 5
Concurrency: 10
```

## データストア

### Event Store (PostgreSQL)

**スキーマ設計**:

```sql
-- イベントテーブル
events (
  event_id UUID PRIMARY KEY,
  stream_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  event_data JSONB NOT NULL,
  metadata JSONB,
  occurred_at TIMESTAMPTZ NOT NULL,
  sequence_number BIGSERIAL
)

-- スナップショット
snapshots (
  snapshot_id UUID PRIMARY KEY,
  stream_id TEXT NOT NULL,
  snapshot_data JSONB NOT NULL,
  event_sequence BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
)

-- チェックポイント
checkpoints (
  projection_name TEXT PRIMARY KEY,
  last_processed_sequence BIGINT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
)
```

### Read Model (PostgreSQL)

**主要テーブル**:

- daily_stats: 日別統計
- weekly_stats: 週別統計
- item_stats: 項目別統計
- domain_stats: 領域別統計
- level_stats: レベル別統計
- learning_streaks: 連続学習記録

### Redis Cache

**キャッシュ戦略**:

```yaml
Cache Patterns:
  - GraphQL Response Cache: TTL 5-60分
  - User Summary Cache: TTL 10分
  - Static Stats Cache: TTL 24時間

Key Format:
  - query:{query_name}:{user_id}:{params_hash}
  - summary:{user_id}
  - stats:{type}:{date}:{user_id}
```

## メッセージング

### Google Pub/Sub トピック

```yaml
Topics:
  - learning-events: Learning Context からのイベント
  - algorithm-events: Algorithm Context からのイベント
  - vocabulary-events: Vocabulary Context からのイベント
  - user-events: User Context からのイベント
  - progress-commands: 内部コマンド

Subscriptions:
  - progress-projection-sub: projection-service 用
  - progress-analytics-sub: 分析用（将来拡張）
```

## 監視とロギング

### メトリクス

**重要指標**:

- イベント処理レイテンシ
- 投影更新時間
- GraphQL クエリ応答時間
- キャッシュヒット率
- エラー率

**監視ツール**:

- Cloud Monitoring
- Cloud Logging
- OpenTelemetry

### アラート設定

```yaml
Alerts:
  - Event Processing Delay > 5s
  - GraphQL Response Time P95 > 500ms
  - Cache Hit Rate < 70%
  - Error Rate > 1%
  - Projection Lag > 1000 events
```

## セキュリティ

### ネットワーク

- VPC 内での通信
- Cloud Run サービス間は内部トラフィック
- Redis は VPC 内のみアクセス可能

### 認証・認可

- サービス間: サービスアカウント
- GraphQL API: JWT トークン検証
- 管理操作: Cloud IAM

### データ保護

- 保存時暗号化 (Cloud SQL)
- 転送時暗号化 (TLS 1.3)
- 機密データのマスキング

## CI/CD パイプライン

### ビルド

```yaml
steps:
  - cargo test
  - cargo build --release
  - docker build
  - docker push to Artifact Registry
```

### デプロイ

```yaml
environments:
  - dev: 自動デプロイ (main ブランチ)
  - staging: 手動承認後デプロイ
  - production: Blue-Green デプロイ
```

## スケーリング戦略

### 水平スケーリング

- query-service: リクエスト数に基づく
- projection-service: イベント処理遅延に基づく
- command-service: CPU 使用率に基づく

### 垂直スケーリング

- 大量バッチ処理時は一時的にメモリ増強
- 月末集計時は projection-service を増強

## 災害復旧

### バックアップ

- Event Store: 日次バックアップ、7日間保持
- Read Model: 6時間ごとスナップショット
- 設定: Git リポジトリで管理

### 復旧手順

1. Event Store の復元
2. チェックポイントの確認
3. 投影の再構築
4. キャッシュのウォームアップ

## コスト最適化

### リソース調整

- 低負荷時間帯は最小インスタンス数を削減
- 開発環境は営業時間外にスケールダウン

### ストレージ

- 古いイベントは圧縮保存
- 1年以上前のデータはアーカイブ
