# Learning Context - ドメインイベント

## 概要

Learning Context で発生するドメインイベントのカタログです。学習セッションのライフサイクルと学習進捗の変化を表現します。

## イベント一覧

| イベント名 | 説明 | 発生タイミング |
|-----------|------|-------------|
| SessionStarted | 学習セッションが開始された | ユーザーが学習を開始した時 |
| ItemPresented | 項目が提示された | 問題が画面に表示された時 |
| AnswerRevealed | 解答が表示された | ユーザー要求または3秒経過時 |
| CorrectnessJudged | 正誤が判定された | ユーザーが正誤を申告した時 |
| SessionCompleted | セッションが完了した | 全問題終了または中断時 |
| ItemMasteryUpdated | 項目の習熟状態が更新された | 学習結果により状態が変化した時 |

## イベント詳細

### 1. SessionStarted

学習セッションの開始を表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- session_id: セッション識別子
- user_id: ユーザー識別子
- item_count: 問題数（1-100）
- strategy: 選定戦略

**発生条件**:

- ユーザーが学習開始ボタンを押した時
- 項目選定が完了し、セッションが作成された時

### 2. ItemPresented

項目が学習者に提示されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- session_id: セッション識別子
- item_id: 項目識別子
- time_limit: 制限時間（通常3秒）

**発生条件**:

- 新しい問題が画面に表示された時
- 前の問題の処理が完了し、次の問題に移った時

### 3. AnswerRevealed

解答が表示されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- session_id: セッション識別子
- item_id: 項目識別子
- trigger: トリガー（UserRequested または TimeLimit）

**発生条件**:

- ユーザーが「解答を見る」ボタンを押した時
- 3秒のタイムリミットに達した時

### 4. CorrectnessJudged

正誤が判定されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- session_id: セッション識別子
- item_id: 項目識別子
- judgment: 判定結果
  - AutoConfirmed: 3秒経過で自動正解
  - UserConfirmedCorrect: ユーザーが「わかった」
  - UserConfirmedIncorrect: ユーザーが「わからなかった」

**発生条件**:

- 3秒経過後、自動的に正解扱いされた時
- ユーザーが明示的に正誤を申告した時

### 5. SessionCompleted

学習セッションが完了したことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- session_id: セッション識別子
- total_items: 総問題数
- correct_count: 正解数

**発生条件**:

- すべての問題が完了した時
- ユーザーがセッションを中断した時

### 6. ItemMasteryUpdated

項目の習熟状態が更新されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- item_id: 項目識別子
- old_status: 変更前の状態
- new_status: 変更後の状態

**習熟状態の種類**:

- Unknown: 未知
- Searched: 検索済み
- Tested: テスト済み
- TestFailed: テスト不正解
- ShortTermMastered: 短期記憶で習得
- LongTermMastered: 長期記憶で習得

**発生条件**:

- 学習結果により習熟度が変化した時
- 初めて項目を学習した時
- 一定の条件を満たして習熟レベルが上がった時

## イベントフロー

```
SessionStarted
  ↓
ItemPresented → AnswerRevealed → CorrectnessJudged → ItemMasteryUpdated
  ↓ (繰り返し)
SessionCompleted
```

## 他コンテキストへの伝播

- **Progress Context**: SessionCompleted, ItemMasteryUpdated を受信して進捗を更新
- **Learning Algorithm Context**: CorrectnessJudged を受信してアルゴリズムパラメータを更新
- **AI Integration Context**: 学習パターンを分析して推薦を改善

## Event Sourcing による実装

すべてのイベントは Event Store に以下の形式で保存される：

```sql
events (
  event_id UUID,
  aggregate_id UUID,        -- session_id または user_item_record_id
  aggregate_type VARCHAR,   -- "LearningSession" または "UserItemRecord"
  event_type VARCHAR,       -- "SessionStarted", "ItemPresented" など
  event_data JSONB,         -- イベントの詳細データ
  event_version INTEGER,    -- 集約のバージョン
  occurred_at TIMESTAMPTZ
)
```

### イベントハンドリング

```rust
// Projection Service でのイベント処理例
async fn handle_correctness_judged(event: CorrectnessJudged) {
    // 1. セッションビューの更新
    update_session_item_view(&event).await?;
    
    // 2. 学習記録の更新
    update_user_item_record(&event).await?;
    
    // 3. 習熟度の再計算
    if let Some(new_status) = calculate_mastery_status(&event).await? {
        // 新しいイベントを生成
        publish_event(ItemMasteryUpdated {
            user_id: event.user_id,
            item_id: event.item_id,
            old_status: new_status.old,
            new_status: new_status.new,
            occurred_at: Utc::now(),
        }).await?;
    }
    
    // 4. Progress Context への転送
    forward_to_progress_context(&event).await?;
}
```

### イベントの順序保証

- 同一セッション内のイベントは順序を保証
- Pub/Sub のメッセージ順序保証機能を使用
- 集約ごとにパーティショニング

これにより：

- 完全な学習履歴の保持
- 任意の時点の状態再現
- 監査証跡の提供
- 分析用データの蓄積
