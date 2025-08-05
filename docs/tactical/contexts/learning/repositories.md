# Learning Context - リポジトリ設計

## 概要

Learning Context には 2 つの集約が存在し、それぞれに対応するリポジトリを定義します：

- `LearningSession`：学習セッションの管理
- `UserItemRecord`：ユーザーの項目別学習記録

## LearningSessionRepository

学習セッションの永続化を担当するリポジトリです。

### 主要な責務

- セッションの基本的な CRUD 操作
- アクティブセッションの管理
- セッション履歴の検索
- 統計情報の集計
- タイムアウト処理のサポート

### インターフェース設計

**基本的な CRUD 操作**:

- `find_by_id`: ID でセッションを取得
- `save`: セッションを保存（新規作成または更新）
- `delete`: セッションを削除（通常は使用しない）

**ユーザー関連のクエリ**:

- `find_active_by_user`: ユーザーのアクティブなセッションを取得
- `find_by_user_paginated`: ユーザーの全セッションをページネーションで取得
- `find_by_user_and_date_range`: 特定期間のセッションを取得

**統計関連のクエリ**:

- `count_completed_by_user`: 完了セッション数を取得
- `count_completed_today_by_user`: 本日の完了セッション数を取得

**管理用クエリ**:

- `find_stale_sessions`: 長時間放置されたセッションを取得（タイムアウト処理用）

## UserItemRecordRepository

ユーザーと項目の学習記録を管理するリポジトリです。

### 主要な責務

- 学習記録の基本的な CRUD 操作
- 習熟状態による検索
- 復習スケジュールの管理
- 学習履歴の追跡
- 統計情報の提供

### インターフェース設計

**基本的な CRUD 操作**:

- `find_by_user_and_item`: ユーザーと項目のペアで記録を取得
- `save`: 記録を保存（新規作成または更新）
- `delete`: 記録を削除（リセット時のみ使用）

**バッチ操作**:

- `find_by_user_and_items`: 複数項目の記録を一括取得
- `save_batch`: 複数の記録を一括保存

**習熟状態による検索**:

- `find_by_user_and_status`: 特定の習熟状態の項目を取得
- `find_new_items_for_user`: 未学習の項目を取得
- `find_weak_items_for_user`: 苦手項目を取得

**復習スケジュール関連**:

- `find_due_for_review`: 復習期限が来た項目を取得
- `find_overdue_items`: 復習期限を過ぎた項目を取得
- `find_next_review_items`: 次回復習予定の項目を取得

**統計関連**:

- `count_by_user_and_status`: 習熟状態別の項目数を取得
- `count_mastered_items`: 習得済み項目数を取得
- `count_total_responses`: 総回答数を取得
- `calculate_average_accuracy`: 平均正答率を計算

**学習アルゴリズム向けクエリ**:

- `find_recent_responses`: 最近の回答履歴を取得
- `find_learning_curve_data`: 学習曲線データを取得

## 実装上の考慮事項

### パフォーマンス最適化

- 頻繁にアクセスされる UserItemRecord はキャッシュを考慮
- バッチ操作で N+1 問題を回避
- インデックスの適切な設定（user_id, item_id, mastery_status）

### データ整合性

- LearningSession と UserItemRecord の整合性はイベント駆動で保証
- トランザクション境界は各集約ごとに設定

### Progress Context との連携

- UserItemRecord は Progress Context とデータを共有
- 変更はイベントを通じて伝播
- 読み取りモデルの非正規化を許容

### 履歴データの管理

- ResponseRecord は追記のみ（イミュータブル）
- 古い履歴データのアーカイブ戦略を考慮
- 分析用途のための効率的なクエリ設計
