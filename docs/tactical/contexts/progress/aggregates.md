# Progress Context - 集約設計

## 概要

Progress Context は、Effect プロジェクトにおける「学習活動の鏡」として機能します。複数のコンテキストから発行されるイベントを集約し、学習の全体像を可視化する、純粋な CQRS/イベントソーシングの実践例です。

### 主要な責務

- **イベント集約**: Learning、Learning Algorithm、Vocabulary Context からのイベント収集
- **統計計算**: 日別・週別・月別の学習統計の生成
- **進捗分析**: 領域別（R/W/L/S）、レベル別（CEFR）の習熟度分析
- **可視化データ生成**: GraphQL 経由でフロントエンドに提供するデータの準備

### 設計方針

- **イベントソーシング**: すべての統計はイベントから導出
- **リードモデル中心**: 集約は持たず、プロジェクション（投影）のみ
- **GraphQL 最適化**: 柔軟なクエリに対応できる細かいリードモデル
- **ハイブリッド更新**: リアルタイムとバッチ処理の使い分け

## アーキテクチャ設計

### Progress Context は集約を持たない

Progress Context は純粋な読み取りモデルであり、従来の意味での「集約」は存在しません。すべてのデータは他のコンテキストから発行されたイベントから導出されます。

### イベント駆動アーキテクチャ

**受信するイベント**:

- Learning Context: SessionCompleted, ItemMasteryUpdated
- Learning Algorithm Context: ReviewRecorded, StatisticsUpdated
- Vocabulary Context: ItemCreated, ItemUpdated
- AI Integration Context: ContentGenerated
- User Context: PreferencesUpdated

## プロジェクション（リードモデル）の設計

### 1. DailyStatsProjection（日別統計）

**目的**: 日ごとの学習活動を集計

**主要な属性**:

- user_id: ユーザー識別子
- date: 日付
- session_count: セッション数
- total_review_count: 総復習数
- correct_count: 正解数
- incorrect_count: 不正解数
- total_study_time: 総学習時間
- average_response_time: 平均反応時間
- new_items_learned: 新規学習項目数
- items_mastered: 習得項目数

### 2. WeeklyStatsProjection（週別統計）

**目的**: 週単位の学習傾向を分析

**主要な属性**:

- user_id: ユーザー識別子
- week_start: 週開始日
- active_days: アクティブ日数
- total_review_count: 総復習数
- average_daily_reviews: 日平均復習数
- mastery_progression: 習熟度の進展
- consistency_score: 継続性スコア

### 3. ItemStatsProjection（項目別統計）

**目的**: 個別項目の学習状況を追跡

**主要な属性**:

- user_id: ユーザー識別子
- item_id: 項目識別子
- first_seen: 初回学習日
- last_reviewed: 最終復習日
- total_reviews: 総復習回数
- correct_count: 正解回数
- accuracy_rate: 正答率
- average_response_time: 平均反応時間
- mastery_level: 習熟レベル

### 4. DomainStatsProjection（領域別統計）

**目的**: Reading/Writing/Listening/Speaking 別の進捗

**主要な属性**:

- user_id: ユーザー識別子
- domain: 領域（R/W/L/S）
- total_items: 総項目数
- mastered_items: 習得項目数
- average_accuracy: 平均正答率
- time_spent: 学習時間

### 5. LevelStatsProjection（レベル別統計）

**目的**: CEFR レベル別の習熟度

**主要な属性**:

- user_id: ユーザー識別子
- cefr_level: CEFR レベル（A1-C2）
- total_items: 総項目数
- mastered_items: 習得項目数
- in_progress_items: 学習中項目数
- not_started_items: 未着手項目数

### 6. StreakProjection（連続学習記録）

**目的**: 学習の継続性を追跡

**主要な属性**:

- user_id: ユーザー識別子
- current_streak: 現在の連続日数
- longest_streak: 最長連続日数
- last_activity_date: 最終活動日
- total_active_days: 総活動日数

## 設計上の重要な決定

### イベントソーシングの利点

1. **完全な監査証跡**: すべての変更がイベントとして記録
2. **柔軟な投影**: 新しい統計ビューを後から追加可能
3. **時系列分析**: 過去の任意の時点の状態を再現可能

### GraphQL との統合

- 各プロジェクションは GraphQL のリゾルバーと1対1対応
- DataLoader パターンで N+1 問題を回避
- リアルタイムサブスクリプションで進捗を即座に反映

### パフォーマンス最適化

- 頻繁にアクセスされる統計はキャッシュ
- 重い集計はバッチ処理で事前計算
- イベントの並列処理で更新を高速化
