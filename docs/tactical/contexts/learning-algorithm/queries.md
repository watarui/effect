# Learning Algorithm Context - クエリ設計

## 概要

Learning Algorithm Context で実行可能なクエリ（読み取り操作）の定義です。学習の最適化に必要な情報を効率的に提供します。

## クエリ一覧

| クエリ名 | 説明 | 使用場面 | キャッシュ |
|---------|------|----------|-----------|
| GetNextItems | 次の学習項目を取得 | セッション開始時 | 5分 |
| GetReviewSchedule | 復習スケジュールを取得 | カレンダー表示 | 30分 |
| GetPerformanceStats | パフォーマンス統計を取得 | ダッシュボード | 1時間 |
| GetLearningProgress | 学習進捗を取得 | 進捗表示 | 30分 |
| GetItemDifficulty | 項目の難易度情報を取得 | 詳細表示 | 1時間 |
| GetOptimalStudyTime | 最適な学習時間を取得 | 学習計画 | 1日 |

## クエリ詳細

### 1. GetNextItems

学習セッションのための次の項目リストを取得します。

**クエリパラメータ**:

- user_id: ユーザー識別子
- count: 取得項目数（デフォルト: 20）
- include_new: 新規項目を含むか（デフォルト: true）
- difficulty_range: 難易度範囲（オプション）

**レスポンスフィールド**:

- items: 項目リスト
  - item_id: 項目識別子
  - priority: 優先度（1-10）
  - review_type: 復習タイプ（New/Learning/Review/Overdue）
  - easiness_factor: 現在の難易度係数
  - last_review_date: 最終復習日
  - overdue_days: 期限切れ日数（該当する場合）
- recommended_order: 推奨学習順序
- estimated_duration: 推定所要時間

**処理フロー**:

1. キャッシュチェック
2. 期限切れ項目の抽出
3. 復習予定項目の抽出
4. 新規項目の抽出
5. 最適な順序で並び替え
6. レスポンス構築

**最適化**:

- Redis キャッシュ（5分間）
- プリフェッチによる高速化
- インデックスの活用

### 2. GetReviewSchedule

復習スケジュールを取得します。

**クエリパラメータ**:

- user_id: ユーザー識別子
- date_range: 期間
  - start_date: 開始日
  - end_date: 終了日
- group_by: グループ化単位（day/week/month）

**レスポンスフィールド**:

- schedule: スケジュール配列
  - date: 日付
  - item_count: 項目数
  - review_items: 復習項目数
  - new_items: 新規項目数
  - estimated_time: 推定時間
- summary: サマリー情報
  - total_items: 総項目数
  - peak_day: ピーク日
  - average_daily: 日平均

**使用例**:

- カレンダービューでの表示
- 学習計画の立案
- 負荷の可視化

### 3. GetPerformanceStats

学習パフォーマンスの統計を取得します。

**クエリパラメータ**:

- user_id: ユーザー識別子
- period: 期間（7d/30d/90d/all）
- metrics: 取得する指標（配列）
  - accuracy: 正答率
  - velocity: 学習速度
  - retention: 保持率
  - consistency: 継続性

**レスポンスフィールド**:

- overall_stats: 全体統計
  - accuracy_rate: 正答率
  - average_quality: 平均品質評価
  - total_reviews: 総復習回数
  - mastered_items: 習得項目数
- trend_data: トレンドデータ
  - daily_accuracy: 日別正答率
  - weekly_progress: 週別進捗
- performance_score: パフォーマンススコア（0-100）
- recommendations: 改善推奨事項

**85%ルール適用**:

- 現在の正答率と理想値（85%）の比較
- 調整推奨の提供

### 4. GetLearningProgress

学習進捗の詳細を取得します。

**クエリパラメータ**:

- user_id: ユーザー識別子
- category: カテゴリ（オプション）
- include_details: 詳細を含むか（デフォルト: false）

**レスポンスフィールド**:

- progress_summary: 進捗サマリー
  - total_items: 総項目数
  - new_items: 新規項目数
  - learning_items: 学習中項目数
  - review_items: 復習項目数
  - mastered_items: 習得済み項目数
- mastery_distribution: 習得度分布
  - beginner: 初級レベル
  - intermediate: 中級レベル
  - advanced: 上級レベル
- estimated_completion: 推定完了時期

**可視化サポート**:

- 円グラフ用データ
- 進捗バー用データ
- トレンドグラフ用データ

### 5. GetItemDifficulty

特定項目の難易度情報を取得します。

**クエリパラメータ**:

- user_id: ユーザー識別子
- item_ids: 項目ID配列
- include_history: 履歴を含むか

**レスポンスフィールド**:

- difficulty_info: 難易度情報配列
  - item_id: 項目識別子
  - easiness_factor: 難易度係数
  - average_quality: 平均品質評価
  - review_count: 復習回数
  - accuracy_rate: 正答率
  - difficulty_level: 難易度レベル（Easy/Medium/Hard）
- history: 履歴データ（オプション）
  - quality_trend: 品質評価の推移
  - ef_changes: 難易度係数の変化

**活用方法**:

- 項目の詳細画面表示
- 学習戦略の調整
- 苦手項目の特定

### 6. GetOptimalStudyTime

最適な学習時間を取得します。

**クエリパラメータ**:

- user_id: ユーザー識別子
- target_date: 対象日（デフォルト: 今日）
- constraints: 制約条件
  - available_minutes: 利用可能時間
  - priority: 優先度（efficiency/coverage）

**レスポンスフィールド**:

- optimal_duration: 最適な学習時間（分）
- recommended_sessions: 推奨セッション
  - start_time: 開始時刻
  - duration: 所要時間
  - item_count: 項目数
  - focus_type: フォーカスタイプ
- reasoning: 推奨理由
- alternative_options: 代替オプション

**科学的根拠**:

- 集中力の持続時間
- 記憶の定着効率
- 疲労度の考慮

## パフォーマンス最適化

### キャッシュ戦略

| データ種別 | TTL | 更新タイミング |
|-----------|-----|---------------|
| 項目選定結果 | 5分 | 復習記録時 |
| スケジュール | 30分 | 項目更新時 |
| 統計データ | 1時間 | 定期バッチ |
| 進捗情報 | 30分 | 学習完了時 |

### インデックス設計

効率的なクエリのための主要インデックス：

- (user_id, next_review_date)
- (user_id, status)
- (user_id, created_at)

### バッチ処理

- 統計計算は定期バッチで事前計算
- 重い集計処理は非同期実行
- 結果はキャッシュに保存

## エラーハンドリング

| エラー型 | 説明 | 対処法 |
|---------|------|--------|
| NoDataFound | データが存在しない | 空の結果を返す |
| InvalidPeriod | 無効な期間指定 | デフォルト期間を使用 |
| CacheError | キャッシュエラー | DBから直接取得 |

## 実装上の注意事項

1. **レスポンス最適化**: 必要最小限のデータのみ返却
2. **N+1問題の回避**: 適切なJOINとプリロード
3. **ページネーション**: 大量データの分割取得
4. **非同期処理**: 重い計算の非同期実行
