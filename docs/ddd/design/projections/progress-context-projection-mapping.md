# Progress Context プロジェクション設計とGraphQLマッピング

## 概要

このドキュメントは、Progress Context のプロジェクション設計と GraphQL クエリとの対応関係を明確化します。

## プロジェクション一覧

### 1. UserProgressSummaryProjection

**責務**: ユーザーの全体的な学習進捗サマリー（現在のストリークを含む）

**保持するデータ**:

- `total_items_learned`: 学習済み項目総数
- `total_sessions_completed`: 完了セッション総数
- `total_learning_time_seconds`: 総学習時間
- `average_session_score`: 平均セッションスコア
- `current_streak_days`: 現在の連続学習日数
- `last_activity_date`: 最終学習日

**対応する GraphQL クエリ**:

```graphql
type UserProgress {
  totalItemsLearned: Int!
  totalSessionsCompleted: Int!
  totalLearningTimeMinutes: Int!
  averageSessionScore: Float!
  currentStreakDays: Int!
  lastActivityDate: DateTime!
}

query GetUserProgress($userId: UUID!) {
  userProgress(userId: $userId) {
    totalItemsLearned
    totalSessionsCompleted
    totalLearningTimeMinutes
    averageSessionScore
    currentStreakDays
    lastActivityDate
  }
}
```

### 2. DailyStatsProjection

**責務**: 日別の学習統計（セッション数、学習時間、項目数）

**保持するデータ**:

- `date`: 日付
- `sessions_completed`: その日の完了セッション数
- `items_reviewed`: その日の復習項目数
- `learning_time_seconds`: その日の学習時間
- `average_score`: その日の平均スコア

**対応する GraphQL クエリ**:

```graphql
type DailyStats {
  date: Date!
  sessionsCompleted: Int!
  itemsReviewed: Int!
  learningTimeMinutes: Int!
  averageScore: Float!
}

query GetDailyStats($userId: UUID!, $startDate: Date!, $endDate: Date!) {
  dailyStats(userId: $userId, startDate: $startDate, endDate: $endDate) {
    date
    sessionsCompleted
    itemsReviewed
    learningTimeMinutes
    averageScore
  }
}
```

### 3. CategoryProgressProjection

**責務**: カテゴリ別の進捗（Reading, Writing, Listening, Speaking）

**保持するデータ**:

- `category`: カテゴリ名（R/W/L/S）
- `items_learned`: 学習済み項目数
- `items_total`: カテゴリ内の総項目数
- `mastery_rate`: 習得率
- `average_difficulty`: 平均難易度

**対応する GraphQL クエリ**:

```graphql
type CategoryProgress {
  category: SkillCategory!
  itemsLearned: Int!
  itemsTotal: Int!
  masteryRate: Float!
  averageDifficulty: Float!
}

enum SkillCategory {
  READING
  WRITING
  LISTENING
  SPEAKING
}

query GetCategoryProgress($userId: UUID!) {
  categoryProgress(userId: $userId) {
    category
    itemsLearned
    itemsTotal
    masteryRate
    averageDifficulty
  }
}
```

## イベント処理マッピング

### SessionCompleted イベント

```rust
pub fn handle_session_completed(&mut self, event: SessionCompleted) {
    // UserProgressSummaryProjection
    - total_sessions_completed += 1
    - total_learning_time_seconds += event.duration_seconds
    - average_session_score を再計算
    - last_activity_date を更新
    - current_streak_days を更新（必要に応じて）
    
    // DailyStatsProjection
    - 該当日の sessions_completed += 1
    - 該当日の learning_time_seconds += event.duration_seconds
    - 該当日の average_score を再計算
}
```

### ItemReviewed イベント

```rust
pub fn handle_item_reviewed(&mut self, event: ItemReviewed) {
    // UserProgressSummaryProjection
    - total_items_learned を更新（初回学習の場合）
    
    // DailyStatsProjection
    - 該当日の items_reviewed += 1
    
    // CategoryProgressProjection
    - 該当カテゴリの items_learned を更新
    - 該当カテゴリの mastery_rate を再計算
}
```

## パフォーマンス最適化

### 1. 事前集計

- 各プロジェクションは事前集計されたデータを保持
- GraphQL クエリ実行時の計算を最小化

### 2. 非同期更新

- 重要でない統計は非同期で更新
- SessionCompleted の基本情報は同期更新

### 3. キャッシュ戦略

- 頻繁にアクセスされるサマリーデータはキャッシュ
- 日別データは1日単位でキャッシュ

## 今後の拡張

### 1. WeeklyProgressProjection

- 週次サマリーの追加
- トレンド分析のサポート

### 2. SkillRadarProjection

- スキル別のレーダーチャート用データ
- IELTS スコア推定との連携

### 3. LearningPatternProjection

- 学習パターンの分析
- 最適な学習時間の提案
