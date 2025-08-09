# クロスコンテキスト イベントフロー

## 概要

異なる Bounded Context 間で共有・連携されるイベントのカタログです。これらのイベントは Integration Events として、Context 間の疎結合な連携を実現します。

## Context 間の関係マップ

```
Vocabulary ─────┬─────> Learning (項目データ提供)
    │           │
    │           └─────> AI Integration (生成要求)
    
Learning ───────┬─────> Progress (学習イベント送信)
    │           │
    │           └─────> Learning Algorithm (正誤判定送信)
    │
    └───────────────< Learning Algorithm (最適項目選択)

Learning Algorithm ────> Progress (アルゴリズム結果送信)

AI Integration ────────> Vocabulary (生成結果)

User ──────────────────> 全コンテキスト (ユーザーID参照)

Progress (Read Model) ─> GraphQL API (データ提供のみ)
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

### 2. Learning → Learning Algorithm Context

**ItemSelectionRequested**

- **目的**: 学習セッション用の最適な項目選択を要求
- **ペイロード**:
  - user_id: ユーザーID
  - session_type: セッションタイプ（新規/復習）
  - max_items: 最大項目数
  - request_timestamp: リクエストタイムスタンプ

**ItemsSelected** (応答)

- **目的**: 選択された項目リストを返答
- **ペイロード**:
  - user_id: ユーザーID
  - selected_items: 項目IDのリスト（優先度順）
  - selection_criteria: 選択基準の説明

### 3. Learning → Progress Context / Learning Algorithm Context

**LearningSessionCompleted**

- **目的**: 学習セッションが完了したことを通知
- **ペイロード**:
  - session_id: セッションID
  - user_id: ユーザーID
  - session_type: セッションタイプ
  - results: セッション結果のサマリー
  - item_results: 個別項目の結果リスト

**CorrectnessJudged** (Learning → Progress & Learning Algorithm)

- **目的**: 個別項目の正誤判定を通知
- **送信先**: Progress Context と Learning Algorithm Context の両方
- **ペイロード**:
  - user_id: ユーザーID
  - item_id: 項目ID
  - session_id: セッションID
  - correctness: 正誤判定（correct/incorrect/timeout）
  - response_time_ms: 回答時間（ミリ秒）
  - timestamp: 判定時刻

### 4. Learning Algorithm → Progress Context

**ReviewRecorded**

- **目的**: SM-2 アルゴリズムの計算結果を通知
- **ペイロード**:
  - user_id: ユーザーID
  - item_id: 項目ID
  - easiness_factor: 更新された難易度係数
  - repetition_count: 復習回数
  - interval_days: 次回復習までの間隔
  - next_review_date: 次回復習予定日
  - quality: 品質評価（0-5）

### 5. Vocabulary ↔ AI Integration Context

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

### 6. User Context との連携

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
