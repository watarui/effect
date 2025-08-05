# Learning Algorithm Context - リポジトリ設計

## 概要

Learning Algorithm Context には 1 つの主要な集約が存在します：

- `ItemLearningRecord`：SM-2 アルゴリズムに基づく学習記録の管理

このコンテキストは学習アルゴリズムの計算に特化しており、UI 表示用の `UserItemRecord` (Learning Context) とは明確に分離されています。

## ItemLearningRecordRepository

学習アルゴリズム用の記録を管理するリポジトリです。

### 主要な責務

- 学習記録の基本的な CRUD 操作
- SM-2 アルゴリズム用のデータ取得
- 最適な項目選定のサポート
- 学習パフォーマンスの分析
- 統計情報の提供

### インターフェース設計

**基本的な CRUD 操作**:

- `find_by_user_and_item`: ユーザーと項目の組み合わせで記録を取得
- `save`: 記録を保存（新規作成または更新）
- `save_batch`: 複数の記録を一括保存（バッチ処理用）
- `delete`: 記録を削除（通常は使用しない）

**アルゴリズム用クエリ**:

- `find_due_for_review`: 次回復習日が到来している項目を取得
- `find_new_items`: 新規項目（未学習）を取得
- `find_overdue_items`: 期限切れの項目を取得
- `find_learning_items`: 学習中（短期記憶形成中）の項目を取得

**最適化用クエリ**:

- `find_optimal_items`: 学習最適化のための項目選定
- `get_user_performance`: ユーザーの学習パフォーマンスを取得
- `calculate_optimal_difficulty`: 85%ルールに基づく最適難易度を計算

**統計・分析用**:

- `get_easiness_distribution`: Easiness Factor の分布を取得
- `get_retention_curve`: 保持曲線データを取得
- `get_learning_velocity`: 学習速度を計算
- `get_accuracy_trend`: 正答率の推移を取得

**バルク操作**:

- `delete_all_by_user`: ユーザーの全記録を削除（アカウント削除時）
- `reset_overdue_items`: 期限切れ項目のリセット

## 実装上の考慮事項

### SM-2 アルゴリズムの実装

```
OptimalItemSelection の構造:
- new_items: 新規学習項目
- review_items: 復習項目
- overdue_items: 期限切れ項目
- learning_items: 短期記憶形成中

選定比率（デフォルト）:
- 新規 : 復習 = 1 : 4
- 期限切れは最優先
- 難易度は分散させる
```

### パフォーマンス最適化

- 複合インデックス: (user_id, next_review_date)
- 部分インデックス: status = 'Review' の項目のみ
- 定期的な統計の事前計算
- バッチ処理による一括更新

### データ整合性

**Learning Context との連携**:

- ItemLearningRecord は計算専用
- UserItemRecord は表示専用
- イベント経由で同期を保つ
- 直接的な依存関係なし

### アルゴリズムのバージョン管理

- 現在は SM-2 を使用
- 将来的には SM-18、FSRS などへの拡張を考慮
- アルゴリズムバージョンを記録に保持
- 異なるアルゴリズムの共存をサポート

### 学習セッション最適化

**SessionConfig による制御**:

- item_count: セッションの項目数
- new_item_ratio: 新規項目の比率
- difficulty_variance: 難易度の分散度
- time_limit: 時間制限（オプション）

**最適化の考慮事項**:

- 認知負荷の分散
- 難易度の適切な混合
- 学習効率の最大化
- ユーザーの集中力維持

### 統計データの活用

**UserPerformance の内容**:

- overall_accuracy: 全体正答率
- recent_accuracy: 最近の正答率（直近100回）
- average_easiness: 平均難易度係数
- learning_streak: 連続学習日数
- velocity_trend: 学習速度の傾向

これらのデータは：

- Progress Context に提供
- AI Integration Context での推薦に活用
- 学習戦略の自動調整に使用
