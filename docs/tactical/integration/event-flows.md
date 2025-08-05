# クロスコンテキスト イベントフロー

## 概要

異なる Bounded Context 間で共有・連携されるイベントのカタログです。これらのイベントは Integration Events として、Context 間の疎結合な連携を実現します。

## Context 間の関係マップ

```
Vocabulary ─────┬─────> Progress (項目情報の提供)
    │           │
    │           └─────> Learning (テスト用項目データ)
    │
    └─────────────────> AI Integration (生成要求)
    
Progress ───────┬─────> Notification (学習リマインダー)
    │           │
    │           └─────> User (統計情報)
    │
    └─────────────────> Learning (学習履歴)

AI Integration ────────> Vocabulary (生成結果)
```

## 主要な Integration Events

### 1. Vocabulary → Progress Context

**VocabularyItemPublished**

- **目的**: 語彙項目が公開され、学習可能になったことを通知
- **ペイロード**:
  - item_id: 項目ID
  - entry_id: エントリID
  - spelling: 綴り
  - part_of_speech: 品詞
  - cefr_level: CEFRレベル
  - difficulty_estimate: 難易度推定値（0.0-1.0）
  - content_quality_score: コンテンツ品質スコア（0.0-1.0）

**VocabularyItemUpdated**

- **目的**: 項目の重要な属性が更新されたことを通知
- **ペイロード**:
  - item_id: 項目ID
  - updated_fields: 更新されたフィールドのリスト
  - difficulty_changed: 難易度が変更されたか
  - cefr_level_changed: CEFRレベルが変更されたか

### 2. Progress → Learning Context

**ItemProgressChanged**

- **目的**: 項目の学習進捗が変更されたことを通知
- **ペイロード**:
  - user_id: ユーザーID
  - item_id: 項目ID
  - new_mastery_level: 新しい習熟度
  - repetition_count: 総復習回数
  - next_review_date: 次回復習予定日

**UserProgressMilestoneReached**

- **目的**: ユーザーが重要なマイルストーンに到達したことを通知
- **ペイロード**:
  - user_id: ユーザーID
  - milestone_type: マイルストーンの種類
  - achievement_date: 達成日時
  - details: 詳細情報

### 3. Learning → Progress Context

**LearningSessionCompleted**

- **目的**: 学習セッションが完了したことを通知
- **ペイロード**:
  - session_id: セッションID
  - user_id: ユーザーID
  - session_type: セッションタイプ
  - results: セッション結果のサマリー
  - item_results: 個別項目の結果リスト

**ItemStudied**

- **目的**: 個別項目が学習されたことを通知
- **ペイロード**:
  - user_id: ユーザーID
  - item_id: 項目ID
  - study_type: 学習タイプ（新規/復習）
  - result: 学習結果（正解/不正解）
  - response_time: 回答時間

### 4. Vocabulary ↔ AI Integration Context

**AIGenerationRequested** (Vocabulary → AI)

- **目的**: AI による項目内容の生成を要求
- **ペイロード**:
  - item_id: 項目ID
  - spelling: 綴り
  - disambiguation: 意味の区別
  - generation_type: 生成タイプ
  - requested_fields: 生成対象フィールド

**AIGenerationCompleted** (AI → Vocabulary)

- **目的**: AI による生成が完了したことを通知
- **ペイロード**:
  - request_id: 元の要求ID
  - item_id: 項目ID
  - generated_content: 生成されたコンテンツ
  - generation_metadata: 生成に関するメタデータ

### 5. User Context との連携

**UserPreferencesUpdated** (User → Learning/Progress)

- **目的**: ユーザー設定が更新されたことを通知
- **ペイロード**:
  - user_id: ユーザーID
  - updated_preferences: 更新された設定項目
  - effective_date: 適用開始日時

**UserAchievementUnlocked** (Progress → User)

- **目的**: ユーザーが実績を解除したことを通知
- **ペイロード**:
  - user_id: ユーザーID
  - achievement_id: 実績ID
  - achievement_type: 実績タイプ
  - unlocked_at: 解除日時

## イベント設計の原則

### 1. 疎結合性

- 各コンテキストは他のコンテキストの内部実装に依存しない
- イベントには必要最小限の情報のみを含める
- 受信側で必要な追加情報は、自身のコンテキストから取得

### 2. 非同期性

- すべての Integration Event は非同期で処理
- イベントの順序性は保証されない前提で設計
- 冪等性を考慮した実装

### 3. 耐障害性

- イベントの配信失敗を考慮したリトライ機構
- Dead Letter Queue による失敗イベントの管理
- 監視とアラートの実装

### 4. バージョニング

- イベントスキーマの後方互換性を維持
- 新しいフィールドは optional として追加
- 既存フィールドの削除は避ける
