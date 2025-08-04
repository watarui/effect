# vocabulary_projection_service 実装状況

最終更新: 2025-08-03

## 概要

`vocabulary_projection_service` は、Vocabulary Context における CQRS/ES パターンの Read Model を構築するサービスです。Event Store からのイベントを購読し、クエリ用に最適化されたビューを生成・更新します。

## 現在の実装状況

### 完了した実装

#### 1. ヘキサゴナルアーキテクチャ構造

```
services/vocabulary_projection_service/
├── src/
│   ├── domain/           # ドメイン層
│   │   ├── read_models.rs    # Read Model 定義
│   │   └── projections.rs    # プロジェクションビルダー
│   ├── application/      # アプリケーション層
│   │   └── event_handler.rs  # イベントハンドラー
│   ├── infrastructure/   # インフラストラクチャ層
│   │   └── outbound/
│   │       ├── postgres.rs   # PostgreSQL リポジトリ
│   │       └── pubsub.rs     # Pub/Sub サブスクライバー
│   └── ports/           # ポート定義
│       ├── inbound.rs       # インバウンドポート
│       └── outbound.rs      # アウトバウンドポート
```

#### 2. Read Model 設計（最小構成）

**vocabulary_items_view**

- 非正規化された語彙項目ビュー
- クエリパフォーマンスに最適化
- JSONB で柔軟なデータ構造（definitions, synonyms, antonyms, collocations）

**projection_state**

- プロジェクションの進行状況管理
- エラーハンドリング
- リプレイ機能のサポート

#### 3. データベース設定

- マイグレーションファイル作成済み (`20250803_000001_create_read_models.sql`)
- SQLx オフラインモード対応（`.sqlx/query-*.json` 生成済み）
- PostgreSQL REAL 型（f32）を quality_score に使用

### 未完了・TODO 項目

#### 1. Clippy エラーの修正

現在、以下の clippy エラーによりコミットできない状態：

```rust
// 未使用のフィールド
fields `read_model_repo`, `projection_repo`, and `projection_name` are never read

// 未使用のメソッド（8個）
- update_projection_success
- update_projection_error
- handle_entry_created
- handle_item_added_to_entry
- handle_definition_added
- handle_example_added
- handle_item_status_changed
- handle_item_deleted

// 引数が多すぎる（8個、制限は7個）
- handle_item_added_to_entry
- handle_definition_added

// new メソッドが Self を返さない
ProjectionStateBuilder::new() -> ProjectionState
```

#### 2. イベントハンドラーの完全実装

```rust
// 現在の handle_event メソッドは仮実装
async fn handle_event(&self, event_data: Vec<u8>) -> DomainResult<()> {
    // TODO: Proto メッセージをデシリアライズして処理
    // 1. VocabularyEvent をデシリアライズ
    // 2. イベントタイプに応じて処理を振り分け
    // 3. プロジェクション状態を更新
    
    // 仮実装
    tracing::info!("Handling event: {} bytes", event_data.len());
    Ok(())
}
```

#### 3. Pub/Sub サブスクライバーの完全実装

```rust
// google-cloud-pubsub のライフタイム問題により一時的に無効化
// TODO: 実際のサブスクリプション処理を実装
// 現時点では google-cloud-pubsub の receive メソッドのライフタイムの問題により、
// 完全な実装ができません。
```

## 解決策

### Clippy エラーの修正案

1. **未使用エラーの解決**
   - `handle_event` 内で実際にイベント処理を実装
   - Proto デシリアライズと各ハンドラーメソッドの呼び出し

2. **引数が多すぎる問題**

   ```rust
   // パラメータ構造体を導入
   struct ItemAddedParams {
       item_id: Uuid,
       entry_id: Uuid,
       spelling: String,
       disambiguation: String,
       created_by_type: String,
       created_by_id: Option<Uuid>,
       occurred_at: DateTime<Utc>,
   }
   ```

3. **new メソッドの問題**
   - `build` にリネーム
   - または `#[allow(clippy::new_ret_no_self)]` を追加

## イベントマッピング

現在の proto ファイル (`vocabulary_events.proto`) と Read Model の対応：

| Proto イベント | ハンドラーメソッド | 処理内容 |
|--------------|----------------|---------|
| EntryCreated | handle_entry_created | ログのみ（Read Model 作成なし） |
| ItemCreated | handle_item_added_to_entry | VocabularyItemView を作成 |
| FieldUpdated | （未実装） | フィールドパスに応じて更新 |
| ItemPublished | handle_item_status_changed | ステータスを "published" に更新 |
| ItemDeleted | handle_item_deleted | VocabularyItemView を削除 |

## 次のステップ

1. **即座に必要な修正**
   - Clippy エラーを修正してコミット可能にする
   - 最小限の動作するイベントハンドラーを実装

2. **次のコミットで対応**
   - Proto デシリアライズの完全実装
   - イベントメタデータの処理
   - FieldUpdated イベントの詳細な処理

3. **将来的な改善**
   - Pub/Sub ライフタイム問題の解決
   - エラーリトライ機構
   - プロジェクションのリプレイ機能
   - パフォーマンス最適化

## 参考情報

- 関連ドキュメント: `/docs/ddd/bounded-contexts/vocabulary-context.md`
- Proto 定義: `/protos/events/vocabulary_events.proto`
- 共有コンテキスト: `/shared/contexts/vocabulary/`
