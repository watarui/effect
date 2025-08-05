# Learning Context - コマンド定義

## 概要

Learning Context で使用されるコマンドの定義です。コマンドは学習セッションの管理と学習フローの制御を行います。

## コマンド一覧

| コマンド名 | 説明 | 起動元 |
|-----------|------|--------|
| StartSession | 学習セッションを開始する | ユーザー操作 |
| PresentItem | 次の項目を提示する | システム自動 |
| RevealAnswer | 解答を表示する | ユーザー操作/タイマー |
| JudgeCorrectness | 正誤を判定する | ユーザー操作/タイマー |
| CompleteSession | セッションを完了する | ユーザー操作/システム |
| AbandonSession | セッションを中断する | ユーザー操作 |

## コマンド詳細

### StartSession

新しい学習セッションを開始します。

**パラメータ**:

```rust
struct StartSessionCommand {
    user_id: UserId,
    item_count: u8,        // 1-100
    session_type: SessionType,
    item_selection_strategy: ItemSelectionStrategy,
}

enum SessionType {
    Review,          // 復習のみ
    NewItems,        // 新規項目のみ
    Mixed,           // 混合
}

enum ItemSelectionStrategy {
    SmartSelection,     // AI による最適化選択
    DueForReview,      // 復習期限優先
    WeakItems,         // 苦手項目優先
    Random,            // ランダム
}
```

**検証ルール**:

- item_count は 1-100 の範囲内
- ユーザーにアクティブなセッションがないこと
- 選択可能な項目が十分にあること

**発生イベント**:

- SessionStarted

### PresentItem

次の学習項目を提示します。

**パラメータ**:

```rust
struct PresentItemCommand {
    session_id: SessionId,
    item_id: ItemId,
    presentation_order: u8,
}
```

**検証ルール**:

- セッションが InProgress 状態であること
- 前の項目の処理が完了していること
- 項目がセッションの項目リストに含まれていること

**発生イベント**:

- ItemPresented

### RevealAnswer

解答を表示します。ユーザー操作または3秒タイマーによって発動されます。

**パラメータ**:

```rust
struct RevealAnswerCommand {
    session_id: SessionId,
    item_id: ItemId,
    trigger: AnswerRevealTrigger,
}

enum AnswerRevealTrigger {
    UserRequested,      // ユーザーが要求
    TimeLimit,          // 3秒経過
}
```

**検証ルール**:

- 項目が現在提示中であること
- まだ解答が表示されていないこと

**発生イベント**:

- AnswerRevealed

### JudgeCorrectness

項目の正誤を判定します。

**パラメータ**:

```rust
struct JudgeCorrectnessCommand {
    session_id: SessionId,
    item_id: ItemId,
    judgment: CorrectnessJudgment,
    response_time_ms: u32,
}

enum CorrectnessJudgment {
    AutoConfirmed,           // 3秒経過で自動正解
    UserConfirmedCorrect,    // ユーザーが「わかった」
    UserConfirmedIncorrect,  // ユーザーが「わからなかった」
}
```

**検証ルール**:

- 解答が表示済みであること
- まだ判定されていないこと
- response_time_ms が妥当な範囲内（0-10000ms）

**発生イベント**:

- CorrectnessJudged
- ItemMasteryUpdated（習熟度が変化した場合）

### CompleteSession

学習セッションを完了します。

**パラメータ**:

```rust
struct CompleteSessionCommand {
    session_id: SessionId,
    completion_reason: CompletionReason,
}

enum CompletionReason {
    AllItemsCompleted,    // すべての項目を完了
    UserRequested,        // ユーザーが終了を選択
    TimeOut,              // タイムアウト
}
```

**検証ルール**:

- セッションが存在すること
- まだ完了していないこと

**発生イベント**:

- SessionCompleted

### AbandonSession

学習セッションを中断します（進捗は保存されません）。

**パラメータ**:

```rust
struct AbandonSessionCommand {
    session_id: SessionId,
    reason: String,
}
```

**検証ルール**:

- セッションが InProgress 状態であること

**発生イベント**:

- SessionAbandoned

## コマンドハンドラーの実装

### 基本構造

```rust
trait CommandHandler<C> {
    type Error;
    
    async fn handle(&self, command: C) -> Result<Vec<DomainEvent>, Self::Error>;
}

// 例: StartSessionHandler
impl CommandHandler<StartSessionCommand> for StartSessionHandler {
    type Error = LearningError;
    
    async fn handle(&self, command: StartSessionCommand) -> Result<Vec<DomainEvent>, Self::Error> {
        // 1. 項目選定をAlgorithm Contextに依頼
        let items = self.algorithm_service
            .select_items(&command.user_id, command.item_count)
            .await?;
        
        // 2. セッション集約を作成
        let session = LearningSession::start(
            command.user_id,
            items,
            command.session_type,
        )?;
        
        // 3. Event Storeに保存
        self.event_store.save(&session).await?;
        
        // 4. ドメインイベントを返す
        Ok(session.uncommitted_events())
    }
}
```

### エラーハンドリング

```rust
enum LearningError {
    SessionNotFound,
    SessionAlreadyActive,
    InvalidItemCount,
    InsufficientItems,
    InvalidState,
    ExternalServiceError(String),
}
```

## 外部サービスとの連携

### Algorithm Context

- `StartSession` 時に項目選定を依頼
- 同期的な gRPC 呼び出し

### Vocabulary Context

- 項目の詳細情報取得
- キャッシュを活用して高速化

### Progress Context

- セッション完了時にイベント発行
- 非同期（Pub/Sub 経由）

## トランザクション境界

各コマンドハンドラーは単一のトランザクション内で実行され、以下を保証します：

1. 集約の状態変更
2. イベントの永続化
3. Event Bus へのイベント発行

失敗時はすべてロールバックされます。
