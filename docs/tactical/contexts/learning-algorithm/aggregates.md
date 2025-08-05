# Learning Algorithm Context - 集約設計

## 概要

Learning Algorithm Context は、Effect プロジェクトの学習効果を最大化する中核コンテキストです。科学的に実証された SM-2（SuperMemo 2）アルゴリズムを基盤に、最適な復習タイミングと項目選定を実現します。

### 主要な責務

- **項目選定**: 学習戦略に基づいて最適な項目を選定
- **復習スケジューリング**: SM-2 アルゴリズムによる次回復習日の計算
- **難易度管理**: 各項目の難易度係数（Easiness Factor）の調整
- **学習統計**: 正答率、習熟度、パフォーマンスの追跡

### 設計方針

- 項目ごとに独立した集約（スケーラビリティと並行性を重視）
- 反応時間を考慮した品質評価（0-5スケール）
- 85%ルールに基づく動的な難易度調整
- 科学的根拠に基づいた学習効率の最適化

## 集約の設計

### 1. ItemLearningRecord（項目学習記録）- 集約ルート

ユーザーと項目の組み合わせごとの学習状態を管理します。

**主要な属性**:

- record_id: 記録識別子（user_id + item_id の複合キー）
- user_id: ユーザー識別子
- item_id: 項目識別子

**SM-2 アルゴリズム関連**:

- easiness_factor: 難易度係数（1.3-2.5）
- repetition_count: 連続正解回数
- interval_days: 現在の復習間隔（日数）
- next_review_date: 次回復習予定日

**統計情報**:

- total_reviews: 総復習回数
- correct_count: 正解回数
- streak_count: 現在の連続正解数
- average_response_time: 平均反応時間
- last_review_date: 最終復習日
- last_quality: 最後の品質評価（0-5）

**ReviewStatus（復習状態）**:

- New: 未学習
- Learning: 学習中（短期記憶形成中）
- Review: 通常復習
- Overdue: 期限切れ
- Suspended: 一時停止中

**不変条件**:

- easiness_factor は 1.3 以上 2.5 以下
- 品質評価は 0-5 の範囲
- 連続正解回数は正解時にのみ増加

### 2. SelectionCriteria（選定基準）- 値オブジェクト

項目選定時の評価基準を表現します。

**主要な属性**:

- priority_score: 優先度スコア（0.0-1.0）
- selection_reason: 選定理由
- urgency_factor: 緊急度（期限切れ日数など）
- difficulty_match: 現在の実力との適合度

**SelectionReason（選定理由）**:

- NewItem: 新規項目
- DueForReview: 復習予定
- Overdue: 期限切れ
- WeakItem: 苦手項目
- AIRecommended: AI推薦

### 3. LearningPerformance（学習パフォーマンス）- 値オブジェクト

ユーザーの現在のパフォーマンスを表現します。

**主要な属性**:

- recent_accuracy_rate: 最近の正答率
- average_response_time: 平均反応時間
- optimal_difficulty_level: 最適な難易度レベル
- learning_velocity: 学習速度

## ドメインサービス

### SM2Calculator

SM-2 アルゴリズムの計算を実装するドメインサービス。

**主要なメソッド**:

- `calculate_next_interval`: 次回復習間隔の計算
- `update_easiness_factor`: 難易度係数の更新
- `calculate_quality`: 反応時間から品質評価を算出

**SM-2 アルゴリズムの計算式**:

```
新しい間隔 = 前回の間隔 × 難易度係数
難易度係数 = EF + (0.1 - (5 - q) × (0.08 + (5 - q) × 0.02))
```

### PerformanceAnalyzer

学習パフォーマンスを分析するドメインサービス。

**主要なメソッド**:

- `analyze_recent_performance`: 最近のパフォーマンスを分析
- `calculate_optimal_difficulty`: 85%ルールに基づく最適難易度を計算
- `detect_learning_patterns`: 学習パターンの検出

## 設計上の重要な決定

### 85%ルールの実装

認知科学研究に基づく「85%正答率が最適」という原則：

- 正答率が85%を大きく上回る → より難しい項目を選定
- 正答率が85%を下回る → より簡単な項目を選定
- 動的な難易度調整で最適な学習状態を維持

### 反応時間の活用

単純な正誤だけでなく、反応時間も品質評価に反映：

- 即答（3秒以内）: 品質5
- 迅速（10秒以内）: 品質4
- 標準（30秒以内）: 品質3
- 遅延（30秒以上）: 品質2以下

### 項目ごとの独立性

各項目の学習記録は独立した集約：

- 高い並行性とスケーラビリティ
- 項目間の依存関係なし
- 大量の項目でもパフォーマンス劣化なし

### 学習の個別最適化

ユーザーごと、項目ごとに最適化：

- 個人の記憶特性に適応
- 項目の難易度に応じた調整
- 継続的な学習データによる改善
