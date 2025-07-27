# User Context - EventStorming Design Level

## 概要

User Context は、Effect プロジェクトにおけるユーザー認証、プロファイル管理、学習設定を担当するコンテキストです。
Firebase Authentication を活用し、Google OAuth によるシンプルで安全な認証フローを提供します。

### 主要な責務

- **認証管理**: Firebase Auth による Google OAuth 認証
- **プロファイル管理**: ユーザー情報、学習目標の管理
- **権限管理**: Admin/User のロールベース権限制御
- **セキュリティ**: アクセストークンの管理、権限制御

### 設計方針

- Firebase Auth への依存を最小限に抑える（Anti-Corruption Layer）
- ユーザー集約はシンプルに保つ（認証は Firebase に委譲）
- プライバシーとセキュリティを最優先
- 将来の認証プロバイダー追加に備えた抽象化

## 集約の設計

### 1. UserProfile（ユーザープロファイル）- 集約ルート

ユーザーの基本情報と学習設定を管理します。

````rust
pub struct UserProfile {
    // 識別子
    user_id: UserId,

    // 基本情報（Google アカウントから取得）
    email: Email,
    display_name: String,
    photo_url: Option<Url>,

    // 学習設定（シンプルに）
    learning_goal: LearningGoal,
    difficulty_preference: CefrLevel,  // 難易度の好み（A1-C2）

    // 権限
    role: UserRole,

    // アカウント状態
    account_status: AccountStatus,
    created_at: DateTime<Utc>,
    last_active_at: DateTime<Utc>,

    // メタデータ
    version: u64,  // 楽観的ロック
}

pub enum AccountStatus {
    Active,
    Deleted { deleted_at: DateTime<Utc> },
}

pub enum UserRole {
    Admin,  // 全ユーザーのデータ閲覧、システム設定変更
    User,   // 通常のユーザー（自分のデータのみ）
}

### 2. LearningGoal（学習目標）- 値オブジェクト

ユーザーの学習目標を表現します。シンプルに保ちます。

```rust
pub enum LearningGoal {
    IeltsScore { overall: f32 },      // 例: 7.0
    GeneralLevel { cefr: CefrLevel },  // 例: B2
    NoSpecificGoal,                    // 特に目標なし
}
````

## 認証管理

### Firebase Auth との統合

```rust
// Anti-Corruption Layer として Firebase Auth を抽象化
pub trait AuthenticationProvider: Send + Sync {
    async fn verify_token(&self, token: &str) -> Result<AuthenticatedUser, AuthError>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, AuthError>;
    async fn revoke_token(&self, token: &str) -> Result<(), AuthError>;
}

pub struct AuthenticatedUser {
    uid: String,           // Firebase UID
    email: String,
    email_verified: bool,
    provider_id: String,   // "google.com"
    display_name: Option<String>,
    photo_url: Option<String>,
    claims: HashMap<String, Value>,
}

pub struct TokenPair {
    access_token: String,
    refresh_token: String,
    expires_at: DateTime<Utc>,
}

// Firebase Auth 実装
pub struct FirebaseAuthProvider {
    admin_sdk: FirebaseAdmin,
    project_id: String,
}

impl AuthenticationProvider for FirebaseAuthProvider {
    async fn verify_token(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        let decoded = self.admin_sdk.verify_id_token(token).await?;

        // Firebase の情報を内部モデルに変換
        Ok(AuthenticatedUser {
            uid: decoded.uid,
            email: decoded.email,
            email_verified: decoded.email_verified,
            provider_id: decoded.firebase.sign_in_provider,
            display_name: decoded.display_name,
            photo_url: decoded.picture,
            claims: decoded.custom_claims,
        })
    }
}
```

## コマンドとイベント

### コマンド（青い付箋 🟦）

```rust
pub enum UserCommand {
    // 認証関連
    SignUp {
        auth_user: AuthenticatedUser,
    },

    SignIn {
        auth_user: AuthenticatedUser,
    },

    SignOut {
        user_id: UserId,
    },

    RefreshSession {
        user_id: UserId,
        refresh_token: String,
    },

    // プロファイル更新
    UpdateProfile {
        user_id: UserId,
        display_name: Option<String>,
        photo_url: Option<String>,
    },

    UpdateLearningGoal {
        user_id: UserId,
        goal: LearningGoal,
    },

    UpdateDifficultyPreference {
        user_id: UserId,
        level: CefrLevel,
    },

    // アカウント管理
    DeleteAccount {
        user_id: UserId,
    },

    // 権限管理（Admin のみ）
    UpdateUserRole {
        admin_id: UserId,
        target_user_id: UserId,
        new_role: UserRole,
    },
}
```

### ドメインイベント（オレンジの付箋 🟠）

```rust
pub enum UserDomainEvent {
    // 認証イベント
    UserSignedUp {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        email: String,
        provider: String,
        role: UserRole,
    },

    UserSignedIn {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        ip_address: Option<IpAddress>,
        user_agent: Option<String>,
    },

    UserSignedOut {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
    },

    SessionRefreshed {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        new_expiry: DateTime<Utc>,
    },

    // プロファイル更新イベント
    ProfileUpdated {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        changes: ProfileChanges,
    },

    LearningGoalSet {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        goal: LearningGoal,
        previous_goal: Option<LearningGoal>,
    },

    DifficultyPreferenceUpdated {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        level: CefrLevel,
    },

    // アカウント状態イベント
    AccountDeleted {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        email: String,  // 削除後の参照用に保持
    },

    // 権限変更イベント
    UserRoleUpdated {
        event_id: EventId,
        occurred_at: DateTime<Utc>,
        user_id: UserId,
        old_role: UserRole,
        new_role: UserRole,
        updated_by: UserId,
    },
}

pub struct ProfileChanges {
    display_name: Option<String>,
    photo_url: Option<String>,
}
```

## ビジネスポリシー（紫の付箋 🟪）

### 新規登録ポリシー

```rust
// Google OAuth 認証済みユーザーの自動登録
when SignUpCommand with auth_user.email_verified == true {
    create UserProfile {
        email: auth_user.email,
        display_name: auth_user.display_name ?? auth_user.email.split('@')[0],
        photo_url: auth_user.photo_url,
        learning_goal: LearningGoal::NoSpecificGoal,
        difficulty_preference: CefrLevel::B1,  // デフォルトは中級
        role: UserRole::User,  // デフォルトは一般ユーザー
        account_status: Active,
    }
    emit UserSignedUpEvent
}

// 最初の Admin ユーザーの判定
when SignUpCommand && no_admin_exists() {
    assign_role = UserRole::Admin
}
```

### 権限管理ポリシー

```rust
// Admin 権限の確認
when UpdateUserRoleCommand {
    if executor.role != Admin {
        reject_with_error("Insufficient permissions")
    }

    if target_user == executor {
        reject_with_error("Cannot change own role")
    }

    update_role()
    emit UserRoleUpdatedEvent
}

// データアクセス制御
when accessing_user_data {
    if accessor.role == Admin {
        allow_access_to_all_users()
    } else {
        allow_access_only_to_own_data()
    }
}
```

### アカウント削除ポリシー

```rust
// アカウント削除処理
when DeleteAccountCommand {
    // 削除処理
    mark_account_as_deleted()
    schedule_data_purge_immediately()  // 即座にデータ削除
    emit AccountDeletedEvent
}

// カスケード削除
when AccountDeletedEvent {
    trigger_cascade_deletion_in_all_contexts()
}
```

## リードモデル（緑の付箋 🟩）

### UserSessionView（セッション情報）

```rust
pub struct UserSessionView {
    user_id: UserId,
    email: String,
    display_name: String,
    photo_url: Option<String>,

    // セッション情報
    session_expires_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,

    // 権限情報
    role: UserRole,

    // 簡易プロファイル
    learning_goal: String,  // "IELTS 7.0" or "B2 Level" or "No specific goal"
    difficulty_preference: String,  // "A1" - "C2"
}
```

### UserSettingsView（設定画面用）

```rust
pub struct UserSettingsView {
    user_id: UserId,

    // プロファイル
    email: String,
    display_name: String,
    photo_url: Option<String>,

    // 学習設定
    learning_goal: String,
    difficulty_preference: String,

    // アカウント情報
    account_created: String,  // "2024-01-20"
    last_active: String,      // "2 hours ago"
    total_study_days: u32,
}
```

### UserListView（管理画面用）

```rust
pub struct UserListView {
    user_id: UserId,
    email: String,
    display_name: String,
    role: String,  // "Admin" or "User"

    // 統計情報
    total_items_learned: u32,
    last_active: DateTime<Utc>,
    account_status: String,

    // 目標情報
    learning_goal: String,
}
```

## セキュリティ考慮事項

### トークン管理

```rust
pub struct TokenManager {
    auth_provider: Box<dyn AuthenticationProvider>,
}

impl TokenManager {
    pub async fn verify_token(&self, token: &str) -> Result<AuthenticatedUser> {
        // Firebase での検証
        self.auth_provider.verify_token(token).await
    }
}
```

## 他コンテキストとの連携

### 認証情報の提供

```rust
// User Context が提供するサービス
pub trait AuthenticationService: Send + Sync {
    async fn verify_user(&self, token: &str) -> Result<UserId, AuthError>;
    async fn get_user_role(&self, user_id: UserId) -> Result<UserRole, AuthError>;
}

// 他のコンテキストでの利用
impl LearningContext {
    async fn start_session(&self, token: &str, config: SessionConfig) -> Result<SessionId> {
        // User Context で認証
        let user_id = self.auth_service.verify_user(token).await?;

        // セッション開始
        self.create_session(user_id, config).await
    }
}
```

### イベント連携

```rust
// User → Progress Context
when UserSignedUpEvent {
    initialize_user_progress()
    create_default_statistics()
}

// User → Learning Algorithm Context
when DifficultyPreferenceUpdatedEvent {
    adjust_item_selection_difficulty()
}

// User → 全コンテキスト
when AccountDeletedEvent {
    trigger_cascade_deletion()
    anonymize_all_data()
}
```

## 実装の考慮事項

### パフォーマンス最適化

```rust
// ユーザー情報のキャッシュ
impl UserContext {
    pub async fn get_user_profile(&self, user_id: UserId) -> Result<UserProfile> {
        // データベースから取得
        self.repository.find_by_id(user_id).await
    }

    pub async fn verify_and_get_user(&self, token: &str) -> Result<(UserId, UserRole)> {
        // Firebase 検証
        let auth_user = self.auth_provider.verify_token(token).await?;
        let user_id = UserId::from(auth_user.uid);

        // プロファイルからロール取得
        let profile = self.get_user_profile(user_id).await?;

        Ok((user_id, profile.role))
    }
}
```

### エラーハンドリング

```rust
pub enum UserContextError {
    // 認証エラー
    AuthenticationFailed { reason: String },
    TokenExpired { expired_at: DateTime<Utc> },
    InvalidToken { details: String },

    // プロファイルエラー
    ProfileNotFound { user_id: UserId },
    ProfileUpdateFailed { reason: String },

    // アカウントエラー
    AccountDeleted,

    // バリデーションエラー
    InvalidEmail { email: String },
    InvalidGoal { reason: String },
}
```

## 更新履歴

- 2025-07-27: 初版作成（Firebase Auth 統合、シンプルなプロファイル管理、Admin/User 権限）
