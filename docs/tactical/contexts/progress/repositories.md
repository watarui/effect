# Progress Context - リポジトリ設計

## 概要

Progress Context は純粋な CQRS Read Model であり、他のコンテキストから発行されるイベントを集約して、様々な統計情報を提供します。

このコンテキストの特徴：

- **集約なし**：全てがイベントから生成される Read Model
- **イベントソーシング**：イベントの履歴から状態を再構築
- **結果整合性**：リアルタイム性より正確性を優先

## EventStore

イベントの永続化と読み取りを担当する特別なリポジトリです。

### 主要な責務

- イベントの追記（Append-Only）
- イベントストリームの読み取り
- グローバルイベントの順序保証
- スナップショットによる最適化

### インターフェース設計

**基本操作**:

- `append_events`: イベントを追記
- `read_events`: ストリームからイベントを読み取り
- `read_all_events`: グローバルイベントストリームを読み取り
- `read_events_by_type`: イベントタイプでフィルタリング

**スナップショット**:

- `save_snapshot`: スナップショットの保存
- `get_latest_snapshot`: 最新のスナップショットを取得

## ProjectionRepository

各種プロジェクション（読み取りモデル）の永続化を担当します。

### 共通インターフェース

すべてのプロジェクションリポジトリが実装すべき基本インターフェース：

**基本操作**:

- `save`: プロジェクションの保存（作成/更新）
- `find_by_id`: ID による取得
- `delete`: プロジェクションの削除

**バッチ操作**:

- `save_batch`: 複数のプロジェクションを一括保存
- `rebuild_from_events`: イベントから再構築

### 個別のプロジェクションリポジトリ

#### DailyStatsRepository

日別統計のリポジトリ。

**特有のクエリ**:

- `find_by_user_and_date`: ユーザーと日付で取得
- `find_by_user_and_date_range`: 期間指定で取得
- `find_latest_by_user`: 最新の日別統計を取得

#### WeeklyStatsRepository

週別統計のリポジトリ。

**特有のクエリ**:

- `find_by_user_and_week`: ユーザーと週で取得
- `find_recent_weeks`: 最近N週間の統計を取得
- `calculate_trend`: 週別トレンドを計算

#### ItemStatsRepository

項目別統計のリポジトリ。

**特有のクエリ**:

- `find_by_user_and_item`: ユーザーと項目で取得
- `find_by_user_and_accuracy`: 正答率で項目を検索
- `find_struggling_items`: 苦手項目を取得
- `find_mastered_items`: 習得済み項目を取得

#### DomainStatsRepository

領域別（R/W/L/S）統計のリポジトリ。

**特有のクエリ**:

- `find_by_user_and_domain`: ユーザーと領域で取得
- `find_all_domains_by_user`: 全領域の統計を取得
- `compare_domains`: 領域間の比較データを取得

#### LevelStatsRepository

CEFR レベル別統計のリポジトリ。

**特有のクエリ**:

- `find_by_user_and_level`: ユーザーとレベルで取得
- `find_all_levels_by_user`: 全レベルの統計を取得
- `find_next_target_level`: 次の目標レベルを取得

#### StreakRepository

連続学習記録のリポジトリ。

**特有のクエリ**:

- `find_by_user`: ユーザーの連続記録を取得
- `find_top_streaks`: 上位の連続記録を取得
- `check_streak_status`: 連続記録の状態を確認

## 実装上の考慮事項

### イベントソーシングの実装

- イベントは追記のみ（イミュータブル）
- イベントの順序は厳密に保証
- イベントIDによる重複排除

### プロジェクションの更新戦略

**リアルタイム更新**:

- 重要な統計（日別、連続記録）は即座に更新
- イベントハンドラーで非同期処理

**バッチ更新**:

- 集計の重い統計は定期バッチで更新
- 深夜などの低負荷時に再計算

### スナップショット戦略

- 1000イベントごとにスナップショット作成
- 古いスナップショットは定期的に削除
- リビルド時はスナップショットから開始

### パフォーマンス最適化

- 頻繁にアクセスされるプロジェクションはキャッシュ
- インデックスの適切な設定
- 読み取り専用レプリカの活用
