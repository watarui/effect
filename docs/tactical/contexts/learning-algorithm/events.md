# Learning Algorithm Context - ドメインイベント

## 概要

Learning Algorithm Context で発生するドメインイベントのカタログです。SM-2 アルゴリズムによる復習スケジューリング、難易度調整、学習パフォーマンス分析に関するイベントを管理します。

## イベント一覧

| イベント名 | 説明 | 発生タイミング |
|-----------|------|-------------|
| ItemLearningStarted | 項目の学習が開始された | 新規項目の初回学習時 |
| ReviewRecorded | 復習結果が記録された | 復習の正誤判定完了時 |
| ReviewScheduled | 次回復習がスケジュールされた | SM-2計算完了時 |
| DifficultyAdjusted | 難易度係数が調整された | 品質評価に基づく調整時 |
| ItemMastered | 項目が習得された | 一定条件を満たした時 |
| ItemSuspended | 項目が一時停止された | 学習を中断した時 |
| PerformanceAnalyzed | パフォーマンスが分析された | 定期分析実行時 |
| OptimalDifficultyCalculated | 最適難易度が計算された | 85%ルール適用時 |

## イベント詳細

### 1. ItemLearningStarted

項目の学習が開始されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- item_id: 項目識別子
- initial_difficulty: 初期難易度
- learning_context: 学習コンテキスト

**発生条件**:

- ユーザーが初めて項目を学習した時
- ItemLearningRecord が新規作成された時

### 2. ReviewRecorded

復習結果が記録されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- record_id: 学習記録識別子
- user_id: ユーザー識別子
- item_id: 項目識別子
- quality: 品質評価（0-5）
- response_time: 反応時間
- is_correct: 正誤
- review_type: 復習タイプ（New、Learning、Review）

**品質評価の基準**:

- 5: 完璧な想起（3秒以内）
- 4: 正解だが遅延あり（10秒以内）
- 3: 正解だが困難（30秒以内）
- 2: 不正解だが部分的に記憶
- 1: 不正解で記憶なし
- 0: 完全なブラックアウト

**発生条件**:

- Learning Context から CorrectnessJudged イベントを受信
- 反応時間と正誤から品質を計算

### 3. ReviewScheduled

次回復習がスケジュールされたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- record_id: 学習記録識別子
- user_id: ユーザー識別子
- item_id: 項目識別子
- next_review_date: 次回復習日
- interval_days: 復習間隔（日数）
- algorithm_version: 使用アルゴリズムバージョン

**発生条件**:

- ReviewRecorded イベントの後
- SM-2 アルゴリズムによる計算完了時

### 4. DifficultyAdjusted

難易度係数が調整されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- record_id: 学習記録識別子
- user_id: ユーザー識別子
- item_id: 項目識別子
- old_easiness_factor: 変更前の難易度係数
- new_easiness_factor: 変更後の難易度係数
- adjustment_reason: 調整理由

**調整理由**:

- QualityBased: 品質評価に基づく
- PerformanceBased: パフォーマンス分析に基づく
- Manual: 手動調整

**発生条件**:

- 品質評価が3未満の場合
- パフォーマンス分析で異常値検出時

### 5. ItemMastered

項目が習得されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- item_id: 項目識別子
- mastery_criteria: 習得基準
- total_reviews: 総復習回数
- accuracy_rate: 正答率
- retention_days: 保持日数

**習得基準**:

- 連続5回以上正解
- 復習間隔が21日以上
- 正答率90%以上

**発生条件**:

- すべての習得基準を満たした時
- Learning Context に伝播

### 6. PerformanceAnalyzed

学習パフォーマンスが分析されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- analysis_period: 分析期間
- performance_metrics: パフォーマンス指標
- recommendations: 推奨事項

**パフォーマンス指標**:

- overall_accuracy: 全体正答率
- learning_velocity: 学習速度
- retention_rate: 保持率
- optimal_study_time: 最適学習時間

### 7. OptimalDifficultyCalculated

最適難易度が計算されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- current_accuracy: 現在の正答率
- optimal_difficulty: 最適難易度レベル
- adjustment_direction: 調整方向（Increase、Decrease、Maintain）

**発生条件**:

- 定期的なパフォーマンス分析時
- 85%ルールに基づく計算

## 他コンテキストとの連携

### Learning Context への影響

- ItemMastered → ItemMasteryUpdated への変換
- OptimalDifficultyCalculated → 項目選定戦略の調整

### Progress Context への影響

- ReviewRecorded → 統計更新
- PerformanceAnalyzed → 進捗レポート生成

### AI Integration Context との連携

- 学習パターンの分析データ提供
- パーソナライズされた推薦の基礎データ
