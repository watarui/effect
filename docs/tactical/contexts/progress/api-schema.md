# Progress Context - API スキーマ

## 概要

Progress Context は GraphQL API を通じて統計データを提供します。フロントエンドからの柔軟なクエリに対応し、必要なデータのみを効率的に取得できます。

## GraphQL スキーマ定義

### 基本型定義

```graphql
scalar Date
scalar DateTime

enum Period {
  DAY
  WEEK
  MONTH
  QUARTER
  YEAR
  ALL
}

enum TrendDirection {
  UP
  STABLE
  DOWN
}

enum CategoryType {
  DOMAIN
  LEVEL
}

enum Domain {
  READING
  WRITING
  LISTENING
  SPEAKING
}

enum CEFRLevel {
  A1
  A2
  B1
  B2
  C1
  C2
}

enum MilestoneType {
  FIRST_SESSION
  CONSECUTIVE_DAYS
  ITEMS_MASTERED
  LEVEL_COMPLETE
  PERFECT_WEEK
  STUDY_HOURS
}
```

### 主要な型

```graphql
type DailyStats {
  date: Date!
  sessionCount: Int!
  totalReviewCount: Int!
  correctCount: Int!
  incorrectCount: Int!
  accuracyRate: Float!
  totalStudyTime: Int! # 分単位
  averageResponseTime: Int! # ミリ秒
  newItemsLearned: Int!
  itemsMastered: Int!
}

type WeeklyStats {
  weekStart: Date!
  weekEnd: Date!
  activeDays: Int!
  totalReviewCount: Int!
  averageDailyReviews: Float!
  masteryProgression: Int!
  consistencyScore: Float!
  trend: TrendDirection!
}

type DomainProgress {
  domain: Domain!
  totalItems: Int!
  masteredItems: Int!
  inProgressItems: Int!
  notStartedItems: Int!
  averageAccuracy: Float!
  timeSpent: Int! # 分単位
  progressPercentage: Float!
}

type LevelProgress {
  level: CEFRLevel!
  totalItems: Int!
  masteredItems: Int!
  inProgressItems: Int!
  notStartedItems: Int!
  estimatedCompletion: Date
  progressPercentage: Float!
}

type ItemStats {
  itemId: ID!
  firstSeen: DateTime!
  lastReviewed: DateTime!
  totalReviews: Int!
  correctCount: Int!
  accuracyRate: Float!
  averageResponseTime: Int!
  masteryLevel: Float!
  nextReviewDate: DateTime
}

type LearningStreak {
  currentStreak: Int!
  longestStreak: Int!
  lastActivityDate: Date!
  totalActiveDays: Int!
  streakCalendar: [StreakDay!]!
  isActiveToday: Boolean!
}

type StreakDay {
  date: Date!
  isActive: Boolean!
  reviewCount: Int
}

type UserSummary {
  userId: ID!
  totalItemsLearned: Int!
  totalItemsMastered: Int!
  totalStudyHours: Float!
  averageDailyMinutes: Float!
  currentStreak: Int!
  longestStreak: Int!
  overallProgressScore: Int! # 0-100
  nextMilestone: Milestone
  recentAchievements: [Milestone!]!
}

type Milestone {
  type: MilestoneType!
  achievedAt: DateTime
  details: String!
  progress: Float!
  target: Int!
  reward: String
}

type PerformanceMetrics {
  period: Period!
  accuracyTrend: [DataPoint!]!
  learningVelocity: Float!
  retentionRate: Float!
  optimalReviewTiming: Float!
  performanceScore: Int! # 0-100
  insights: [String!]!
}

type DataPoint {
  timestamp: DateTime!
  value: Float!
}
```

### クエリ定義

```graphql
type Query {
  # 日別統計
  dailyStats(
    userId: ID!
    date: Date
    dateRange: DateRange
  ): [DailyStats!]!

  # 週別トレンド
  weeklyTrend(
    userId: ID!
    weeks: Int = 4
  ): WeeklyTrendResponse!

  # カテゴリ別進捗
  categoryProgress(
    userId: ID!
    categoryType: CategoryType!
  ): CategoryProgressResponse!

  # ユーザーサマリー
  userSummary(
    userId: ID!
  ): UserSummary!

  # 連続学習記録
  learningStreak(
    userId: ID!
  ): LearningStreak!

  # 項目別進捗
  itemProgress(
    userId: ID!
    itemIds: [ID!]
    filter: ItemFilter
    limit: Int = 100
    offset: Int = 0
  ): ItemProgressResponse!

  # パフォーマンス指標
  performanceMetrics(
    userId: ID!
    period: Period!
  ): PerformanceMetrics!

  # マイルストーン
  milestones(
    userId: ID!
    status: MilestoneStatus
  ): [Milestone!]!
}
```

### レスポンス型

```graphql
type WeeklyTrendResponse {
  weeklyStats: [WeeklyStats!]!
  trendDirection: TrendDirection!
  consistencyScore: Float!
  recommendations: [String!]!
}

type CategoryProgressResponse {
  domainProgress: [DomainProgress!]
  levelProgress: [LevelProgress!]
  overallCompletion: Float!
}

type ItemProgressResponse {
  items: [ItemStats!]!
  totalCount: Int!
  hasMore: Boolean!
}

input DateRange {
  start: Date!
  end: Date!
}

enum ItemFilter {
  ALL
  STRUGGLING
  MASTERED
  IN_PROGRESS
  OVERDUE
}

enum MilestoneStatus {
  ACHIEVED
  PENDING
  ALL
}
```

### サブスクリプション

```graphql
type Subscription {
  # リアルタイム進捗更新
  progressUpdated(userId: ID!): ProgressUpdate!

  # ストリーク更新通知
  streakUpdated(userId: ID!): LearningStreak!

  # マイルストーン達成通知
  milestoneAchieved(userId: ID!): Milestone!
}

type ProgressUpdate {
  type: String!
  timestamp: DateTime!
  data: JSON!
}
```

## 使用例

### 日別統計の取得

```graphql
query GetDailyStats($userId: ID!, $date: Date) {
  dailyStats(userId: $userId, date: $date) {
    date
    sessionCount
    totalReviewCount
    accuracyRate
    totalStudyTime
    itemsMastered
  }
}
```

### カテゴリ別進捗の取得

```graphql
query GetCategoryProgress($userId: ID!) {
  categoryProgress(userId: $userId, categoryType: DOMAIN) {
    domainProgress {
      domain
      totalItems
      masteredItems
      progressPercentage
    }
  }
}
```

### ユーザーサマリーの取得

```graphql
query GetUserSummary($userId: ID!) {
  userSummary(userId: $userId) {
    totalItemsMastered
    totalStudyHours
    currentStreak
    overallProgressScore
    nextMilestone {
      type
      progress
      target
    }
  }
}
```

## パフォーマンス最適化

### DataLoader の使用

N+1 問題を回避するためのバッチローディング：

```typescript
// 実装例（概念的）
const itemLoader = new DataLoader(async (itemIds) => {
  const items = await repository.findByIds(itemIds);
  return itemIds.map(id => items.find(item => item.id === id));
});
```

### フィールドレベルの解決

必要なフィールドのみを選択的に取得：

```graphql
# 最小限のフィールドのみ
query MinimalStats($userId: ID!) {
  dailyStats(userId: $userId) {
    accuracyRate
    totalStudyTime
  }
}
```

### キャッシング

Redis による結果キャッシュ：

- TTL: クエリタイプに応じて 5分〜24時間
- キー: クエリ名 + パラメータのハッシュ
- 無効化: 関連イベント受信時

## エラーハンドリング

```graphql
type Error {
  code: String!
  message: String!
  field: String
}

union DailyStatsResult = DailyStats | Error
```

## セキュリティ

### 認証・認可

- JWT トークンによる認証
- ユーザーは自分のデータのみアクセス可能

### レート制限

- 1分あたり 100 リクエスト/ユーザー
- 複雑なクエリは追加制限

### クエリ深度制限

- 最大深度: 5
- 最大複雑度: 1000
