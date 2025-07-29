# User Context - リポジトリインターフェース

## 概要

User Context には 1 つの主要な集約が存在します：

- `UserProfile`：ユーザー情報と設定の管理

このコンテキストは他の全てのコンテキストから参照される中心的な存在であり、
基本的なユーザー認証（OAuth メイン）、プロフィール管理、設定管理を担当します。MVP として最小限の機能に絞った設計です。

## UserProfileRepository

ユーザープロフィールの永続化を担当するリポジトリです。

### インターフェース定義

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// ユーザープロフィールのリポジトリ
#[async_trait]
pub trait UserProfileRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    // ===== 基本的な CRUD 操作 =====

    /// ID でユーザーを取得
    async fn find_by_id(&self, id: &UserId) -> Result<Option<UserProfile>, Self::Error>;

    /// メールアドレスでユーザーを取得
    async fn find_by_email(&self, email: &str) -> Result<Option<UserProfile>, Self::Error>;

    /// OAuth プロバイダー ID でユーザーを取得
    async fn find_by_oauth_provider(
        &self,
        provider: &OAuthProvider,
        provider_id: &str,
    ) -> Result<Option<UserProfile>, Self::Error>;

    /// ユーザーを保存（新規作成または更新）
    async fn save(&self, user: &UserProfile) -> Result<(), Self::Error>;

    /// ユーザーを削除（論理削除のみ実装）
    async fn delete(&self, id: &UserId) -> Result<(), Self::Error>;

    // ===== 認証関連 =====

    /// ログイン認証用の情報を取得
    async fn find_for_authentication(
        &self,
        email: &str,
    ) -> Result<Option<AuthenticationInfo>, Self::Error>;

    // ===== 設定管理 =====

    /// ユーザー設定を取得
    async fn get_user_settings(
        &self,
        user_id: &UserId,
    ) -> Result<Option<UserSettings>, Self::Error>;

    /// ユーザー設定を保存
    async fn save_user_settings(
        &self,
        settings: &UserSettings,
    ) -> Result<(), Self::Error>;
}
```

### 使用例

```rust
// アプリケーションサービスでの使用例
pub struct RegisterUserUseCase<R: UserProfileRepository> {
    repository: Arc<R>,
    event_bus: Arc<dyn EventBus>,
    password_hasher: Arc<dyn PasswordHasher>,
}

impl<R: UserProfileRepository> RegisterUserUseCase<R> {
    pub async fn execute(&self, command: RegisterUserCommand) -> Result<UserId> {
        // メールアドレスの重複チェック
        if let Some(_) = self.repository.find_by_email(&command.email).await? {
            return Err(DomainError::EmailAlreadyExists);
        }

        // パスワードのハッシュ化
        let password_hash = self.password_hasher.hash(&command.password)?;

        // ユーザー作成
        let user = UserProfile::new(
            command.email,
            password_hash,
            command.display_name,
        )?;
        let user_id = user.id().clone();

        // 保存
        self.repository.save(&user).await?;

        // イベント発行
        self.event_bus.publish(UserRegistered {
            user_id: user_id.clone(),
            email: user.email().to_string(),
            registered_at: Utc::now(),
        }).await?;

        Ok(user_id)
    }
}
```

### 補助的な型定義

```rust
/// ユーザーロール
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

/// OAuth プロバイダー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthProvider {
    Google,
    GitHub,
    Apple,
}

/// 認証情報
#[derive(Debug, Clone)]
pub struct AuthenticationInfo {
    pub user_id: UserId,
    pub email: String,
    pub password_hash: String,
    pub is_email_verified: bool,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
}

/// ユーザー設定
#[derive(Debug, Clone)]
pub struct UserSettings {
    pub user_id: UserId,
    pub language: LanguageCode,
    pub timezone: String,
    pub theme: ThemePreference,
    pub updated_at: DateTime<Utc>,
}
```

## 実装上の考慮事項

### 1. パフォーマンス最適化

```rust
// インデックスの推奨
// UserProfile
// - (email) - UNIQUE, メールアドレス検索
// - (oauth_provider, oauth_provider_id) - UNIQUE, OAuth ログイン用
// - (role) - ロール別フィルタリング（将来的な管理機能用）
// - (created_at) - ソート用
// - (is_deleted, created_at) - 論理削除とソート

// UserSettings
// - (user_id) - PRIMARY KEY, ユーザーと 1:1
```

### 2. トランザクション境界

```rust
// ユーザー削除のカスケード処理例
pub async fn delete_user_cascade(
    user_repo: &dyn UserProfileRepository,
    event_bus: &dyn EventBus,
    user_id: UserId,
) -> Result<()> {
    // 1. ユーザーを論理削除（トランザクション 1）
    let mut user = user_repo.find_by_id(&user_id).await?
        .ok_or(DomainError::NotFound)?;

    user.mark_as_deleted();
    user_repo.save(&user).await?;

    // 2. イベントを発行して他コンテキストに通知
    event_bus.publish(UserDeleted {
        user_id: user_id.clone(),
        deleted_at: Utc::now(),
    }).await?;

    // 各コンテキストはイベントを受けて各自でデータ削除
    // - Learning Context: UserItemRecord を削除
    // - Learning Algorithm: ItemLearningRecord を削除
    // - Progress Context: 統計を匿名化
    // - AI Integration: ChatSession を削除

    Ok(())
}
```

### 3. エラーハンドリング

```rust
/// User Context 固有のリポジトリエラー
#[derive(Debug, thiserror::Error)]
pub enum UserRepositoryError {
    #[error("User not found: {0}")]
    UserNotFound(UserId),

    #[error("Email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("Invalid OAuth provider: {0}")]
    InvalidOAuthProvider(String),

    #[error("Database error: {0}")]
    Database(String),
}
```

### 4. セキュリティ考慮事項

```rust
/// パスワードハッシュインターフェース
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, HashError>;
    fn verify(&self, password: &str, hash: &str) -> Result<bool, HashError>;
}

/// トークン生成インターフェース
#[async_trait]
pub trait TokenGenerator: Send + Sync {
    fn generate_session_token(&self) -> String;
}

/// セキュリティポリシー（OAuth 認証がメインなので最小限）
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub min_password_length: usize,
    pub session_timeout: chrono::Duration,
}
```

## 他コンテキストとの連携

```rust
// ユーザー削除イベントのハンドラー例（Learning Context）
pub struct UserDeletionHandler<R: UserItemRecordRepository> {
    repository: Arc<R>,
}

#[async_trait]
impl<R: UserItemRecordRepository> EventHandler for UserDeletionHandler<R> {
    async fn handle(&self, event: DomainEvent) -> Result<()> {
        match event {
            DomainEvent::UserDeleted { user_id, .. } => {
                // ユーザーの全学習記録を削除
                let deleted_count = self.repository
                    .delete_all_by_user(&user_id)
                    .await?;

                log::info!(
                    "Deleted {} learning records for user {}",
                    deleted_count,
                    user_id
                );
            }
            _ => {}
        }
        Ok(())
    }
}
```

## 更新履歴

- 2025-07-28: 初版作成（中心的なコンテキストとしての設計）
- 2025-07-29: MVP に向けて機能を簡潔化（トークン認証、ユーザー管理、ストリーク、統計機能を削除）
