# ADR 003: Event Store と Event Bus の技術選定

## ステータス

**改訂** (2025-08-09) - Event Store をマイクロサービス化
**採択** (2025-08-03)

## コンテキスト

CQRS と Event Sourcing の実装において、以下の2つの重要なコンポーネントが必要です：

1. **Event Store**: イベントの永続的な保存
2. **Event Bus**: イベントの配信とサービス間通信

それぞれ異なる要件を持つため、適切な技術選定が必要です。

### Event Store の要件

1. **永続性**: イベントの無期限保存
2. **順序保証**: 集約ごとのイベント順序の厳密な保持
3. **イベントリプレイ**: 任意時点からの再生機能
4. **スナップショット**: パフォーマンス最適化
5. **トランザクション**: イベントの原子性保証

### Event Bus の要件

1. **信頼性**: At-least-once デリバリー保証
2. **スケーラビリティ**: 複数コンシューマーへの配信
3. **非同期処理**: サービス間の疎結合
4. **パーティショニング**: 並列処理のサポート
5. **監視**: メッセージフローの可視化

## 決定

**マイクロサービスアプローチ**を採用します（2025-08-09 改訂）：

- **Event Store Service**: 独立したマイクロサービス（PostgreSQL ベース）
- **Event Bus**: Google Pub/Sub

### アーキテクチャ（改訂版）

```
┌─────────────┐
│Command Service│
└──────┬──────┘
       │ 1. AppendEvents (gRPC)
       ▼
┌─────────────────┐
│Event Store Service│ ← 中央集権的なイベント管理
│  (PostgreSQL)    │
└────────┬────────┘
         │ 2. Publish Event
         ▼
┌─────────────┐
│Google Pub/Sub│ ← Event Bus (配信)
└──────┬──────┘
       │ 3. Subscribe
       ▼
┌─────────────┐
│ Projection  │
│  Service    │
└─────────────┘
```

### 改訂理由（2025-08-09）

当初はライブラリアプローチを採用していましたが、以下の理由でマイクロサービス化を決定：

1. **スケーラビリティ**: Event Store を独立してスケール可能
2. **順序保証**: 中央管理によるグローバルな順序保証
3. **運用性**: Event Store の監視・管理を一元化
4. **実践的**: 実務で採用されているパターン

## 理由

### PostgreSQL を Event Store として選定した理由

1. **実績と信頼性**: トランザクショナルな永続性
2. **JSONBサポート**: イベントデータの柔軟な保存
3. **順序保証**: シーケンス番号による厳密な順序管理
4. **既存の知識**: チームの PostgreSQL 経験を活用
5. **コスト効率**: 追加インフラ不要

### Google Pub/Sub を Event Bus として選定した理由

1. **Google Cloud との統合**: Cloud Run との相性
2. **マネージドサービス**: 運用負荷の削減
3. **スケーラビリティ**: 自動スケーリング
4. **コスト**: 従量課金（最初の10GB/月は無料）
5. **信頼性**: Google のインフラストラクチャ

## 実装詳細

### Event Store (PostgreSQL) スキーマ

```sql
-- イベントストアテーブル
CREATE TABLE events (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_version INTEGER NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(aggregate_id, event_version)
);

-- インデックス
CREATE INDEX idx_events_aggregate ON events(aggregate_id);
CREATE INDEX idx_events_occurred_at ON events(occurred_at);
CREATE INDEX idx_events_type ON events(event_type);

-- スナップショットテーブル（オプション）
CREATE TABLE snapshots (
    aggregate_id UUID PRIMARY KEY,
    aggregate_type VARCHAR(100) NOT NULL,
    snapshot_data JSONB NOT NULL,
    event_version INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Event Bus (Pub/Sub) 設定

```yaml
# Topic 構成
topics:
  - name: vocabulary-events
    messageRetentionDuration: 7d
    
  - name: progress-events
    messageRetentionDuration: 30d
    
  - name: integration-events
    messageRetentionDuration: 7d

# Subscription 構成
subscriptions:
  - name: vocabulary-projection
    topic: vocabulary-events
    ackDeadlineSeconds: 60
    retryPolicy:
      minimumBackoff: 10s
      maximumBackoff: 600s
```

### 実装例

```rust
// Event Store 実装
pub struct PostgresEventStore {
    pool: PgPool,
}

impl EventStore for PostgresEventStore {
    async fn append_events(&self, events: &[DomainEvent]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        
        for event in events {
            sqlx::query!(
                r#"
                INSERT INTO events 
                (aggregate_id, aggregate_type, event_type, event_version, 
                 event_data, metadata, occurred_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                event.aggregate_id(),
                event.aggregate_type(),
                event.event_type(),
                event.version(),
                event.data(),
                event.metadata(),
                event.occurred_at()
            )
            .execute(&mut tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }
}

// Event Bus 実装
pub struct PubSubEventBus {
    publisher: Publisher,
}

impl EventBus for PubSubEventBus {
    async fn publish(&self, event: DomainEvent) -> Result<()> {
        let topic = self.topic_for_event(&event);
        let message = PubsubMessage {
            data: serde_json::to_vec(&event)?,
            attributes: HashMap::from([
                ("event_type".to_string(), event.event_type()),
                ("aggregate_id".to_string(), event.aggregate_id()),
            ]),
            ..Default::default()
        };
        
        self.publisher.publish(topic, message).await?;
        Ok(())
    }
}
```

## 結果

### 正の結果

1. **完全な Event Sourcing**: Event Store による永続化
2. **スケーラブルな配信**: Pub/Sub による非同期通信
3. **運用の簡素化**: マネージドサービスの活用
4. **コスト最適化**: 必要な機能に対して適切なツール
5. **Google Cloud 統合**: 既存インフラとの親和性

### 負の結果

1. **複雑性**: 2つのシステムの管理
2. **一貫性**: Event Store と Event Bus の同期
3. **学習曲線**: Pub/Sub の理解が必要

## 代替案

### 代替案 1: Kafka のみ

Event Store と Event Bus の両方を Kafka で実装。

**却下理由**:

- Event Sourcing には不適切（保持期間の制限）
- 運用コストが高い
- Google Cloud での管理が複雑

### 代替案 2: PostgreSQL のみ

LISTEN/NOTIFY を使用して Event Bus 機能も実装。

**却下理由**:

- スケーラビリティの制限
- 非同期処理に不向き
- リトライ機能の実装が必要

### 代替案 3: EventStore (専用製品)

EventStoreDB などの専用製品を使用。

**却下理由**:

- 追加の学習コスト
- Google Cloud での運用が複雑
- 小規模プロジェクトにはオーバースペック

## 移行戦略

1. **Phase 1**: PostgreSQL Event Store の実装
2. **Phase 2**: Pub/Sub の基本実装
3. **Phase 3**: デッドレターキューの設定
4. **Phase 4**: 監視とアラートの設定

## 開発環境

ローカル開発では以下を使用：

- PostgreSQL: Docker コンテナ
- Pub/Sub: エミュレータまたは開発用プロジェクト

```yaml
# docker-compose.yml
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: event_store
      POSTGRES_USER: effect
      POSTGRES_PASSWORD: effect_password
    
  pubsub-emulator:
    image: gcr.io/google.com/cloudsdktool/cloud-sdk:emulators
    command: gcloud beta emulators pubsub start --host-port=0.0.0.0:8085
    ports:
      - "8085:8085"
```

## 参考資料

- [Event Sourcing Pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/event-sourcing)
- [Google Pub/Sub Documentation](https://cloud.google.com/pubsub/docs)
- [PostgreSQL as Event Store](https://blog.eventstore.com/event-sourcing-and-postgresql/)

## 更新履歴

- 2025-08-09: Event Store をマイクロサービス化
- 2025-08-03: 初版作成（Kafka から PostgreSQL + Pub/Sub に変更）
