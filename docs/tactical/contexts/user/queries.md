# User Context - クエリ設計

## 概要

User Context で実行可能なクエリ（読み取り操作）の定義です。必要最小限のクエリを提供し、シンプルさを保ちます。

## クエリ一覧

| クエリ名 | 説明 | 実行権限 | キャッシュ |
|---------|------|----------|-----------|
| GetCurrentUser | 現在のユーザー情報取得 | 認証済み | 30分 |
| GetUserById | ID でユーザー取得 | 本人または Admin | 30分 |
| GetUserByEmail | メールでユーザー検索 | Admin のみ | 30分 |
| ListUsers | ユーザー一覧取得 | Admin のみ | 5分 |
| GetUserStats | ユーザー統計情報 | Admin のみ | 1時間 |

## クエリ詳細

### 1. GetCurrentUser

現在認証されているユーザーの情報を取得。

**クエリ構造**: パラメータなし（トークンから user_id を取得）

**レスポンスフィールド**:

- user_id, email, display_name, photo_url
- learning_goal, difficulty_preference
- role, created_at, last_active_at

**処理フロー**:

1. トークンから user_id 取得
2. キャッシュチェック
3. データベースから取得
4. キャッシュ更新
5. レスポンス返却

**キャッシュ戦略**:

- TTL: 30分
- キー: `user:current:{user_id}`
- 更新時は即座に無効化

### 2. GetUserById

指定された ID のユーザー情報を取得。

**クエリパラメータ**: user_id

**レスポンスフィールド**:

- CurrentUserResult に加えて account_status を含む

**権限チェック**:

- 本人: 自分の情報のみ取得可能
- Admin: 全ユーザーの情報取得可能

**エラーケース**:

- `NotFound`: ユーザーが存在しない
- `Forbidden`: 他人の情報へのアクセス

### 3. GetUserByEmail

メールアドレスでユーザーを検索（Admin 専用）。

**クエリパラメータ**: email

**レスポンス**: UserResult または null

**処理フロー**:

1. Admin 権限チェック
2. メールアドレスで検索
3. 削除済みユーザーは除外
4. 結果返却

**インデックス**:

- `idx_users_email`: email カラムにユニークインデックス

### 4. ListUsers

ユーザー一覧を取得（Admin 専用、ページネーション対応）。

**クエリ構造**:

```rust
pub struct ListUsers {
    pub limit: u32,           // デフォルト: 20, 最大: 100
    pub offset: u32,          // デフォルト: 0
    pub include_deleted: bool, // 削除済みを含むか
    pub sort_by: UserSortBy,  // ソート項目
    pub sort_order: SortOrder, // 昇順/降順
}

pub enum UserSortBy {
    CreatedAt,
    LastActiveAt,
    Email,
    DisplayName,
}

pub enum SortOrder {
    Asc,
    Desc,
}
```

**レスポンス**:

```rust
pub struct UserListResult {
    pub users: Vec<UserSummary>,
    pub total_count: u64,
    pub has_next: bool,
}

pub struct UserSummary {
    pub user_id: UserId,
    pub email: String,
    pub display_name: Option<String>,
    pub role: UserRole,
    pub account_status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}
```

**SQL 例**:

```sql
SELECT 
    user_id, email, display_name, role, 
    account_status, created_at, last_active_at
FROM users
WHERE 
    CASE WHEN $1 THEN true ELSE deleted_at IS NULL END
ORDER BY 
    CASE WHEN $2 = 'created_at' THEN created_at END DESC,
    CASE WHEN $2 = 'last_active_at' THEN last_active_at END DESC
LIMIT $3 OFFSET $4;
```

### 5. GetUserStats

ユーザー統計情報を取得（Admin 専用）。

**クエリ構造**:

```rust
pub struct GetUserStats {
    // パラメータなし
}
```

**レスポンス**:

```rust
pub struct UserStatsResult {
    pub total_users: u64,
    pub active_users: u64,      // 30日以内にログイン
    pub deleted_users: u64,
    pub admin_count: u64,
    pub user_count: u64,
    pub users_by_goal: GoalDistribution,
    pub users_by_difficulty: DifficultyDistribution,
    pub signup_trend: Vec<SignupData>,
}

pub struct GoalDistribution {
    pub ielts: u64,
    pub cefr: u64,
    pub no_goal: u64,
}

pub struct DifficultyDistribution {
    pub a1: u64,
    pub a2: u64,
    pub b1: u64,
    pub b2: u64,
    pub c1: u64,
    pub c2: u64,
}

pub struct SignupData {
    pub date: Date,
    pub count: u64,
}
```

**キャッシュ戦略**:

- TTL: 1時間
- キー: `user:stats`
- 定期的に再計算

## クエリハンドラーの実装例

```rust
#[async_trait]
impl QueryHandler<GetCurrentUser> for UserQueryService {
    type Result = CurrentUserResult;
    
    async fn handle(&self, _query: GetCurrentUser) -> Result<Self::Result> {
        // 1. コンテキストから user_id 取得
        let user_id = self.context.current_user_id()?;
        
        // 2. キャッシュチェック
        let cache_key = format!("user:current:{}", user_id);
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(cached);
        }
        
        // 3. データベースから取得
        let user = self.repository
            .find_by_id(&user_id)
            .await?
            .ok_or(Error::NotFound)?;
        
        // 4. レスポンス構築
        let result = CurrentUserResult {
            user_id: user.user_id,
            email: user.email,
            display_name: user.display_name,
            photo_url: user.photo_url,
            learning_goal: user.learning_goal,
            difficulty_preference: user.difficulty_preference,
            role: user.role,
            created_at: user.created_at,
            last_active_at: user.last_active_at,
        };
        
        // 5. キャッシュ更新
        self.cache.set(&cache_key, &result, Duration::minutes(30)).await?;
        
        Ok(result)
    }
}
```

## パフォーマンス最適化

### インデックス設計

```sql
-- 主キー
CREATE UNIQUE INDEX idx_users_pk ON users(user_id);

-- メールアドレス検索用
CREATE UNIQUE INDEX idx_users_email ON users(email);

-- プロバイダーID検索用
CREATE INDEX idx_users_provider ON users(provider_user_id);

-- リスト表示用
CREATE INDEX idx_users_created ON users(created_at DESC);
CREATE INDEX idx_users_active ON users(last_active_at DESC);

-- 削除フラグ
CREATE INDEX idx_users_deleted ON users(deleted_at) WHERE deleted_at IS NOT NULL;
```

### キャッシュ戦略

| データ種別 | TTL | 更新タイミング |
|-----------|-----|---------------|
| 個別ユーザー | 30分 | プロファイル更新時 |
| ユーザーリスト | 5分 | 新規登録/削除時 |
| 統計情報 | 1時間 | 定期バッチ |

### N+1 問題の回避

```rust
// 悪い例
for user_id in user_ids {
    let user = repository.find_by_id(&user_id).await?;
    // ...
}

// 良い例
let users = repository.find_by_ids(&user_ids).await?;
```

## エラーハンドリング

```rust
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Resource not found")]
    NotFound,
    
    #[error("Access forbidden")]
    Forbidden,
    
    #[error("Invalid query parameters")]
    InvalidParameters(String),
    
    #[error("Cache error")]
    CacheError(#[from] CacheError),
    
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}
```
