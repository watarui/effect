# ユーザー管理機能仕様

## 概要

Effect のユーザー管理機能は、認証・認可、プロフィール管理、学習設定を提供します。独立したマイクロサービスとして実装され、他のサービスと連携して完全な学習体験を実現します。

## 機能一覧

### 1. 認証・認可

#### 認証方式

```rust
pub enum AuthMethod {
    EmailPassword,
    OAuth {
        provider: OAuthProvider,
    },
}

pub enum OAuthProvider {
    Google,
    GitHub,
    Apple,
}
```

#### JWT トークン管理

```rust
pub struct JwtClaims {
    pub sub: Uuid,  // user_id
    pub email: String,
    pub exp: i64,   // 有効期限
    pub iat: i64,   // 発行日時
}

pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u32,
}
```

### 2. ユーザー管理

#### User エンティティ

```rust
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub auth_method: AuthMethod,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}
```

#### UserProfile エンティティ

```rust
pub struct UserProfile {
    pub user_id: Uuid,
    pub bio: Option<String>,
    pub timezone: String,
    pub language: LanguageCode,
    pub country: Option<CountryCode>,
    pub learning_goals: Vec<LearningGoal>,
}

pub struct LearningGoal {
    pub test_type: TestCategory,
    pub target_score: Option<u32>,
    pub target_date: Option<Date>,
}
```

### 3. ユーザー設定

#### UserSettings エンティティ

```rust
pub struct UserSettings {
    pub user_id: Uuid,
    pub daily_goal: u32,  // 1日の目標単語数
    pub notification_enabled: bool,
    pub notification_time: Option<Time>,
    pub preferred_categories: Vec<TestCategory>,
    pub learning_mode_preference: LearningModePreference,
    pub sound_enabled: bool,
    pub dark_mode: bool,
}

pub struct LearningModePreference {
    pub default_mode: LearningMode,
    pub session_length: u32,  // 分単位
    pub difficulty_auto_adjust: bool,
}
```

### 4. 貢献度管理（将来対応）

#### ContributionScore エンティティ

```rust
pub struct ContributionScore {
    pub user_id: Uuid,
    pub total_edits: u32,
    pub quality_score: f32,  // 0.0-1.0
    pub helpful_edits: u32,
    pub last_contribution: DateTime<Utc>,
}

pub enum ContributionType {
    WordAdded,
    MeaningAdded,
    ExampleAdded,
    AudioRecorded,
    ErrorCorrected,
}
```

## API エンドポイント

### GraphQL スキーマ

```graphql
type User {
    id: ID!
    email: String!
    displayName: String!
    avatarUrl: String
    profile: UserProfile!
    settings: UserSettings!
    contributionScore: ContributionScore
}

type UserProfile {
    bio: String
    timezone: String!
    language: String!
    country: String
    learningGoals: [LearningGoal!]!
}

type Mutation {
    register(input: RegisterInput!): AuthResult!
    login(input: LoginInput!): AuthResult!
    refreshToken(refreshToken: String!): TokenPair!
    updateProfile(input: UpdateProfileInput!): UserProfile!
    updateSettings(input: UpdateSettingsInput!): UserSettings!
}

type Query {
    me: User!
    user(id: ID!): User
    searchUsers(query: String!): [User!]!
}
```

## サービス間連携

### 1. 認証フロー

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│ API Gateway │────▶│User Service │
└─────────────┘     └──────┬──────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ JWT 検証    │
                    │ ユーザー情報 │
                    └─────────────┘
```

### 2. ユーザー情報の共有

```rust
// gRPC インターフェース
service UserService {
    rpc GetUser(GetUserRequest) returns (UserResponse);
    rpc GetUsers(GetUsersRequest) returns (UsersResponse);
    rpc ValidateToken(ValidateTokenRequest) returns (ValidationResponse);
}

message GetUserRequest {
    string user_id = 1;
}

message UserResponse {
    string id = 1;
    string email = 2;
    string display_name = 3;
    string avatar_url = 4;
}
```

### 3. イベント連携

```rust
// ユーザー関連イベントの発行
pub enum UserEvent {
    UserRegistered {
        user_id: Uuid,
        email: String,
        registered_at: DateTime<Utc>,
    },
    UserProfileUpdated {
        user_id: Uuid,
        changes: HashMap<String, Value>,
        updated_at: DateTime<Utc>,
    },
    UserDeleted {
        user_id: Uuid,
        deleted_at: DateTime<Utc>,
    },
}
```

## セキュリティ

### 1. パスワード管理

- bcrypt によるハッシュ化
- パスワード強度の検証
- パスワードリセット機能

### 2. セッション管理

- Refresh Token のローテーション
- 異常なアクセスパターンの検出
- 複数デバイスのセッション管理

### 3. プライバシー保護

- 最小権限の原則
- 個人情報の暗号化
- アクセスログの記録

## データベース設計

### Users テーブル

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    display_name VARCHAR(100) NOT NULL,
    avatar_url TEXT,
    auth_method VARCHAR(50) NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_login_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_users_email ON users(email);
```

### User Settings テーブル

```sql
CREATE TABLE user_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    daily_goal INTEGER DEFAULT 10,
    notification_enabled BOOLEAN DEFAULT TRUE,
    notification_time TIME,
    preferred_categories TEXT[],
    default_learning_mode VARCHAR(50),
    session_length INTEGER DEFAULT 15,
    sound_enabled BOOLEAN DEFAULT TRUE,
    dark_mode BOOLEAN DEFAULT FALSE,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);
```

## 実装の優先順位

### Phase 1（MVP）

- Email/Password 認証
- 基本的なプロフィール管理
- JWT によるセッション管理
- 学習設定の基本機能

### Phase 2

- OAuth 認証（Google）
- アバター画像のアップロード
- 通知設定の詳細化
- タイムゾーン対応

### Phase 3

- 貢献度スコアリング
- ソーシャル機能（フォロー）
- アクティビティフィード
- 高度なプライバシー設定
