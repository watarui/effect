# Progress Context - クエリ

## 概要

Progress Context は GraphQL API を通じて、様々な切り口から学習進捗データを提供します。すべてのクエリは Read Model から効率的にデータを取得します。

## 主要クエリ

### 1. GetDailyStats

**目的**: 日別の学習統計を取得

**パラメータ**:

- user_id: ユーザー識別子
- date: 対象日（オプション、デフォルトは今日）
- date_range: 期間指定（オプション）

**返却データ**:

- session_count: セッション数
- total_review_count: 総復習数
- correct_count: 正解数
- accuracy_rate: 正答率
- total_study_time: 総学習時間
- new_items_learned: 新規学習項目数
- items_mastered: 習得項目数

### 2. GetWeeklyTrend

**目的**: 週別の学習傾向を分析

**パラメータ**:

- user_id: ユーザー識別子
- weeks: 取得する週数（デフォルト: 4）

**返却データ**:

- weekly_stats: 週別統計の配列
- trend_direction: 上昇/横ばい/下降
- consistency_score: 継続性スコア（0-100）
- recommendations: 改善提案

### 3. GetCategoryProgress

**目的**: カテゴリ別（領域/レベル）の進捗

**パラメータ**:

- user_id: ユーザー識別子
- category_type: "domain" | "level"

**返却データ（domain の場合）**:

- reading: Reading の統計
- writing: Writing の統計
- listening: Listening の統計
- speaking: Speaking の統計

**返却データ（level の場合）**:

- a1: A1レベルの統計
- a2: A2レベルの統計
- b1: B1レベルの統計
- b2: B2レベルの統計
- c1: C1レベルの統計
- c2: C2レベルの統計

### 4. GetUserSummary

**目的**: ユーザーの全体的な学習サマリー

**パラメータ**:

- user_id: ユーザー識別子

**返却データ**:

- total_items_learned: 総学習項目数
- total_items_mastered: 総習得項目数
- total_study_hours: 総学習時間
- average_daily_minutes: 日平均学習時間
- current_streak: 現在の連続日数
- longest_streak: 最長連続日数
- overall_progress_score: 総合進捗スコア（0-100）
- next_milestone: 次のマイルストーン

### 5. GetLearningStreak

**目的**: 連続学習記録の詳細

**パラメータ**:

- user_id: ユーザー識別子

**返却データ**:

- current_streak: 現在の連続日数
- longest_streak: 最長記録
- last_activity_date: 最終活動日
- total_active_days: 総活動日数
- streak_calendar: カレンダー表示用データ

### 6. GetItemProgress

**目的**: 個別項目の学習進捗

**パラメータ**:

- user_id: ユーザー識別子
- item_ids: 項目ID配列（オプション）
- filter: "struggling" | "mastered" | "in_progress"

**返却データ**:

- items: 項目別統計の配列
  - item_id: 項目識別子
  - first_seen: 初回学習日
  - last_reviewed: 最終復習日
  - total_reviews: 総復習回数
  - accuracy_rate: 正答率
  - mastery_level: 習熟レベル

### 7. GetPerformanceMetrics

**目的**: 学習パフォーマンス指標

**パラメータ**:

- user_id: ユーザー識別子
- period: "7d" | "30d" | "90d" | "all"

**返却データ**:

- accuracy_trend: 正答率の推移
- learning_velocity: 学習速度
- retention_rate: 定着率
- optimal_review_timing: 最適復習タイミング達成率
- performance_score: パフォーマンススコア（0-100）

### 8. GetMilestones

**目的**: 達成済み・未達成のマイルストーン

**パラメータ**:

- user_id: ユーザー識別子
- status: "achieved" | "pending" | "all"

**返却データ**:

- milestones: マイルストーンの配列
  - type: マイルストーンの種類
  - achieved_at: 達成日時
  - details: 詳細情報
  - next_target: 次の目標

## GraphQL スキーマの特徴

### DataLoader による最適化

- N+1 問題の回避
- バッチ処理による効率化

### フィールドレベルの解決

- 必要なフィールドのみ取得
- 遅延評価による最適化

### リアルタイムサブスクリプション

- 進捗更新の即座な反映
- WebSocket による双方向通信

## キャッシング戦略

| クエリ | キャッシュ期間 | 無効化タイミング |
|--------|--------------|----------------|
| GetDailyStats（今日） | 5分 | 新イベント受信時 |
| GetDailyStats（過去） | 24時間 | 日次バッチ後 |
| GetWeeklyTrend | 1時間 | 統計更新時 |
| GetUserSummary | 10分 | 進捗更新時 |
| GetLearningStreak | 5分 | セッション完了時 |
