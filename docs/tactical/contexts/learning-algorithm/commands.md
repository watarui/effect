# Learning Algorithm Context - コマンド設計

## 概要

Learning Algorithm Context で実行可能なコマンド（書き込み操作）の定義です。SM-2 アルゴリズムによる学習記録の更新と最適化を行います。

## コマンド一覧

| コマンド名 | 説明 | 実行タイミング |
|-----------|------|-------------|
| RecordReview | 復習結果を記録 | 項目の正誤判定後 |
| SelectItems | 学習項目を選定 | セッション開始時 |
| AnalyzePerformance | パフォーマンスを分析 | 定期実行/セッション終了時 |
| AdjustDifficulty | 難易度を調整 | パフォーマンス分析後 |
| SuspendItem | 項目を一時停止 | ユーザー要求時 |
| ResetProgress | 進捗をリセット | ユーザー要求時 |

## コマンド詳細

### 1. RecordReview

復習結果を記録し、SM-2 アルゴリズムで次回復習日を計算します。

**コマンド構造**:

- user_id: ユーザー識別子
- item_id: 項目識別子
- is_correct: 正誤
- response_time_ms: 反応時間（ミリ秒）
- review_context: 復習コンテキスト（オプション）

**処理フロー**:

1. 反応時間と正誤から品質評価（0-5）を計算
2. SM-2 アルゴリズムで難易度係数を更新
3. 次回復習間隔を計算
4. 学習記録を更新
5. ReviewRecorded イベント発行
6. ReviewScheduled イベント発行

**品質評価の決定ロジック**:

- 正解 + 3秒以内 → 品質5
- 正解 + 10秒以内 → 品質4
- 正解 + 30秒以内 → 品質3
- 正解 + 30秒以上 → 品質2
- 不正解（部分的） → 品質1
- 不正解（完全） → 品質0

**バリデーション**:

- response_time_ms は 0 以上
- 学習記録が存在すること

### 2. SelectItems

学習セッションのための項目を選定します。

**コマンド構造**:

- user_id: ユーザー識別子
- session_config: セッション設定
  - item_count: 項目数（デフォルト: 20）
  - new_item_ratio: 新規項目の比率（デフォルト: 0.2）
  - difficulty_variance: 難易度分散（low/medium/high）
  - time_limit_minutes: 時間制限（オプション）

**処理フロー**:

1. 期限切れ項目を最優先で選定
2. 新規項目と復習項目をバランスよく選定
3. 難易度を適切に分散
4. 85%ルールに基づく調整
5. ItemsSelected レスポンス返却

**選定優先順位**:

1. Overdue（期限切れ）項目
2. Due（復習予定）項目
3. Learning（学習中）項目
4. New（新規）項目

**最適化考慮事項**:

- 認知負荷の分散
- 学習効率の最大化
- ユーザーの集中力維持

### 3. AnalyzePerformance

ユーザーの学習パフォーマンスを分析します。

**コマンド構造**:

- user_id: ユーザー識別子
- period_days: 分析期間（デフォルト: 30）
- analysis_type: 分析タイプ
  - comprehensive: 総合分析
  - accuracy_focused: 正答率重視
  - velocity_focused: 学習速度重視

**処理フロー**:

1. 指定期間の学習データを取得
2. 統計指標を計算
3. 学習パターンを検出
4. 最適難易度を算出（85%ルール）
5. PerformanceAnalyzed イベント発行

**分析指標**:

- overall_accuracy: 全体正答率
- recent_accuracy: 最近の正答率
- learning_velocity: 学習速度
- retention_rate: 記憶保持率
- optimal_difficulty: 最適難易度

### 4. AdjustDifficulty

難易度を動的に調整します。

**コマンド構造**:

- user_id: ユーザー識別子
- adjustment_strategy: 調整戦略
  - automatic: 85%ルールに基づく自動調整
  - manual: 手動調整
- target_accuracy: 目標正答率（デフォルト: 0.85）

**処理フロー**:

1. 現在の正答率を確認
2. 目標との差分を計算
3. 調整方向を決定（上げる/下げる/維持）
4. 項目の難易度係数を調整
5. DifficultyAdjusted イベント発行

**85%ルールの適用**:

- 正答率 > 90%: より難しい項目を増やす
- 正答率 < 80%: より簡単な項目を増やす
- 80% ≤ 正答率 ≤ 90%: 現状維持

### 5. SuspendItem

項目の学習を一時停止します。

**コマンド構造**:

- user_id: ユーザー識別子
- item_id: 項目識別子
- reason: 停止理由（オプション）
- resume_date: 再開予定日（オプション）

**処理フロー**:

1. 学習記録の存在確認
2. ステータスを Suspended に変更
3. 復習スケジュールから除外
4. ItemSuspended イベント発行

**使用ケース**:

- 一時的に難しすぎる項目
- 誤りのある項目の報告
- ユーザーの学習戦略

### 6. ResetProgress

特定の項目または全体の進捗をリセットします。

**コマンド構造**:

- user_id: ユーザー識別子
- scope: リセット範囲
  - single_item: 単一項目
  - category: カテゴリ全体
  - all: 全項目
- item_ids: 対象項目ID（scope が single_item の場合）
- category: カテゴリ（scope が category の場合）

**処理フロー**:

1. 対象範囲の特定
2. 学習記録を初期状態に戻す
3. 難易度係数を初期値（2.5）に設定
4. ProgressReset イベント発行

**注意事項**:

- 不可逆的な操作
- 確認ダイアログが必要
- 監査ログに記録

## エラーハンドリング

### 主要なエラー型

| エラー型 | 説明 | 発生条件 |
|---------|------|----------|
| RecordNotFound | 学習記録が見つからない | 未学習項目への操作 |
| InvalidQuality | 品質評価が無効 | 計算エラー |
| InsufficientData | データ不足 | 分析に必要なデータ不足 |
| InvalidConfiguration | 設定が無効 | セッション設定の矛盾 |

## トランザクション管理

すべてのコマンドはトランザクション内で実行され、以下を保証：

- 原子性: 全体の成功または失敗
- 一貫性: SM-2 アルゴリズムの不変条件維持
- 分離性: 同時実行の制御
- 永続性: 確実なデータ保存

## パフォーマンス要件

| コマンド | 応答時間要件 |
|---------|-------------|
| RecordReview | <50ms |
| SelectItems | <100ms |
| AnalyzePerformance | <500ms |
| AdjustDifficulty | <200ms |
| SuspendItem | <50ms |
| ResetProgress | <100ms |

## 実装上の注意事項

1. **SM-2 アルゴリズムの正確性**: 科学的に検証された計算式を厳密に実装
2. **並行性の考慮**: ユーザーごとの独立した処理
3. **キャッシュの活用**: 頻繁にアクセスされるデータのキャッシング
4. **イベント発行**: 他コンテキストへの確実な通知
