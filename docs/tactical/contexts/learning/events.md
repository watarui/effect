# Learning Context - ドメインイベント

## 概要

Learning Context から Progress Context へ通知されるドメインイベントの定義です。学習セッションの主要な状態変化を Progress Context に伝えます。

## Progress Context への通知イベント

| イベント名 | 説明 | 発生タイミング |
|-----------|------|-------------|
| SessionStarted | 学習セッションが開始された | ユーザーが学習を開始した時 |
| SessionCompleted | セッションが完了した | 全問題終了または中断時 |
| CorrectnessJudged | 正誤が判定された | ユーザーが正誤を申告した時 |

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

### 2. CorrectnessJudged

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

### 3. SessionCompleted

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

## セッション内部イベント（通知対象外）

以下のイベントは Learning Context 内部でのみ使用され、Progress Context へは通知されません：

- **ItemPresented**: 項目の提示（セッション内部管理）
- **AnswerRevealed**: 解答の表示（UI制御用）
- **ItemMasteryUpdated**: 習熟度更新（Progress Context が管理）

## イベントフロー

```
SessionStarted → [Progress Context へ通知]
  ↓
(セッション内部で項目提示・解答表示)
  ↓
CorrectnessJudged → [Progress Context へ通知]
  ↓ (繰り返し)
SessionCompleted → [Progress Context へ通知]
```

## Progress Context への伝播

- **SessionStarted**: セッション開始を記録
- **SessionCompleted**: セッション結果を保存、統計更新
- **CorrectnessJudged**: 学習履歴に記録、習熟度計算

## イベント発行の実装

Google Cloud Pub/Sub を使用して Progress Context にイベントを通知します：

```rust
// イベント発行の例
async fn publish_to_progress(event: LearningEvent) {
    let topic = "learning-to-progress";
    
    // Pub/Sub に発行
    pubsub_client
        .publish(topic, &event)
        .await?;
}

// セッション完了時の処理
async fn complete_session(session_id: SessionId) {
    // 1. Redis からセッションデータを取得
    let session = redis.get(&session_id).await?;
    
    // 2. サマリーを計算
    let summary = calculate_summary(&session);
    
    // 3. Progress Context に通知
    publish_to_progress(SessionCompleted {
        session_id,
        user_id: session.user_id,
        total_items: summary.total_items,
        correct_count: summary.correct_count,
        occurred_at: Utc::now(),
    }).await?;
    
    // 4. Redis から削除
    redis.delete(&session_id).await?;
}
```

### 通知の特徴

- **Fire-and-forget**: 非同期での通知
- **最終的一貫性**: Progress Context での処理を保証
- **軽量**: 必要最小限の情報のみ送信

これにより：

- Learning Context はセッション実行に集中
- Progress Context が履歴管理を担当
- シンプルで保守しやすいアーキテクチャ
