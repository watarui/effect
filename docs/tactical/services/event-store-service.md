# Event Store Service

## 概要

Event Store Service は、Effect プロジェクト全体のイベントソーシングを管理する中央集権的なマイクロサービスです。すべての Bounded Context のイベントを一元的に管理し、イベントの永続化、取得、配信を担当します。

## 責務

1. **イベントの永続化**: ドメインイベントの追加専用ストレージ
2. **イベントストリームの管理**: 集約ごとのイベント順序保証
3. **スナップショット管理**: パフォーマンス最適化のためのスナップショット
4. **イベント配信**: リアルタイムイベントストリーミング
5. **監査ログ**: すべてのイベントの完全な履歴

## アーキテクチャ

### 位置づけ

```
┌─────────────────────────────────────────────┐
│              各マイクロサービス                │
│  (Vocabulary, Learning, Progress, etc.)      │
└────────────────┬────────────────────────────┘
                 │ gRPC
                 ▼
┌─────────────────────────────────────────────┐
│           Event Store Service                │
│  ┌─────────────────────────────────────┐    │
│  │      gRPC API Layer                 │    │
│  └─────────────────────────────────────┘    │
│  ┌─────────────────────────────────────┐    │
│  │      Repository Layer               │    │
│  └─────────────────────────────────────┘    │
│  ┌─────────────────────────────────────┐    │
│  │      PostgreSQL                     │    │
│  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
```

### データモデル

#### Event Streams Table

```sql
CREATE TABLE event_streams (
    stream_id UUID NOT NULL,
    stream_type VARCHAR(255) NOT NULL,
    version BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (stream_id, stream_type)
);
```

#### Events Table

```sql
CREATE TABLE events (
    event_id UUID PRIMARY KEY,
    stream_id UUID NOT NULL,
    stream_type VARCHAR(255) NOT NULL,
    version BIGINT NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL,
    position BIGSERIAL,  -- グローバル順序
    UNIQUE (stream_id, stream_type, version)
);
```

#### Snapshots Table

```sql
CREATE TABLE snapshots (
    snapshot_id UUID PRIMARY KEY,
    stream_id UUID NOT NULL,
    stream_type VARCHAR(255) NOT NULL,
    version BIGINT NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    UNIQUE (stream_id, stream_type, version)
);
```

## gRPC API

### Service Definition

```protobuf
service EventStoreService {
  // イベントを追加
  rpc AppendEvents(AppendEventsRequest) returns (AppendEventsResponse);
  
  // イベントを取得
  rpc GetEvents(GetEventsRequest) returns (GetEventsResponse);
  
  // スナップショットを取得
  rpc GetSnapshot(GetSnapshotRequest) returns (GetSnapshotResponse);
  
  // スナップショットを保存
  rpc SaveSnapshot(SaveSnapshotRequest) returns (SaveSnapshotResponse);
  
  // イベントストリームを購読
  rpc SubscribeToStream(SubscribeRequest) returns (stream EventNotification);
  
  // 全イベントを購読
  rpc SubscribeToAll(SubscribeAllRequest) returns (stream EventNotification);
}
```

### 主要な操作

#### 1. AppendEvents

- **目的**: 新しいイベントをストリームに追加
- **楽観的ロック**: `expected_version` による競合制御
- **アトミック性**: すべてのイベントが成功するか、すべて失敗

#### 2. GetEvents

- **目的**: 特定ストリームのイベントを取得
- **範囲指定**: バージョンによる範囲指定可能
- **ページング**: 大量イベントの分割取得

#### 3. SubscribeToStream

- **目的**: 特定ストリームのリアルタイム購読
- **プッシュ型**: gRPC ストリーミングによるプッシュ配信
- **再接続**: 接続断からの自動復旧

## 実装詳細

### 楽観的ロック

```rust
pub async fn append_events(
    &self,
    stream_id: Uuid,
    events: Vec<Event>,
    expected_version: Option<i64>,
) -> Result<i64> {
    // 現在のバージョンを確認
    let current_version = self.get_stream_version(stream_id).await?;
    
    // バージョンチェック
    if let Some(expected) = expected_version {
        if current_version != expected {
            return Err(VersionConflict { 
                expected, 
                actual: current_version 
            });
        }
    }
    
    // イベントを保存
    // ...
}
```

### スナップショット戦略

- **作成タイミング**: 100イベントごと（設定可能）
- **保持期間**: 30日間（設定可能）
- **リプレイ最適化**: スナップショット + 差分イベント

### グローバル順序保証

`position` カラムによる全イベントの順序保証：

```sql
-- すべてのイベントを順序付きで取得
SELECT * FROM events 
ORDER BY position ASC
WHERE position > $1
LIMIT 100;
```

## 運用

### 設定

環境変数による設定：

```bash
# gRPC ポート
PORT=50051

# データベース接続
DATABASE_URL=postgres://effect:password@localhost:5432/event_store_db

# スナップショット設定
SNAPSHOT_THRESHOLD=100
SNAPSHOT_RETENTION_DAYS=30
```

### モニタリング

#### ヘルスチェック

- `/health/live`: プロセスの生存確認
- `/health/ready`: サービス提供可能状態

#### メトリクス

- イベント追加レート
- ストリーム数
- スナップショット作成数
- gRPC レイテンシ

### バックアップ

1. **定期バックアップ**: 日次でフルバックアップ
2. **WAL アーカイブ**: 継続的なアーカイブ
3. **ポイントインタイムリカバリ**: 任意時点への復元

## 他サービスとの連携

### Command Service からの利用

```rust
// Event Store Client の使用例
let client = EventStoreClient::connect("http://event-store:50051").await?;

// イベントの追加
let request = AppendEventsRequest {
    stream_id: aggregate_id.to_string(),
    stream_type: "VocabularyItem".to_string(),
    events: vec![event],
    expected_version: Some(current_version),
};

client.append_events(request).await?;
```

### Projection Service での購読

```rust
// ストリーム購読の例
let request = SubscribeRequest {
    stream_type: "VocabularyItem".to_string(),
    from_version: 0,
    include_existing: true,
};

let mut stream = client.subscribe_to_stream(request).await?;

while let Some(notification) = stream.message().await? {
    // イベント処理
    handle_event(notification.event);
}
```

## セキュリティ

### 認証・認可

- サービス間通信: mTLS
- API キー: 各サービスに個別のキー発行

### 監査

- すべての書き込み操作をログ記録
- イベントメタデータに操作者情報を記録

## パフォーマンス考慮事項

### スケーリング戦略

1. **垂直スケーリング**: 書き込み性能の向上
2. **読み取りレプリカ**: 読み取り負荷の分散
3. **パーティショニング**: 将来的なシャーディング対応

### 最適化

- インデックス戦略
- 接続プーリング
- バッチ処理
- キャッシュ（スナップショット）

## 今後の拡張

1. **イベント変換**: スキーマ進化への対応
2. **プロジェクション管理**: 投影状態の管理
3. **イベントリプレイ**: 特定時点からの再実行
4. **分散トランザクション**: Saga パターンのサポート

## 関連ドキュメント

- [ADR-003: Event Store と Event Bus の技術選定](../../decisions/003-event-store-and-event-bus-selection.md)
- [Event Sourcing ガイドライン](../event-sourcing-guidelines.md)
- [共有インフラストラクチャ](../shared/infrastructure.md)

## 更新履歴

- 2025-08-09: 初版作成（Event Store のマイクロサービス化）
