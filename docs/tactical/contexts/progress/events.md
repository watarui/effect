# Progress Context - ドメインイベント

## 概要

Progress Context は純粋なイベントソーシングアーキテクチャを採用しているため、自身でイベントを発行することは少なく、主に他のコンテキストからのイベントを受信して処理します。ここでは、Progress Context が発行する可能性のあるイベントと、受信して処理する主要なイベントを記載します。

## Progress Context が発行するイベント

### 1. MilestoneAchieved

学習のマイルストーンが達成されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- milestone_type: マイルストーンの種類
- achievement_details: 達成の詳細

**マイルストーンの種類**:

- FirstSession: 初回学習セッション完了
- ConsecutiveDays: 連続学習日数達成（7日、30日、100日など）
- ItemsMastered: 習得項目数達成（100、500、1000項目など）
- LevelComplete: CEFR レベル完了（A1、A2など）
- PerfectWeek: 1週間毎日学習
- StudyHours: 累計学習時間達成

### 2. StreakUpdated

連続学習記録が更新されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- current_streak: 現在の連続日数
- previous_streak: 更新前の連続日数
- longest_streak: 最長記録

### 3. ProgressReportGenerated

進捗レポートが生成されたことを表すイベント。

**イベント構造**:

- event_id: イベント識別子
- occurred_at: 発生日時
- user_id: ユーザー識別子
- report_type: レポートタイプ（Daily、Weekly、Monthly）
- report_period: 対象期間
- summary_stats: サマリー統計

## 受信して処理する主要なイベント

### Learning Context から

**SessionCompleted**:

- 日別統計の更新
- セッション統計の集計
- 連続学習記録の更新

**ItemMasteryUpdated**:

- 習熟度統計の更新
- レベル別進捗の更新
- マイルストーン達成チェック

**CorrectnessJudged**:

- 項目別統計の更新
- 正答率の再計算
- 領域別パフォーマンスの更新

### Learning Algorithm Context から

**ReviewScheduled**:

- 復習スケジュールの更新
- 将来の学習負荷予測

**DifficultyAdjusted**:

- 項目難易度統計の更新
- レベル別分布の再計算

**StatisticsUpdated**:

- アルゴリズムパフォーマンスの記録
- 最適化指標の追跡

### Vocabulary Context から

**VocabularyItemPublished**:

- 利用可能項目数の更新
- レベル別・領域別の項目分布更新

**ItemDetailsUpdated**:

- 項目メタデータの同期
- 統計の再集計トリガー

### User Context から

**UserPreferencesUpdated**:

- 目標設定の反映
- 進捗計算基準の更新

**UserCreated**:

- 初期プロジェクションの作成
- ウェルカムマイルストーンの設定

## イベント処理パターン

### リアルタイム処理

- SessionCompleted: 即座に統計更新
- ItemMasteryUpdated: 即座に進捗反映
- MilestoneAchieved: 即座に通知

### バッチ処理

- 日次集計: 深夜に前日分を集計
- 週次・月次レポート: 定期的に生成
- 統計の再計算: 低負荷時に実行

### イベントの冪等性

すべてのイベント処理は冪等性を保証：

- 同じイベントを複数回処理しても結果は同じ
- イベントID による重複チェック
- タイムスタンプによる順序保証
