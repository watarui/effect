# Vocabulary Context - インフラストラクチャ

## 概要

Vocabulary Context の技術選択、デプロイ構成、非機能要件の定義です。

## 技術スタック

### データストア

**PostgreSQL**

- Event Store: イベントの永続化
- Read Model: 非正規化されたビュー
- バージョン: 15以上（JSONB、gen_random_uuid() のため）

**Redis**

- 用途: Query Service のキャッシュ
- TTL: 5分（頻繁に更新されるデータ）
- 構成: Google Cloud Memorystore

**Meilisearch**

- 用途: 全文検索
- バージョン: 最新安定版
- デプロイ: Cloud Run（コンテナ）

### メッセージング

**Google Cloud Pub/Sub**

- Event Bus として使用
- トピック構成:
  - `vocabulary-events`: すべてのドメインイベント
  - `vocabulary-commands`: コマンドのキューイング（オプション）

### コンテナ・オーケストレーション

**Google Cloud Run**

- 各マイクロサービスをコンテナとしてデプロイ
- 自動スケーリング
- サーバーレス課金

## デプロイ構成

```yaml
services:
  vocabulary-command:
    image: gcr.io/effect-project/vocabulary-command-service
    env:
      - DATABASE_URL
      - PUBSUB_TOPIC
    scaling:
      min_instances: 1
      max_instances: 10

  vocabulary-query:
    image: gcr.io/effect-project/vocabulary-query-service
    env:
      - DATABASE_URL
      - REDIS_URL
    scaling:
      min_instances: 2
      max_instances: 20

  vocabulary-projection:
    image: gcr.io/effect-project/vocabulary-projection-service
    env:
      - DATABASE_URL
      - PUBSUB_SUBSCRIPTION
    scaling:
      min_instances: 1
      max_instances: 5

  vocabulary-search:
    image: gcr.io/effect-project/vocabulary-search-service
    env:
      - MEILISEARCH_URL
      - PUBSUB_SUBSCRIPTION
    scaling:
      min_instances: 1
      max_instances: 5
```

## 非機能要件

### 認証・認可

**Firebase Authentication**

- Google OAuth によるログイン
- JWT トークンの検証

**権限モデル**:

- 未認証ユーザー: 読み取りのみ
- 認証済みユーザー: 作成・更新可能
- 管理者: 公開・削除可能

### レート制限

**実装方法**: Redis を使用したトークンバケット

**制限値**:

- 未認証: 60 req/min
- 認証済み: 600 req/min
- 管理者: 無制限

**実装例**:

```rust
// シンプルなレート制限
async fn check_rate_limit(user_id: &str, redis: &Redis) -> Result<bool> {
    let key = format!("rate_limit:{}", user_id);
    let count = redis.incr(&key).await?;
    if count == 1 {
        redis.expire(&key, 60).await?; // 1分間
    }
    Ok(count <= 600)
}
```

### ロギング

**Google Cloud Logging**

- 構造化ログ（JSON形式）
- トレースID による分散トレーシング
- ログレベル: INFO（本番）、DEBUG（開発）

**ログ形式**:

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "severity": "INFO",
  "trace": "projects/effect/traces/abc123",
  "service": "vocabulary-command",
  "message": "Item created",
  "labels": {
    "user_id": "user123",
    "item_id": "item456"
  }
}
```

### モニタリング

**Google Cloud Monitoring**

- 基本メトリクス:
  - リクエスト数
  - レスポンスタイム（p50, p95, p99）
  - エラー率
  - CPU/メモリ使用率

**カスタムメトリクス**:

- イベント処理遅延
- キャッシュヒット率
- 検索クエリ応答時間

**アラート設定**:

- エラー率 > 1%
- レスポンスタイム p99 > 1秒
- Event Bus の遅延 > 5秒

### セキュリティ

**ネットワーク**:

- Cloud Run サービス間は内部通信
- HTTPS のみ許可
- Cloud Armor で DDoS 対策

**データ**:

- データベース接続は Cloud SQL Proxy 経由
- シークレットは Secret Manager で管理
- 個人情報の暗号化（必要に応じて）

### バックアップ・DR

**データベース**:

- 自動バックアップ（毎日）
- Point-in-time recovery（7日間）

**Event Store**:

- イベントは不変なので、バックアップから完全復元可能
- スナップショット + イベント再生で高速復旧

### パフォーマンス目標

- API レスポンスタイム: p99 < 200ms
- 検索レスポンスタイム: p99 < 500ms
- イベント処理遅延: p99 < 1秒
- 可用性: 99.9%

## アセット管理

### 画像・音声ファイルの保存

**Google Cloud Storage**

- 用途: 語彙項目に関連する画像（イラスト）や音声ファイルの保存
- バケット構成:
  - `effect-vocabulary-assets`: 本番用バケット
  - `effect-vocabulary-assets-dev`: 開発用バケット

**ディレクトリ構造**:

```
/images/
  /{item_id}/
    /illustration.webp   # メインイラスト
    /thumbnail.webp      # サムネイル
/audio/
  /{item_id}/
    /pronunciation.mp3   # 発音音声
```

**アクセス制御**:

- 読み取り: 公開（CDN 経由）
- 書き込み: サービスアカウント経由のみ
- URL 形式: 署名付き URL（有効期限付き）

## AI 生成の設計

### AI エンリッチメント

**処理方式**:

- 非同期処理（Pub/Sub 経由）
- 単一コマンド: すべての AI 生成可能な情報を一括生成
  - 発音記号
  - 例文（複数）
  - コロケーション
  - 同義語・反意語
  - CEFR レベル推定

**優先度制御**:

- FIFO（First In, First Out）で処理
- 将来的に優先度キューを検討可能だが、現時点では実装しない

### 実装しない機能

以下の機能は複雑性とコストを考慮し、実装しない：

1. **例文の難易度評価**
   - 理由: 難易度の客観的基準の設定が困難
   - 代替: CEFR レベルで十分

2. **コロケーションの自動抽出**
   - 理由: 高度な自然言語処理が必要で複雑
   - 代替: AI による生成時に含める

3. **外部辞書 API の利用**
   - 理由: ライセンス、コスト、API 制限の問題
   - 代替: AI による生成で十分な品質を確保
