# Learning Context - API スキーマ

## 概要

Learning Context の GraphQL API スキーマ定義です。学習セッションの管理とハイブリッド UI による学習フローをサポートします。

## GraphQL スキーマ

### 基本型定義

```graphql
scalar DateTime
scalar UUID

enum SessionType {
  REVIEW          # 復習のみ
  NEW_ITEMS       # 新規項目のみ
  MIXED           # 混合
}

enum SessionStatus {
  NOT_STARTED
  IN_PROGRESS
  COMPLETED
  ABANDONED
}

enum ItemSelectionStrategy {
  SMART_SELECTION    # AI による最適化選択
  DUE_FOR_REVIEW    # 復習期限優先
  WEAK_ITEMS        # 苦手項目優先
  RANDOM            # ランダム
}

enum AnswerRevealTrigger {
  USER_REQUESTED    # ユーザーが要求
  TIME_LIMIT        # 3秒経過
}

enum CorrectnessJudgment {
  AUTO_CONFIRMED           # 3秒経過で自動正解
  USER_CONFIRMED_CORRECT   # ユーザーが「わかった」
  USER_CONFIRMED_INCORRECT # ユーザーが「わからなかった」
}

enum MasteryStatus {
  UNKNOWN              # 未知
  SEARCHED            # 検索済み
  TESTED              # テスト済み
  TEST_FAILED         # テスト不正解
  SHORT_TERM_MASTERED # 短期記憶で習得
  LONG_TERM_MASTERED  # 長期記憶で習得
}

enum ProgressPeriod {
  TODAY
  THIS_WEEK
  THIS_MONTH
  ALL_TIME
}
```

### オブジェクト型

```graphql
type LearningSession {
  id: UUID!
  userId: UUID!
  startedAt: DateTime!
  completedAt: DateTime
  sessionType: SessionType!
  status: SessionStatus!
  totalItems: Int!
  completedItems: Int!
  currentItem: SessionItem
  items: [SessionItem!]!
  summary: SessionSummary
}

type SessionItem {
  itemId: UUID!
  spelling: String!
  presentedAt: DateTime
  answerRevealedAt: DateTime
  responseTimeMs: Int
  revealTrigger: AnswerRevealTrigger
  judgment: CorrectnessJudgment
  vocabularyDetails: VocabularyItemDetails
}

type VocabularyItemDetails {
  pronunciation: String
  definitions: [Definition!]!
  examples: [Example!]!
  synonyms: [String!]!
  antonyms: [String!]!
}

type Definition {
  id: UUID!
  meaning: String!
  meaningTranslation: String
  partOfSpeech: String!
  examples: [Example!]!
}

type Example {
  text: String!
  translation: String
}

type SessionSummary {
  totalItems: Int!
  completedItems: Int!
  correctCount: Int!
  incorrectCount: Int!
  averageResponseTimeMs: Int!
  accuracyRate: Float!
}

type ActiveSession {
  sessionId: UUID!
  startedAt: DateTime!
  totalItems: Int!
  completedItems: Int!
  currentItem: CurrentItemView
  remainingTime: Int # 秒単位
}

type CurrentItemView {
  itemId: UUID!
  spelling: String!
  presentedAt: DateTime!
  answerRevealed: Boolean!
  timeRemaining: Int # 3秒カウントダウン
}

type UserItemRecord {
  itemId: UUID!
  spelling: String!
  masteryStatus: MasteryStatus!
  totalAttempts: Int!
  correctCount: Int!
  lastReviewed: DateTime!
  nextReview: DateTime
  responseTimeStats: ResponseTimeStats!
}

type ResponseTimeStats {
  averageMs: Int!
  bestMs: Int!
  recentTrend: Trend!
}

enum Trend {
  IMPROVING
  STABLE
  DECLINING
}

type LearningProgress {
  period: ProgressPeriod!
  sessionsCompleted: Int!
  itemsLearned: Int!
  itemsMastered: Int!
  totalStudyTimeMinutes: Int!
  averageAccuracy: Float!
  streakDays: Int!
  dailyProgress: [DailyProgress!]!
}

type DailyProgress {
  date: DateTime!
  sessions: Int!
  itemsReviewed: Int!
  accuracy: Float!
  studyTimeMinutes: Int!
}

type SessionEdge {
  node: LearningSession!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

type SessionConnection {
  edges: [SessionEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}
```

### クエリ

```graphql
type Query {
  # セッション関連
  activeSession: ActiveSession
  session(id: UUID!): LearningSession
  sessionHistory(
    dateFrom: DateTime
    dateTo: DateTime
    first: Int = 20
    after: String
  ): SessionConnection!
  
  # 学習記録関連
  userItemRecords(itemIds: [UUID!]!): [UserItemRecord!]!
  masteryStatusCounts: MasteryStatusCounts!
  dueForReview(limit: Int = 50): [UserItemRecord!]!
  
  # 進捗・統計
  learningProgress(period: ProgressPeriod!): LearningProgress!
  learningStats: LearningStats!
}

type MasteryStatusCounts {
  unknown: Int!
  searched: Int!
  tested: Int!
  testFailed: Int!
  shortTermMastered: Int!
  longTermMastered: Int!
  total: Int!
}

type LearningStats {
  totalSessions: Int!
  totalItemsLearned: Int!
  averageAccuracy: Float!
  currentStreak: Int!
  bestStreak: Int!
}
```

### ミューテーション

```graphql
input StartSessionInput {
  itemCount: Int! # 1-100
  sessionType: SessionType!
  strategy: ItemSelectionStrategy!
}

input JudgeCorrectnessInput {
  judgment: CorrectnessJudgment!
  responseTimeMs: Int!
}

type Mutation {
  # セッション管理
  startSession(input: StartSessionInput!): UUID! @auth(requires: USER)
  
  # 学習フロー（3秒タイマー対応）
  presentNextItem(sessionId: UUID!): SessionItem! @auth(requires: USER)
  revealAnswer(sessionId: UUID!, itemId: UUID!): SessionItem! @auth(requires: USER)
  judgeCorrectness(
    sessionId: UUID!
    itemId: UUID!
    input: JudgeCorrectnessInput!
  ): SessionItem! @auth(requires: USER)
  
  # セッション終了
  completeSession(sessionId: UUID!): LearningSession! @auth(requires: USER)
  abandonSession(sessionId: UUID!): Boolean! @auth(requires: USER)
}
```

### サブスクリプション

```graphql
type Subscription {
  # セッションの進行状況をリアルタイム配信
  sessionProgress(sessionId: UUID!): SessionProgressUpdate!
  
  # 3秒タイマーのカウントダウン
  itemTimer(sessionId: UUID!, itemId: UUID!): TimerUpdate!
}

type SessionProgressUpdate {
  sessionId: UUID!
  completedItems: Int!
  totalItems: Int!
  currentAccuracy: Float!
}

type TimerUpdate {
  itemId: UUID!
  remainingSeconds: Int!
  autoRevealAt: DateTime!
}
```

## 認証・認可

### ディレクティブ

```graphql
directive @auth(requires: Role = USER) on FIELD_DEFINITION

enum Role {
  USER
  ADMIN
}
```

### 認可ルール

- すべての学習操作は認証必須
- ユーザーは自分のセッション・記録のみアクセス可能
- 管理者は分析目的で全データ閲覧可能

## エラーハンドリング

### エラー型

```graphql
interface Error {
  message: String!
  code: String!
}

type SessionNotFoundError implements Error {
  message: String!
  code: String!
  sessionId: String!
}

type InvalidStateError implements Error {
  message: String!
  code: String!
  currentState: String!
  attemptedAction: String!
}

type RateLimitError implements Error {
  message: String!
  code: String!
  retryAfter: Int! # 秒単位
}
```

### エラーコード

- `SESSION_NOT_FOUND`: セッションが見つからない
- `SESSION_ALREADY_ACTIVE`: すでにアクティブなセッションがある
- `INVALID_ITEM_COUNT`: 項目数が範囲外（1-100）
- `INSUFFICIENT_ITEMS`: 学習可能な項目が不足
- `INVALID_STATE`: 無効な状態遷移
- `RATE_LIMIT_EXCEEDED`: レート制限超過

## 使用例

### セッション開始と学習フロー

```graphql
# 1. セッション開始
mutation StartLearning {
  startSession(input: {
    itemCount: 25
    sessionType: MIXED
    strategy: SMART_SELECTION
  })
}

# 2. 次の項目を取得
mutation GetNextItem {
  presentNextItem(sessionId: "session-id") {
    itemId
    spelling
    presentedAt
  }
}

# 3. 解答表示（ユーザー要求または3秒後自動）
mutation ShowAnswer {
  revealAnswer(sessionId: "session-id", itemId: "item-id") {
    vocabularyDetails {
      definitions {
        meaning
        meaningTranslation
        examples {
          text
          translation
        }
      }
    }
  }
}

# 4. 正誤判定
mutation JudgeItem {
  judgeCorrectness(
    sessionId: "session-id"
    itemId: "item-id"
    input: {
      judgment: USER_CONFIRMED_CORRECT
      responseTimeMs: 2500
    }
  ) {
    judgment
  }
}

# 5. セッション完了
mutation FinishLearning {
  completeSession(sessionId: "session-id") {
    summary {
      totalItems
      correctCount
      accuracyRate
    }
  }
}
```

### 進捗確認

```graphql
query MyProgress {
  learningProgress(period: THIS_WEEK) {
    sessionsCompleted
    itemsMastered
    averageAccuracy
    streakDays
    dailyProgress {
      date
      accuracy
      studyTimeMinutes
    }
  }
}
```

## パフォーマンス考慮事項

1. **activeSession**: キャッシュなし（リアルタイム性重視）
2. **sessionHistory**: カーソルベースページネーション
3. **userItemRecords**: バッチ取得で N+1 問題回避
4. **サブスクリプション**: セッション参加者のみに配信
