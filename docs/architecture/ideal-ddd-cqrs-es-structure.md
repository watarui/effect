# 理想的な DDD CQRS+ES マイクロサービス構造

## 概要

Effect プロジェクトにおける理想的な DDD (Domain-Driven Design)、CQRS (Command Query Responsibility Segregation)、ES (Event Sourcing) の実装構造について説明します。

## ディレクトリ構造

```
effect/
├── shared/
│   ├── kernel/                    # 共有カーネル（最小限の共通概念）
│   │   ├── src/
│   │   │   ├── events.rs         # イベント基本型・トレイト
│   │   │   ├── ids.rs            # 共有識別子（UserId, ItemId等）
│   │   │   ├── value_objects.rs  # 共通値オブジェクト
│   │   │   └── proto.rs          # Proto 共通型
│   │   └── protos/
│   │       └── common/           # 共通 Proto 定義
│   │
│   └── contexts/                  # 各 Bounded Context
│       ├── vocabulary/
│       │   ├── src/
│       │   │   ├── events.rs     # Vocabulary ドメインイベント
│       │   │   ├── commands.rs   # コマンド定義
│       │   │   ├── queries.rs    # クエリ定義
│       │   │   └── domain/       # ドメインモデル
│       │   ├── protos/
│       │   │   └── events/       # Vocabulary 固有 Proto
│       │   └── build.rs          # Proto ビルドスクリプト
│       │
│       ├── user/                 # 同様の構造
│       ├── learning/
│       ├── algorithm/
│       ├── progress/
│       └── ai/
│
└── services/                      # 各マイクロサービス
    ├── vocabulary_service/
    ├── user_service/
    └── ...
```

## 主要な設計原則

### 1. Bounded Context の自律性

各 Bounded Context は自身のイベント、コマンド、クエリを所有します：

```rust
// shared/contexts/vocabulary/src/events.rs

// Proto 生成コードを含める
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/effect.events.vocabulary.rs"));
}

// Proto 型を再エクスポート（Domain Events として使用）
pub use proto::*;

// Integration Events（他コンテキスト公開用）
pub enum VocabularyIntegrationEvent {
    ItemPublished { /* 公開情報のみ */ },
    ItemUpdated { /* 最小限の情報 */ },
}
```

### 2. イベントの2層構造

#### a) Proto Events（永続化・メッセージング用、単一の真実の源）

```protobuf
message ItemCreated {
  effect.common.EventMetadata metadata = 1;
  string item_id = 2;
  string spelling = 3;
  // すべての詳細情報
}
```

Proto ファイルが単一の真実の源となり、ここから Rust コードが生成されます。

#### b) Integration Events（コンテキスト間連携用）

```rust
pub enum VocabularyIntegrationEvent {
    ItemPublished {
        event_id: String,
        occurred_at: chrono::DateTime<chrono::Utc>,
        item_id: String,
        spelling: String,
        // 最小限の公開情報
    },
}
```

### 3. 共有カーネルの最小化

`shared_kernel` には本当に共有が必要な要素のみを含めます：

```rust
// shared/kernel/src/events.rs
pub trait DomainEvent: Send + Sync {
    fn event_type(&self) -> &str;
    fn metadata(&self) -> &EventMetadata;
}

pub trait IntegrationEvent: Send + Sync {
    fn event_type(&self) -> &str;
    fn source_context(&self) -> &str;
}
```

### 4. CQRS の実装

#### Command Side（書き込み）

```rust
// コマンドハンドラー
impl CommandHandler for CreateVocabularyItemHandler {
    async fn handle(&self, cmd: CreateVocabularyItem) -> Result<ItemId> {
        // 1. ドメインロジック実行
        let item = VocabularyItem::new(cmd)?;
        
        // 2. Proto イベント生成
        let event = ItemCreated {
            metadata: Some(EventMetadata::new()),
            item_id: item.id.to_string(),
            spelling: item.spelling.clone(),
            // ...
        };
        
        // 3. イベントストアに保存
        self.event_store.append(&event).await?;
        
        // 4. 統合イベント発行
        let integration_event = VocabularyIntegrationEvent::ItemPublished {
            event_id: Uuid::new_v4().to_string(),
            occurred_at: Utc::now(),
            item_id: item.id.to_string(),
            spelling: item.spelling,
            // 最小限の公開情報
        };
        self.event_bus.publish(integration_event).await?;
        
        Ok(item.id)
    }
}
```

#### Query Side（読み取り）

```rust
// クエリハンドラー（プロジェクションから読み取り）
impl QueryHandler for GetVocabularyItemHandler {
    async fn handle(&self, query: GetVocabularyItem) -> Result<VocabularyItemView> {
        // プロジェクションから直接読み取り
        self.read_model.get_item(query.item_id).await
    }
}
```

### 5. Event Sourcing の実装

```rust
// アグリゲートの再構築
pub fn rebuild_aggregate(events: Vec<VocabularyEvent>) -> Result<VocabularyItem> {
    let mut item = VocabularyItem::default();
    
    for event in events {
        item.apply(event)?;
    }
    
    Ok(item)
}

// Proto イベントの適用
impl VocabularyItem {
    fn apply(&mut self, event: VocabularyEvent) -> Result<()> {
        match event.event {
            Some(vocabulary_event::Event::ItemCreated(e)) => {
                self.id = ItemId::from_str(&e.item_id)?;
                self.spelling = e.spelling;
                // ...
            }
            Some(vocabulary_event::Event::FieldUpdated(e)) => {
                // フィールドパスに基づいて更新を適用
                self.apply_field_update(&e.field_path, &e.new_value_json)?;
                // ...
            }
            _ => {}
        }
        Ok(())
    }
}
```

## 移行のメリット

### 1. 疎結合

- 各コンテキストが独立して進化可能
- 依存関係が明確で管理しやすい

### 2. スケーラビリティ

- コンテキスト単位での独立したスケーリング
- イベント駆動による非同期処理

### 3. 保守性

- 変更の影響範囲が限定的
- テストが書きやすい

### 4. 可監査性

- すべての変更がイベントとして記録
- 完全な監査証跡

## ベストプラクティス

1. **イベントの不変性**: 一度発行されたイベントは変更しない
2. **後方互換性**: スキーマバージョニングで互換性を維持
3. **最終的整合性**: 即座の整合性を求めない
4. **補償トランザクション**: 失敗時の補償イベントを設計
5. **イベントの冪等性**: 重複配信に対する耐性

## まとめ

この構造により、真の DDD/CQRS/ES マイクロサービスアーキテクチャが実現されます。各 Bounded Context が自律的に動作し、イベントを通じて協調することで、スケーラブルで保守性の高いシステムが構築できます。
