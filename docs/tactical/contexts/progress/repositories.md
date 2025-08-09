# Progress Context - リポジトリ設計

## 概要

Progress Context のリポジトリは Event Store と Read Model の永続化を担当します。純粋な CQRS アーキテクチャのため、書き込みと読み取りが完全に分離されています。

## EventStore リポジトリ

### 責務

- イベントの永続化（Append-Only）
- イベントストリームの読み取り
- スナップショット管理

### 主要メソッド

- append_events: イベント追記
- read_events: ストリーム読み取り
- save_snapshot: スナップショット保存
- get_latest_snapshot: 最新スナップショット取得

## Projection リポジトリ

### 共通インターフェース

すべての投影リポジトリが実装する基本操作：

- save: 投影の保存/更新
- find_by_id: ID による取得
- save_batch: バッチ保存
- rebuild_from_events: イベントから再構築

### 個別リポジトリ

**DailyStatsRepository**

- 日別統計の管理
- ユーザーと日付による検索

**WeeklyStatsRepository**

- 週別統計の管理
- トレンド分析データの提供

**ItemStatsRepository**

- 項目別統計の管理
- 苦手/習得済み項目の抽出

**DomainStatsRepository**

- R/W/L/S 別統計の管理
- 領域間比較データの提供

**LevelStatsRepository**

- CEFR レベル別統計の管理
- 目標レベルの算出

**StreakRepository**

- 連続学習記録の管理
- ランキングデータの提供

## 実装上の考慮事項

### イベントソーシング

- イベントは追記のみ（イミュータブル）
- 厳密な順序保証
- イベントID による重複排除

### 更新戦略

- **リアルタイム**: 日別統計、連続記録
- **バッチ（5分）**: 週別統計、領域別統計
- **遅延評価**: アクセス頻度の低い統計

### パフォーマンス最適化

- Redis によるキャッシング
- 適切なインデックス設定
- 読み取り専用レプリカの活用

### スナップショット

- 1000イベントごとに作成
- 古いスナップショットの定期削除
- リビルド時の高速化
