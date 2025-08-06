# Learning Context - サービスアーキテクチャ

## 概要

Learning Context は、学習セッションの実行管理に特化した単一サービスとして設計されています。リアルタイムな UI インタラクションと他コンテキストとの同期通信を重視し、シンプルで高性能なアーキテクチャを採用しています。

## 設計方針

### なぜ CQRS+ES を採用しないのか

Learning Context は以下の理由から、CQRS+ES ではなくシンプルなアーキテクチャを選択しました：

1. **一時的な責務**: セッション実行という短期的なタスクに特化
2. **リアルタイム性**: UI との密結合で即座のレスポンスが必要
3. **履歴管理の委譲**: 長期的な記録は Progress Context が担当
4. **複雑性の回避**: 過度な複雑化を避け、保守性を重視

### アーキテクチャの特徴

- **単一サービス**: `learning_service` として統合
- **ステートフル but 一時的**: Redis でセッション状態を管理
- **イベント発行**: Progress Context への通知に限定
- **同期通信**: Algorithm/Vocabulary Context との連携

## サービス構成

```
learning_service/
├── domain/                 # ドメイン層
│   ├── session/           # LearningSession 集約
│   │   ├── aggregate.rs   # セッション集約ルート
│   │   ├── commands.rs    # コマンド定義
│   │   └── events.rs      # ドメインイベント
│   ├── user_record/       # UserItemRecord（軽量版）
│   │   └── entity.rs      # 最小限の学習記録
│   └── value_objects/     # 値オブジェクト
│       ├── judgment.rs    # 正誤判定
│       └── trigger.rs     # 解答表示トリガー
│
├── application/           # アプリケーション層
│   ├── session_manager.rs # セッション管理ロジック
│   ├── ui_handler.rs      # UI インタラクション処理
│   └── dto/              # Data Transfer Objects
│
├── infrastructure/        # インフラストラクチャ層
│   ├── redis/            # セッション状態の一時保存
│   │   └── session_store.rs
│   ├── grpc/             # 他サービスとの通信
│   │   ├── algorithm_client.rs
│   │   └── vocabulary_client.rs
│   └── pubsub/           # イベント発行
│       └── event_publisher.rs
│
└── api/                  # API層
    └── graphql/         # GraphQL エンドポイント
        ├── schema.rs    # スキーマ定義
        └── resolvers.rs # リゾルバー実装
```

## 責務の明確化

### Learning Service が担当すること

1. **セッション管理**
   - 学習セッションの開始・進行・完了
   - 1-100問の問題数管理
   - セッション状態の一時保存

2. **ハイブリッド UI の実装**
   - 3秒ルールの自動判定
   - ユーザーインタラクションの処理
   - リアルタイムなフィードバック

3. **正誤判定**
   - ユーザーの理解度判定
   - 即座のフィードバック提供
   - 判定結果の記録

4. **他サービスとの連携**
   - Algorithm Context から項目選定を取得（同期）
   - Vocabulary Context から項目詳細を取得（同期）
   - Progress Context へイベント通知（非同期）

### Learning Service が担当しないこと

- **長期的な学習履歴**: Progress Context に委譲
- **複雑な統計分析**: Progress Context に委譲
- **項目選定アルゴリズム**: Algorithm Context に委譲
- **語彙の詳細管理**: Vocabulary Context に委譲

## データ管理戦略

### セッション状態（Redis）

```rust
// セッション状態の構造
struct SessionState {
    session_id: SessionId,
    user_id: UserId,
    items: Vec<SessionItem>,
    current_index: usize,
    status: SessionStatus,
    started_at: DateTime<Utc>,
    ttl: Duration, // 通常2時間
}
```

**特徴**:

- インメモリで高速アクセス
- TTL 設定で自動削除
- セッション完了後は Progress Context に移譲

### UserItemRecord（最小限の永続化）

```rust
// 最小限の学習記録
struct UserItemRecord {
    user_id: UserId,
    item_id: ItemId,
    last_seen: DateTime<Utc>,
    correct_count: u32,
    total_count: u32,
}
```

**特徴**:

- 最新の状態のみ保持
- 詳細な履歴は Progress Context を参照
- パフォーマンス重視の簡潔な構造

## 通信パターン

### 同期通信（gRPC）

```rust
// Algorithm Context との通信
async fn select_items(&self, request: SelectItemsRequest) 
    -> Result<Vec<ItemId>> {
    self.algorithm_client
        .select_items(request)
        .await
}

// Vocabulary Context との通信
async fn get_item_details(&self, item_id: ItemId) 
    -> Result<VocabularyItem> {
    self.vocabulary_client
        .get_item(item_id)
        .await
}
```

### 非同期通信（Pub/Sub）

```rust
// Progress Context へのイベント発行
async fn publish_event(&self, event: LearningEvent) -> Result<()> {
    self.event_publisher
        .publish("learning.events", event)
        .await
}
```

## 主要な処理フロー

### 1. セッション開始

```
User Request → GraphQL → SessionManager
                             ↓
                    Algorithm Context (同期)
                             ↓
                    Redis に状態保存
                             ↓
                    Progress Context へ通知 (非同期)
```

### 2. 項目提示

```
GraphQL Query → SessionManager → Redis (状態取得)
                                    ↓
                          Vocabulary Context (項目詳細)
                                    ↓
                              UI へレスポンス
```

### 3. 正誤判定

```
User Action → GraphQL → UIHandler
                           ↓
                    3秒タイマー or ユーザー入力
                           ↓
                    判定結果を Redis に保存
                           ↓
                    Progress Context へ通知
```

## パフォーマンス最適化

### キャッシング戦略

1. **セッション状態**: Redis で管理、高速アクセス
2. **項目詳細**: 短期キャッシュ（5分）
3. **アクティブセッション**: メモリキャッシュ

### 応答時間目標

- セッション操作: < 50ms
- 項目取得: < 100ms
- 正誤判定: < 30ms

## エラーハンドリング

### フォールバック戦略

```rust
// Algorithm Context 障害時
if algorithm_service.is_unavailable() {
    // ランダム選定にフォールバック
    return select_random_items(count);
}

// Redis 障害時
if redis.is_unavailable() {
    // インメモリで継続（制限付き）
    use_in_memory_session_store();
}
```

### リトライポリシー

- 同期通信: 最大3回、指数バックオフ
- 非同期通信: 自動リトライ（Pub/Sub の保証）

## モニタリング

### メトリクス

1. **ビジネスメトリクス**
   - アクティブセッション数
   - 平均セッション時間
   - 完了率

2. **技術メトリクス**
   - API レスポンスタイム
   - Redis ヒット率
   - gRPC 通信エラー率

### ログ

- 構造化ログ（JSON 形式）
- セッション ID による追跡
- エラーレベルの自動アラート

## 移行戦略

現在のアーキテクチャはシンプルですが、将来的な拡張に対応できる設計となっています：

1. **段階的な複雑化**: 必要に応じて Command/Query を分離可能
2. **マイクロサービス化**: 責務が増えた場合は分割可能
3. **Event Sourcing**: Progress Context との統合を強化可能

## 関連ドキュメント

- [Learning Context Canvas](canvas.md)
- [集約設計](aggregates.md)
- [ドメインイベント](events.md)
- [Progress Context Architecture](../progress/architecture.md)
