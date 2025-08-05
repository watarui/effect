# Learning Context - 集約設計

## 概要

Learning Context は、Effect プロジェクトの中核となるコンテキストで、学習セッションの管理、テストの実施、ユーザーの反応記録、「覚えた」状態の判定を担当します。

### 主要な責務

- ユーザー設定の問題数での学習セッション管理
- 項目の提示と反応時間の記録
- 正誤判定と学習状態の追跡
- 他コンテキストとの連携（Vocabulary、AI Integration、Learning Algorithm）

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

### 3. UserItemRecord（ユーザー項目記録）- 別の集約

ユーザーと項目の学習履歴を永続的に管理します。

**主要な属性**:

- user_id: ユーザー識別子
- item_id: 項目識別子
- mastery_status: 習熟状態
- response_history: 回答履歴
- last_reviewed: 最終復習日時
- created_at: 作成日時

**習熟状態（MasteryStatus）**:

- Unknown: 未知（まだ見たことない）
- Searched: 検索済み（辞書で見た）
- Tested: テスト済み（正答率低い）
- TestFailed: テスト不正解（直近で間違えた）
- ShortTermMastered: 短期記憶で覚えた
- LongTermMastered: 長期記憶で覚えた

**ResponseRecord（回答記録）**:

- responded_at: 回答日時
- response_time_ms: 反応時間
- judgment: 正誤判定
- session_id: セッション識別子

## CQRS による実装

### Write Model（Command Service）

Event Sourcing により、集約の状態はイベントの累積として管理されます。

```rust
impl LearningSession {
    // コマンドハンドラー
    pub fn start(user_id: UserId, items: Vec<ItemId>, session_type: SessionType) -> Result<Self> {
        let session = Self::new(user_id, items, session_type);
        session.record_event(SessionStarted { /* ... */ });
        Ok(session)
    }
    
    pub fn judge_correctness(&mut self, item_id: ItemId, judgment: CorrectnessJudgment) -> Result<()> {
        // ビジネスルールの検証
        self.validate_can_judge(item_id)?;
        
        // イベントの記録
        self.record_event(CorrectnessJudged { /* ... */ });
        Ok(())
    }
}
```

### Read Model（Query/Analytics Service）

非正規化されたビューで高速な読み取りを実現します。

```sql
-- セッションビュー
CREATE TABLE session_views (
    session_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    session_data JSONB NOT NULL,  -- 非正規化データ
    summary JSONB,                 -- 事前計算された統計
    INDEX idx_user_started (user_id, started_at DESC)
);

-- 学習記録ビュー  
CREATE TABLE learning_record_views (
    user_id UUID,
    item_id UUID,
    mastery_data JSONB NOT NULL,  -- 習熟度、統計など
    PRIMARY KEY (user_id, item_id)
);
```

## 設計上の重要な決定

### ハイブリッド UI フロー

1. **即座の解答表示**: ユーザーは任意のタイミングで解答を確認可能
2. **3秒ルール**: 3秒経過後は自動的に「わかった」扱い
3. **明示的な申告**: ユーザーは「わからなかった」を明示的に申告

### 項目選定戦略

- 新規項目優先
- 復習期限が来た項目
- AIによる推薦項目
- ユーザーの明示的な選択

### Progress Context との連携

- UserItemRecord は Progress Context と共有される重要な概念
- 学習履歴は Progress Context でも分析・集計される
- イベント駆動で状態を同期
