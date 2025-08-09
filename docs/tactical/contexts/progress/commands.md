# Progress Context - コマンド

## 概要

Progress Context は純粋な Read Model であるため、従来の意味でのコマンドは持ちません。代わりに、他のコンテキストからのイベントを受信して処理するイベントハンドラーが主要な入力インターフェースとなります。

## イベント受信コマンド

### 1. ProcessLearningEvent

**目的**: Learning Context からのイベントを処理

**受信イベント**:

- SessionCompleted
- ItemMasteryUpdated  
- CorrectnessJudged

**処理内容**:

- 日別統計の更新
- 項目別統計の更新
- 連続学習記録の確認
- マイルストーン達成チェック

### 2. ProcessAlgorithmEvent

**目的**: Learning Algorithm Context からのイベントを処理

**受信イベント**:

- ReviewScheduled
- DifficultyAdjusted
- StatisticsUpdated

**処理内容**:

- 復習スケジュール投影の更新
- 難易度分布の再計算
- アルゴリズムパフォーマンス記録

### 3. ProcessVocabularyEvent

**目的**: Vocabulary Context からのイベントを処理

**受信イベント**:

- VocabularyItemPublished
- ItemDetailsUpdated

**処理内容**:

- 利用可能項目数の更新
- レベル別・領域別分布の更新

### 4. ProcessUserEvent

**目的**: User Context からのイベントを処理

**受信イベント**:

- UserCreated
- UserPreferencesUpdated

**処理内容**:

- 初期投影の作成
- 目標設定の反映

### 5. RebuildProjection

**目的**: イベントストアから特定の投影を再構築

**パラメータ**:

- projection_type: 再構築する投影の種類
- from_timestamp: 開始時点（オプション）
- to_timestamp: 終了時点（オプション）

**処理内容**:

- イベントストアから該当期間のイベントを読み取り
- 指定された投影を最初から再構築
- Blue-Green デプロイメントでダウンタイムなし

### 6. CreateSnapshot

**目的**: 現在の状態のスナップショットを作成

**パラメータ**:

- projection_type: スナップショット対象
- user_id: 対象ユーザー（オプション）

**処理内容**:

- 現在の投影状態を保存
- 将来の再構築時の起点として使用

## イベント処理パターン

### 冪等性の保証

すべてのイベント処理は冪等：

- イベントID による重複チェック
- 同じイベントを複数回処理しても結果は同じ

### 順序保証

イベントの処理順序を保証：

- グローバルシーケンス番号
- タイムスタンプによる順序付け
- 遅延到着イベントの適切な処理

### エラー処理

処理失敗時の戦略：

- 再試行可能なエラー: 指数バックオフで再試行
- 永続的エラー: Dead Letter Queue へ
- データ不整合: 投影の再構築をトリガー

## バッチ処理コマンド

### GenerateDailyReport

**実行タイミング**: 毎日深夜

**処理内容**:

- 前日の学習活動を集計
- 日別統計の確定
- キャッシュの事前生成

### UpdateWeeklyStats

**実行タイミング**: 5分間隔

**処理内容**:

- 週別統計の更新
- トレンド分析の実行
- パフォーマンス指標の計算

### ArchiveOldEvents

**実行タイミング**: 月次

**処理内容**:

- 1年以上前のイベントをアーカイブ
- スナップショットの最適化
- ストレージの効率化
