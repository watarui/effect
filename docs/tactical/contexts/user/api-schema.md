# User Context - API スキーマ

## 概要

User Context の GraphQL API スキーマ定義です。認証・認可、プロファイル管理の API を提供します。

## GraphQL スキーマ

### 基本型定義

```graphql
scalar DateTime
scalar UUID

enum UserRole {
  ADMIN
  USER
}

enum AccountStatus {
  ACTIVE
  DELETED
}

enum CEFRLevel {
  A1
  A2
  B1
  B2
  C1
  C2
}

enum ProviderType {
  GOOGLE
  APPLE     # 将来対応
  MICROSOFT # 将来対応
}

enum UserSortBy {
  CREATED_AT
  LAST_ACTIVE_AT
  EMAIL
  DISPLAY_NAME
}

enum SortOrder {
  ASC
  DESC
}
```

### オブジェクト型

```graphql
type User {
  id: UUID!
  email: String!
  displayName: String
  photoUrl: String
  learningGoal: LearningGoal!
  difficultyPreference: CEFRLevel!
  role: UserRole!
  accountStatus: AccountStatus!
  createdAt: DateTime!
  lastActiveAt: DateTime!
}

type UserSummary {
  id: UUID!
  email: String!
  displayName: String
  role: UserRole!
  accountStatus: AccountStatus!
  createdAt: DateTime!
  lastActiveAt: DateTime!
}

union LearningGoal = IeltsGoal | CEFRGoal | NoGoal

type IeltsGoal {
  targetScore: Float! # 4.0 - 9.0
}

type CEFRGoal {
  targetLevel: CEFRLevel!
}

type NoGoal {
  placeholder: Boolean # GraphQL は空の型を許可しないため
}

type AuthResult {
  userId: UUID!
  accessToken: String!
  refreshToken: String!
  expiresIn: Int!
}

type UserConnection {
  nodes: [UserSummary!]!
  totalCount: Int!
  pageInfo: PageInfo!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
}

type UserStats {
  totalUsers: Int!
  activeUsers: Int!
  deletedUsers: Int!
  adminCount: Int!
  userCount: Int!
  usersByGoal: GoalDistribution!
  usersByDifficulty: DifficultyDistribution!
  signupTrend: [SignupData!]!
}

type GoalDistribution {
  ielts: Int!
  cefr: Int!
  noGoal: Int!
}

type DifficultyDistribution {
  a1: Int!
  a2: Int!
  b1: Int!
  b2: Int!
  c1: Int!
  c2: Int!
}

type SignupData {
  date: String! # YYYY-MM-DD
  count: Int!
}
```

### クエリ

```graphql
type Query {
  # 現在のユーザー情報
  me: User! @auth(requires: USER)
  
  # ID でユーザー取得
  user(id: UUID!): User @auth(requires: USER)
  
  # メールでユーザー検索（Admin のみ）
  userByEmail(email: String!): User @auth(requires: ADMIN)
  
  # ユーザー一覧（Admin のみ）
  users(
    first: Int = 20
    offset: Int = 0
    includeDeleted: Boolean = false
    sortBy: UserSortBy = CREATED_AT
    sortOrder: SortOrder = DESC
  ): UserConnection! @auth(requires: ADMIN)
  
  # ユーザー統計（Admin のみ）
  userStats: UserStats! @auth(requires: ADMIN)
  
  # トークンの検証
  verifyToken(token: String!): Boolean!
}
```

### ミューテーション

```graphql
type Mutation {
  # 認証
  signUp(idToken: String!): AuthResult!
  signIn(idToken: String!): AuthResult!
  refreshToken(refreshToken: String!): AuthResult!
  signOut: Boolean! @auth(requires: USER)
  
  # プロファイル管理
  updateProfile(
    input: UpdateProfileInput!
  ): User! @auth(requires: USER)
  
  # 学習目標の更新
  updateLearningGoal(
    input: UpdateLearningGoalInput!
  ): User! @auth(requires: USER)
  
  # ロール変更（Admin のみ）
  changeUserRole(
    userId: UUID!
    newRole: UserRole!
    reason: String
  ): User! @auth(requires: ADMIN)
  
  # アカウント削除
  deleteAccount(
    userId: UUID!
    reason: String
  ): Boolean! @auth(requires: USER)
}

input UpdateProfileInput {
  displayName: String
  photoUrl: String
  difficultyPreference: CEFRLevel
  version: Int! # 楽観的ロック
}

input UpdateLearningGoalInput {
  goal: LearningGoalInput!
  version: Int!
}

input LearningGoalInput {
  type: GoalType!
  ieltsScore: Float    # type が IELTS の場合
  cefrLevel: CEFRLevel # type が CEFR の場合
}

enum GoalType {
  IELTS
  CEFR
  NONE
}
```

## 認証・認可

### ディレクティブ

```graphql
directive @auth(requires: UserRole = USER) on FIELD_DEFINITION
```

### 認証フロー

1. **サインアップ/サインイン**:

   ```graphql
   mutation {
     signUp(idToken: "...") {
       userId
       accessToken
       refreshToken
       expiresIn
     }
   }
   ```

2. **認証済みリクエスト**:

   ```http
   Authorization: Bearer {accessToken}
   ```

3. **トークン更新**:

   ```graphql
   mutation {
     refreshToken(refreshToken: "...") {
       accessToken
       refreshToken
       expiresIn
     }
   }
   ```

### 認可ルール

| 操作 | 必要な権限 |
|-----|-----------|
| 自分の情報取得 | USER |
| 他人の情報取得 | ADMIN |
| プロファイル更新 | USER（本人のみ） |
| ロール変更 | ADMIN |
| アカウント削除 | USER（本人）または ADMIN |
| ユーザー一覧 | ADMIN |
| 統計情報 | ADMIN |

## エラーハンドリング

### エラー型

```graphql
interface Error {
  message: String!
  code: String!
}

type NotFoundError implements Error {
  message: String!
  code: String! # "NOT_FOUND"
  resourceType: String!
  resourceId: String!
}

type ForbiddenError implements Error {
  message: String!
  code: String! # "FORBIDDEN"
  requiredRole: UserRole
}

type ValidationError implements Error {
  message: String!
  code: String! # "VALIDATION_ERROR"
  field: String!
  constraint: String!
}

type ConflictError implements Error {
  message: String!
  code: String! # "CONFLICT"
  currentVersion: Int!
}

type AuthenticationError implements Error {
  message: String!
  code: String! # "AUTHENTICATION_ERROR"
  reason: String!
}
```

### エラーレスポンス例

```json
{
  "errors": [
    {
      "message": "User not found",
      "extensions": {
        "code": "NOT_FOUND",
        "resourceType": "User",
        "resourceId": "123e4567-e89b-12d3-a456-426614174000"
      }
    }
  ]
}
```

## 使用例

### 現在のユーザー情報取得

```graphql
query GetMe {
  me {
    id
    email
    displayName
    photoUrl
    learningGoal {
      ... on IeltsGoal {
        targetScore
      }
      ... on CEFRGoal {
        targetLevel
      }
      ... on NoGoal {
        placeholder
      }
    }
    difficultyPreference
    role
    createdAt
    lastActiveAt
  }
}
```

### プロファイル更新

```graphql
mutation UpdateMyProfile {
  updateProfile(input: {
    displayName: "John Doe"
    difficultyPreference: B2
    version: 1
  }) {
    id
    displayName
    difficultyPreference
  }
}
```

### 学習目標の設定

```graphql
mutation SetIeltsGoal {
  updateLearningGoal(input: {
    goal: {
      type: IELTS
      ieltsScore: 7.0
    }
    version: 2
  }) {
    id
    learningGoal {
      ... on IeltsGoal {
        targetScore
      }
    }
  }
}
```

### ユーザー一覧取得（Admin）

```graphql
query ListAllUsers {
  users(
    first: 50
    offset: 0
    sortBy: LAST_ACTIVE_AT
    sortOrder: DESC
  ) {
    nodes {
      id
      email
      displayName
      role
      accountStatus
      lastActiveAt
    }
    totalCount
    pageInfo {
      hasNextPage
    }
  }
}
```

## レート制限

| エンドポイント | 制限値 | ウィンドウ |
|--------------|--------|-----------|
| signUp/signIn | 10 req | 1時間 |
| 一般クエリ | 100 req | 1分 |
| ミューテーション | 30 req | 1分 |
| Admin クエリ | 無制限 | - |

## サブスクリプション（将来実装）

```graphql
type Subscription {
  # ユーザープロファイルの更新を購読
  userUpdated(userId: UUID!): User! @auth(requires: USER)
  
  # 新規ユーザー登録を購読（Admin のみ）
  userCreated: User! @auth(requires: ADMIN)
}
```

※ リアルタイム更新は優先度が低いため、当面は実装しない
