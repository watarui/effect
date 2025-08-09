# Learning Algorithm Context - インフラストラクチャ

## 概要

Learning Algorithm Context の技術選択、デプロイ構成、非機能要件の定義です。単一サービスとして SM-2 アルゴリズムの高速実行を実現します。

## 技術スタック

### データストア

**PostgreSQL**

- 用途: 学習記録の永続化、統計データ
- バージョン: 15以上
- 接続プール: 最小 10、最大 30
- 主要テーブル:
  - item_learning_records: 学習記録
  - performance_stats: 事前計算された統計

**Redis**

- 用途: 高速キャッシュ、セッション管理
- TTL設定:
  - 項目選定結果: 5分
  - 復習スケジュール: 30分
  - パフォーマンス統計: 1時間
  - ユーザー設定: 24時間
- 構成: Google Cloud Memorystore

### メッセージング

**Google Cloud Pub/Sub**

- イベント発行用
- トピック構成:
  - `learning-algorithm-events`: ドメインイベント
  - `performance-analytics`: 分析イベント

### コンテナ・オーケストレーション

**Google Cloud Run**

- 単一サービスをコンテナとしてデプロイ
- 自動スケーリング
- CPU 最適化インスタンス

## デプロイ構成

```yaml
services:
  learning-algorithm-service:
    image: gcr.io/effect-project/learning-algorithm-service
    env:
      - DATABASE_URL
      - REDIS_URL
      - PUBSUB_TOPIC
      - ALGORITHM_VERSION  # SM2, SM18, FSRS
    scaling:
      min_instances: 2      # 高速応答のため最小2
      max_instances: 20
    resources:
      cpu: 2
      memory: 1Gi
    concurrency: 100
```

## データベース設計

### テーブル構造

**item_learning_records テーブル**:

```sql
CREATE TABLE item_learning_records (
    record_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    item_id UUID NOT NULL,
    -- SM-2 パラメータ
    easiness_factor DECIMAL(3,2) DEFAULT 2.5,
    repetition_count INTEGER DEFAULT 0,
    interval_days INTEGER DEFAULT 1,
    next_review_date DATE NOT NULL,
    -- 統計情報
    total_reviews INTEGER DEFAULT 0,
    correct_count INTEGER DEFAULT 0,
    streak_count INTEGER DEFAULT 0,
    average_response_time INTEGER,
    last_review_date TIMESTAMP,
    last_quality INTEGER,
    -- メタデータ
    status VARCHAR(20) DEFAULT 'NEW',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    
    CONSTRAINT uk_user_item UNIQUE (user_id, item_id),
    CONSTRAINT chk_ef CHECK (easiness_factor >= 1.3 AND easiness_factor <= 2.5),
    CONSTRAINT chk_quality CHECK (last_quality >= 0 AND last_quality <= 5)
);
```

### インデックス戦略

```sql
-- 項目選定用の複合インデックス
CREATE INDEX idx_review_schedule 
ON item_learning_records(user_id, next_review_date, status);

-- ステータス別の部分インデックス
CREATE INDEX idx_overdue_items 
ON item_learning_records(user_id, next_review_date) 
WHERE next_review_date < CURRENT_DATE;

-- 統計計算用
CREATE INDEX idx_performance 
ON item_learning_records(user_id, last_review_date);
```

### マイグレーション

```sql
-- 001_create_learning_records.sql
CREATE TABLE item_learning_records (...);

-- 002_add_performance_indexes.sql
CREATE INDEX idx_review_schedule ...;

-- 003_add_stats_table.sql
CREATE TABLE performance_stats (...);
```

## 非機能要件

### パフォーマンス要件

**レスポンスタイム目標**:

| 操作 | 目標値 | 最大値 |
|------|--------|--------|
| 項目選定 | 50ms | 100ms |
| 復習記録 | 30ms | 50ms |
| スケジュール計算 | 10ms | 20ms |
| 統計取得 | 200ms | 500ms |

**スループット目標**:

- 同時接続数: 1000
- リクエスト/秒: 5000
- 日次処理項目数: 1000万

### スケーラビリティ

**水平スケーリング**:

- ステートレス設計
- ユーザーごとの独立処理
- キャッシュの分散

**垂直スケーリング**:

- CPU 最適化（アルゴリズム計算）
- メモリ最適化（キャッシュ）

### 可用性

**目標**: 99.9%（月間ダウンタイム: 43分以内）

**実現方法**:

- マルチインスタンス構成
- ヘルスチェック
- 自動復旧
- グレースフルシャットダウン

### キャッシング戦略

**多層キャッシュ**:

1. **アプリケーション層**
   - インメモリキャッシュ（LRU）
   - ホットデータの保持

2. **Redis層**
   - 分散キャッシュ
   - セッション共有

3. **データベース層**
   - マテリアライズドビュー
   - 事前計算された統計

**キャッシュ無効化**:

- TTL ベース
- イベント駆動
- 手動パージ

### ロギング・モニタリング

**Google Cloud Logging**

構造化ログ形式：

```json
{
  "timestamp": "2025-01-01T00:00:00Z",
  "severity": "INFO",
  "service": "learning-algorithm",
  "message": "Review recorded",
  "labels": {
    "user_id": "user123",
    "item_id": "item456",
    "quality": 4,
    "response_time_ms": 2500
  }
}
```

**メトリクス**:

基本メトリクス:

- リクエスト数/レスポンスタイム
- エラー率
- CPU/メモリ使用率

カスタムメトリクス:

- SM-2 計算時間
- キャッシュヒット率
- 平均品質評価
- 85%ルール達成率

**アラート設定**:

- レスポンスタイム p99 > 100ms
- エラー率 > 1%
- キャッシュヒット率 < 80%
- データベース接続エラー

### セキュリティ

**ネットワーク**:

- VPC 内部通信のみ
- プライベートエンドポイント
- ファイアウォールルール

**データ保護**:

- データベース暗号化（at rest）
- TLS 1.3（in transit）
- 個人データの最小化

**アクセス制御**:

- サービスアカウント認証
- IAM ロール
- 最小権限の原則

### バックアップ・DR

**データベース**:

- 自動バックアップ（毎日）
- Point-in-time recovery（7日間）
- クロスリージョンレプリケーション

**復旧手順**:

1. 最新バックアップからリストア
2. キャッシュのウォームアップ
3. ヘルスチェック確認
4. トラフィック切り替え

**RTO/RPO**:

- RTO: 1時間
- RPO: 1時間

## 開発・テスト環境

### ローカル開発

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: learning_algorithm
      POSTGRES_USER: effect
      POSTGRES_PASSWORD: effect_password
    ports:
      - "5433:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6380:6379"

  learning-algorithm:
    build: .
    environment:
      DATABASE_URL: postgres://effect:effect_password@postgres:5432/learning_algorithm
      REDIS_URL: redis://redis:6379
    ports:
      - "50051:50051"
```

### テスト戦略

**ユニットテスト**:

- SM-2 アルゴリズムの正確性
- 品質評価ロジック
- 項目選定ロジック

**統合テスト**:

- データベース操作
- キャッシュ動作
- イベント発行

**パフォーマンステスト**:

- 負荷テスト（k6）
- レスポンスタイム計測
- メモリリーク検出

### CI/CD パイプライン

**GitHub Actions**:

1. テスト実行
   - ユニットテスト
   - 統合テスト
   - コードカバレッジ

2. ビルド
   - Docker イメージ作成
   - Container Registry へプッシュ

3. デプロイ
   - ステージング環境
   - カナリアデプロイ
   - 本番環境

## パフォーマンスチューニング

### アルゴリズム最適化

- 事前計算の活用
- バッチ処理
- 並列処理

### データベース最適化

- クエリ最適化
- コネクションプール調整
- パーティショニング検討

### キャッシュ最適化

- TTL の調整
- プリフェッチ
- ウォームアップ

## コスト最適化

1. **Cloud Run の最適化**
   - 最小インスタンス数の調整
   - CPU アロケーションの最適化

2. **データベースの最適化**
   - 適切なインスタンスサイズ
   - 不要なインデックスの削除

3. **キャッシュの活用**
   - Redis による DB 負荷軽減
   - 静的データのキャッシング
