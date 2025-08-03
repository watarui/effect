# Event Migration Guide - DDD CQRS+ES への移行ガイド

## 概要

このガイドは、`shared/domain_events` から各 Bounded Context へのイベント移行手順を説明します。

## 移行の目的

1. **コンテキストの自律性向上**: 各コンテキストが自身のイベントを所有
2. **依存関係の明確化**: コンテキスト間の依存を最小化
3. **Proto を単一の真実の源に**: Proto ファイルから Domain Events を生成
4. **Integration Events の分離**: 他コンテキストへの公開用イベントを明確に区別

## 移行手順

### 1. Proto ファイルの移動

```bash
# コンテキスト用の proto ディレクトリを作成
mkdir -p shared/contexts/{context_name}/protos/events

# Proto ファイルをコピー
cp protos/events/{context}_events.proto shared/contexts/{context_name}/protos/events/

# 必要に応じて他の proto ファイルもコピー
cp protos/services/{context}_service.proto shared/contexts/{context_name}/protos/services/
```

### 2. build.rs の作成

各コンテキストに `build.rs` を作成：

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "protos".to_string();
    let shared_kernel_proto = "../../../shared/kernel/protos".to_string();

    let mut config = tonic_prost_build::configure();

    // 設定...
    config.compile_protos(
        &[
            &format!("{proto_root}/events/{context}_events.proto"),
        ],
        &[&proto_root, &shared_kernel_proto],
    )?;

    Ok(())
}
```

### 3. Cargo.toml の更新

workspace dependencies を使用：

```toml
[dependencies]
shared_kernel = { path = "../../kernel" }
chrono = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
prost = { workspace = true }
prost-types = { workspace = true }
async-trait = { workspace = true }

[build-dependencies]
tonic-prost-build = { workspace = true }
```

### 4. events.rs の作成

Proto を単一の真実の源として使用し、Integration Events のみ Rust で定義：

```rust
use shared_kernel::{DomainEvent, EventMetadata, IntegrationEvent};

// Proto 生成コードを含める
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/effect.events.{context}.rs"));
}

// Proto 型を再エクスポート（Domain Events として使用）
pub use proto::*;

// DomainEvent トレイトの実装（Proto 生成型用）
impl DomainEvent for {Context}Event {
    fn event_type(&self) -> &str {
        match &self.event {
            Some({context}_event::Event::EventType1(_)) => "{Context}EventType1",
            Some({context}_event::Event::EventType2(_)) => "{Context}EventType2",
            // ...
            None => "{Context}EventUnknown",
        }
    }

    fn metadata(&self) -> &EventMetadata {
        // TODO: Proto の EventMetadata を shared_kernel の EventMetadata に変換
        todo!("Convert proto EventMetadata to shared_kernel EventMetadata")
    }
}

/// Integration Event（他コンテキスト公開用）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum {Context}IntegrationEvent {
    // 公開するイベントのみ、最小限の情報
}

impl IntegrationEvent for {Context}IntegrationEvent {
    fn event_type(&self) -> &str { /* ... */ }
    fn source_context(&self) -> &str { "{context}" }
    fn event_id(&self) -> &str { /* ... */ }
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> { /* ... */ }
}
```

### 5. サービスの依存関係更新

各サービスの `Cargo.toml` を更新：

```toml
[dependencies]
shared_{context}_context = { path = "../../shared/contexts/{context}" }
# domain_events への依存を削除（移行完了後）
```

## イベントの使い分け

### Proto Events (Domain Events)

- **用途**: コンテキスト内部でのイベント処理、イベントストアへの永続化
- **特徴**: Proto が単一の真実の源、詳細な情報を含む、バージョニング対応
- **例**: `ItemCreated`, `UserSignedUp`, `FieldUpdated`

### Integration Events

- **用途**: コンテキスト間の連携
- **特徴**: Rust enum で定義、最小限の情報、安定したインターフェース
- **例**: `VocabularyIntegrationEvent::ItemPublished`, `UserIntegrationEvent::UserRegistered`

## 完了チェックリスト

- [ ] Proto ファイルを移動
- [ ] build.rs を作成
- [ ] Cargo.toml を更新
- [ ] events.rs を作成
- [ ] Domain Events を定義
- [ ] Integration Events を定義
- [ ] サービスの依存関係を更新
- [ ] テストを実行して動作確認

## 移行済みコンテキスト

- ✅ shared/kernel - 共通型定義
- ✅ vocabulary context - 語彙管理
- ✅ user context - ユーザー管理
- ⏳ learning context
- ⏳ algorithm context
- ⏳ ai context
- ⏳ progress context

## 注意事項

1. **Proto が単一の真実の源**: Rust ネイティブの Domain Events は作成せず、Proto から生成されたコードを使用
2. **workspace.dependencies を使用**: 依存関係のバージョン管理を統一
3. **後方互換性**: 移行中は `domain_events` への依存を一時的に保持
4. **段階的移行**: 各コンテキストを独立して移行可能
5. **テスト**: 各段階でテストと clippy を実行して動作確認
