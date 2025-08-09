# Learning Context - 集約設計

## 概要

Learning Context は、学習セッションの実行管理に特化したシンプルな設計を採用しています。セッション管理、ハイブリッドUIの実装、正誤判定を担当します。

### 主要な責務

- 学習セッションの一時的な管理（1-100問）
- ハイブリッドUI（3秒ルール）の実装
- 正誤判定と即座のフィードバック
- Progress Context へのイベント通知

## 集約の設計

### 1. LearningSession（学習セッション）- 集約ルート

学習セッション全体を管理する集約です。1 回のテストセッション（1-100問、ユーザー設定）の状態を保持します。

**主要な属性**:

- session_id: セッション識別子
- user_id: ユーザー識別子
- started_at: 開始日時
- items: セッション項目のリスト
- status: セッションステータス（NotStarted, InProgress, Completed）
- session_type: セッションタイプ（Review, NewItems, Mixed）

**不変条件**:

- 問題数は 1-100 の範囲内
- 開始後はセッションタイプの変更不可
- 完了後の項目追加・変更は不可

### 2. SessionItem（セッション項目）- 値オブジェクト

セッション内の個々の問題を表現します。

**主要な属性**:

- item_id: 項目識別子
- presented_at: 提示日時
- answer_revealed_at: 解答表示日時
- response_time_ms: 反応時間（ミリ秒）
- answer_reveal_trigger: 解答表示トリガー（UserRequested, TimeLimit）
- correctness_judgment: 正誤判定（AutoConfirmed, UserConfirmedCorrect, UserConfirmedIncorrect）

**設計のポイント**:

- 3秒ルール：3秒経過で自動的に「正解」扱い
- ユーザーは任意のタイミングで解答を表示可能
- 解答表示後に正誤を自己申告

### 3. UserItemRecord（ユーザー項目記録）- 最小限の記録

最新の学習状態のみを保持します。詳細な履歴は Progress Context が管理します。

**主要な属性**:

- user_id: ユーザー識別子
- item_id: 項目識別子
- last_seen: 最終学習日時
- correct_count: 正解数
- total_count: 総回答数

**設計のポイント**:

- 最小限のデータのみ保持
- 習熟度の詳細は Progress Context で管理
- セッション完了時に更新

## シンプルな実装

### セッション管理

Redis を使用した一時的なセッション管理を行います。

```rust
impl LearningSession {
    // セッション開始
    pub fn start(user_id: UserId, items: Vec<ItemId>, session_type: SessionType) -> Result<Self> {
        let session = Self::new(user_id, items, session_type);
        // Redis に保存
        redis.set_ex(&session_id, &session, 7200)?; // 2時間TTL
        Ok(session)
    }
    
    pub fn judge_correctness(&mut self, item_id: ItemId, judgment: CorrectnessJudgment) -> Result<()> {
        // ビジネスルールの検証
        self.validate_can_judge(item_id)?;
        
        // セッション状態を更新
        self.items[current_index].judgment = Some(judgment);
        
        // Redis を更新
        redis.set_ex(&session_id, &self, 7200)?;
        Ok(())
    }
}
```

### 最小限の永続化

セッション完了時のサマリーのみ PostgreSQL に保存します。

```sql
-- セッションサマリー
CREATE TABLE session_summaries (
    session_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL,
    total_items INT NOT NULL,
    correct_count INT NOT NULL,
    INDEX idx_user_completed (user_id, completed_at DESC)
);

-- 最小限の学習記録  
CREATE TABLE user_item_records (
    user_id UUID,
    item_id UUID,
    last_seen TIMESTAMPTZ NOT NULL,
    correct_count INT NOT NULL,
    total_count INT NOT NULL,
    PRIMARY KEY (user_id, item_id)
);
```

## 設計上の重要な決定

### ハイブリッド UI フロー

1. **即座の解答表示**: ユーザーは任意のタイミングで解答を確認可能
2. **3秒ルール**: 3秒経過後は自動的に「わかった」扱い
3. **明示的な申告**: ユーザーは「わからなかった」を明示的に申告

### 項目選定戦略

- Algorithm Context に委譲（同期通信）
- セッション開始時に一括取得
- キャッシュを活用して高速化

### Progress Context との連携

- セッション完了時にイベントを発行
- 詳細な履歴管理は Progress Context に完全委譲
- Pub/Sub 経由で非同期通知
