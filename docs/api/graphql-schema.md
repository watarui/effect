# GraphQL API スキーマ

## 概要

effect の GraphQL API は、クライアントアプリケーションとの主要なインターフェースです。
効率的なデータフェッチングと型安全性を提供します。

## スキーマ定義

### 基本型

```graphql
scalar UUID
scalar DateTime

enum LearningMode {
  MULTIPLE_CHOICE
  TYPING
  LISTENING
  SPEAKING
}

enum QuestionType {
  WORD_TO_MEANING
  MEANING_TO_WORD
  PRONUNCIATION
  SPELLING
}
```

### オブジェクト型

```graphql
type Word {
  id: UUID!
  text: String!
  meaning: String!
  pronunciation: String
  exampleSentences: [String!]!
  difficulty: Int!
  category: String!
  tags: [String!]!
  createdAt: DateTime!
  updatedAt: DateTime!
}

type LearningSession {
  id: UUID!
  userId: UUID!
  mode: LearningMode!
  startedAt: DateTime!
  completedAt: DateTime
  words: [Word!]!
  results: [QuestionResult!]!
  statistics: SessionStatistics!
}

type QuestionResult {
  wordId: UUID!
  word: Word!
  isCorrect: Boolean!
  responseTimeMs: Int!
  attemptNumber: Int!
}

type SessionStatistics {
  totalQuestions: Int!
  correctAnswers: Int!
  accuracy: Float!
  averageResponseTime: Int!
}

type UserProgress {
  userId: UUID!
  wordId: UUID!
  word: Word!
  repetitionCount: Int!
  easinessFactor: Float!
  intervalDays: Int!
  nextReviewDate: DateTime!
  lastReviewedAt: DateTime!
}

type LearningStreak {
  userId: UUID!
  currentStreak: Int!
  longestStreak: Int!
  lastStudyDate: DateTime!
}
```

### Query

```graphql
type Query {
  # 単語関連
  word(id: UUID!): Word
  words(
    filter: WordFilter
    orderBy: WordOrderBy
    limit: Int = 20
    offset: Int = 0
  ): WordConnection!

  # セッション関連
  session(id: UUID!): LearningSession
  sessions(
    userId: UUID!
    limit: Int = 20
    offset: Int = 0
  ): SessionConnection!

  # 進捗関連
  userProgress(userId: UUID!, wordId: UUID!): UserProgress
  userProgressList(
    userId: UUID!
    filter: ProgressFilter
  ): [UserProgress!]!

  dueForReview(userId: UUID!): [UserProgress!]!

  # 統計関連
  learningStatistics(userId: UUID!, period: StatisticsPeriod!): Statistics!
  streak(userId: UUID!): LearningStreak!
}

input WordFilter {
  categories: [String!]
  tags: [String!]
  difficultyMin: Int
  difficultyMax: Int
  searchText: String
}

enum WordOrderBy {
  CREATED_AT_DESC
  CREATED_AT_ASC
  DIFFICULTY_ASC
  DIFFICULTY_DESC
  ALPHABETICAL
}

input ProgressFilter {
  overdueOnly: Boolean
  masteredOnly: Boolean
  category: String
}

enum StatisticsPeriod {
  DAY
  WEEK
  MONTH
  YEAR
  ALL_TIME
}

type Statistics {
  totalWordsLearned: Int!
  totalSessions: Int!
  totalTimeMinutes: Int!
  averageAccuracy: Float!
  categoryBreakdown: [CategoryStats!]!
  dailyActivity: [DailyActivity!]!
}
```

### Mutation

```graphql
type Mutation {
  # 単語管理
  createWord(input: CreateWordInput!): CreateWordPayload!
  updateWord(input: UpdateWordInput!): UpdateWordPayload!
  deleteWord(id: UUID!): DeleteWordPayload!

  # 学習セッション
  startSession(input: StartSessionInput!): StartSessionPayload!
  answerQuestion(input: AnswerQuestionInput!): AnswerQuestionPayload!
  completeSession(sessionId: UUID!): CompleteSessionPayload!

  # バッチ操作
  importWords(input: ImportWordsInput!): ImportWordsPayload!
}

input CreateWordInput {
  text: String!
  meaning: String!
  pronunciation: String
  exampleSentences: [String!]
  difficulty: Int!
  category: String!
  tags: [String!]
}

input UpdateWordInput {
  id: UUID!
  text: String
  meaning: String
  pronunciation: String
  exampleSentences: [String!]
  difficulty: Int
  category: String
  tags: [String!]
}

input StartSessionInput {
  userId: UUID!
  mode: LearningMode!
  wordCount: Int!
  categories: [String!]
  difficultyRange: DifficultyRange
}

input DifficultyRange {
  min: Int!
  max: Int!
}

input AnswerQuestionInput {
  sessionId: UUID!
  wordId: UUID!
  answer: String!
  responseTimeMs: Int!
}

# Payload 型
type CreateWordPayload {
  word: Word
  errors: [Error!]
}

type StartSessionPayload {
  session: LearningSession
  firstQuestion: Question
  errors: [Error!]
}

type AnswerQuestionPayload {
  result: QuestionResult!
  nextQuestion: Question
  sessionProgress: SessionProgress!
  errors: [Error!]
}

type Question {
  wordId: UUID!
  questionType: QuestionType!
  questionText: String!
  options: [String!]
  hint: String
}

type SessionProgress {
  totalQuestions: Int!
  answeredQuestions: Int!
  correctAnswers: Int!
  remainingQuestions: Int!
}
```

### Subscription

```graphql
type Subscription {
  # リアルタイム学習追跡
  sessionUpdates(sessionId: UUID!): SessionUpdate!

  # 統計のリアルタイム更新
  statisticsUpdates(userId: UUID!): Statistics!
}

type SessionUpdate {
  sessionId: UUID!
  event: SessionEvent!
  timestamp: DateTime!
}

union SessionEvent = QuestionAnsweredEvent | SessionCompletedEvent

type QuestionAnsweredEvent {
  wordId: UUID!
  isCorrect: Boolean!
  newProgress: UserProgress
}

type SessionCompletedEvent {
  statistics: SessionStatistics!
  achievements: [Achievement!]
}
```

### エラー処理

```graphql
interface Error {
  message: String!
  code: String!
}

type ValidationError implements Error {
  message: String!
  code: String!
  field: String!
}

type BusinessError implements Error {
  message: String!
  code: String!
  details: String
}
```

## 認証・認可

```graphql
directive @auth(requires: Role = USER) on FIELD_DEFINITION

enum Role {
  USER
  ADMIN
}

extend type Query {
  # 認証が必要なクエリ
  myProgress: [UserProgress!]! @auth
  myStatistics: Statistics! @auth
}

extend type Mutation {
  # 全ての mutation は認証が必要
  createWord(input: CreateWordInput!): CreateWordPayload! @auth
}
```

## ページネーション

```graphql
interface Connection {
  edges: [Edge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

interface Edge {
  node: Node!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

type WordConnection implements Connection {
  edges: [WordEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type WordEdge implements Edge {
  node: Word!
  cursor: String!
}
```

## 使用例

### 単語の作成

```graphql
mutation CreateWord {
  createWord(input: {
    text: "ubiquitous"
    meaning: "遍在する、至る所にある"
    pronunciation: "juːˈbɪkwɪtəs"
    exampleSentences: ["Smartphones have become ubiquitous in modern society."]
    difficulty: 4
    category: "IELTS"
    tags: ["academic", "technology"]
  }) {
    word {
      id
      text
      meaning
    }
    errors {
      message
      code
    }
  }
}
```

### 学習セッションの開始

```graphql
mutation StartLearningSession {
  startSession(input: {
    userId: "123e4567-e89b-12d3-a456-426614174000"
    mode: MULTIPLE_CHOICE
    wordCount: 20
    categories: ["IELTS"]
    difficultyRange: { min: 3, max: 5 }
  }) {
    session {
      id
      mode
      words {
        id
        text
      }
    }
    firstQuestion {
      wordId
      questionText
      options
    }
  }
}
```
