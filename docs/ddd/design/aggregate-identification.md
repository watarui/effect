# 集約の特定と境界

## 概要

このドキュメントでは、Effect プロジェクトにおける全ての集約を特定し、その境界と関係性を明確にします。
集約設計の妥当性を検証し、必要に応じて改善点を洗い出します。

## 集約設計の判断基準

### 1. 不変条件（Invariant）

- 一緒に変更される必要があるデータは同じ集約内に配置
- ビジネスルールを強制できる最小単位

### 2. トランザクション境界

- 1 つのトランザクションで変更するのは 1 つの集約のみ
- 複数集約の更新が必要な場合は、ドメインイベントで結果整合性を実現

### 3. 集約のサイズ

- 集約は小さく保つ（パフォーマンスと並行性のため）
- 大きくなりすぎた集約は分割を検討

### 4. 一貫性の境界

- 強い一貫性が必要：同じ集約内
- 結果整合性で十分：別集約 + イベント連携

## コンテキストごとの集約一覧

### 1. Learning Context（学習コンテキスト）

#### LearningSession（学習セッション）

- **集約ルート**: LearningSession
- **境界**: 1 回のテストセッション（最大 100 問、設定可能）
- **不変条件**:
  - セッション内の項目数は 1〜100 個（設定可能、デフォルト 100）
  - 各項目は 1 回だけ出題される
  - セッションの状態遷移は一方向（NotStarted → InProgress → Completed）
- **他集約との関係**:
  - UserItemRecord を item_id で参照（読み取りのみ）
  - User を user_id で参照
- **ライフサイクル**: セッション開始から完了まで（約 25 分）
- **設定**: 最大項目数は環境変数や設定ファイルで調整可能

#### UserItemRecord（ユーザー項目記録）

- **集約ルート**: UserItemRecord
- **境界**: ユーザー × 語彙項目の組み合わせ
- **不変条件**:
  - マスタリー状態は定義された遷移のみ許可
  - 3 回連続正解 + 平均反応時間 3 秒以内で短期マスター
  - 7 日後の再テストで正解なら長期マスター
- **他集約との関係**:
  - VocabularyItem を item_id で参照
  - User を user_id で参照
- **ライフサイクル**: 永続的（ユーザーが項目を学習している限り）

### 2. Vocabulary Context（語彙コンテキスト）

#### VocabularyEntry（語彙エントリ）

- **集約ルート**: VocabularyEntry
- **境界**: 同じスペリングを持つ全ての語彙項目
- **内部エンティティ**: VocabularyItem（複数）
- **不変条件**:
  - 同じエントリ内で item は一意の disambiguation を持つ
  - エントリの spelling は不変
  - version によるオプティミスティックロック
- **他集約との関係**:
  - AI Integration Context にイベント発行（情報生成要求）
- **ライフサイクル**: 永続的（グローバル辞書として）

### 3. Learning Algorithm Context（学習アルゴリズムコンテキスト）

#### ItemLearningRecord（項目学習記録）

- **集約ルート**: ItemLearningRecord
- **境界**: ユーザー × 語彙項目の学習状態（SM-2 アルゴリズム用）
- **不変条件**:
  - SM-2 アルゴリズムに従った interval と easiness_factor の更新
  - quality < 3 の場合は必ずリセット
  - easiness_factor は 1.3〜2.5 の範囲
- **他集約との関係**:
  - Learning Context からのイベントを受信
  - VocabularyItem を item_id で参照
- **ライフサイクル**: 永続的（学習記録として）

### 4. Progress Context（進捗コンテキスト）

#### 集約なし（純粋なイベントソーシング）

- **特徴**: Write Model を持たない、Read Model のみ
- **プロジェクション**:
  - DailyStatsProjection（日別統計）
  - CategoryProgressProjection（カテゴリ別進捗）
  - UserProgressSummaryProjection（全体サマリー）
  - LearningStreakProjection（ストリーク管理）
- **イベントソース**: 他のコンテキストからのドメインイベント
- **更新戦略**: リアルタイム、バッチ、遅延評価の使い分け

### 5. AI Integration Context（AI 統合コンテキスト）

#### AIGenerationTask（AI 生成タスク）

- **集約ルート**: AIGenerationTask
- **境界**: 1 回の AI 生成リクエスト
- **不変条件**:
  - タスクの状態遷移（Pending → Processing → Completed/Failed）
  - リトライ回数の上限
  - タイムアウト制約
- **他集約との関係**:
  - Vocabulary Context へのコールバック
  - User を user_id で参照
- **ライフサイクル**: タスク作成から完了/失敗まで（通常数秒〜数十秒）

#### ChatSession（チャットセッション）

- **集約ルート**: ChatSession
- **境界**: 1 つの深掘りチャットセッション
- **不変条件**:
  - メッセージは追加のみ（変更・削除不可）
  - セッションは特定の語彙項目に紐づく
- **他集約との関係**:
  - VocabularyItem を item_id で参照
  - User を user_id で参照
- **ライフサイクル**: セッション開始から終了まで（数分〜数十分）

### 6. User Context（ユーザーコンテキスト）

#### UserProfile（ユーザープロファイル）

- **集約ルート**: UserProfile
- **境界**: 1 人のユーザーとその設定
- **不変条件**:
  - email は一意
  - role の変更は Admin のみ可能
  - 最初のユーザーは自動的に Admin
- **他集約との関係**:
  - 全てのコンテキストから user_id で参照される
  - アカウント削除時は全コンテキストにイベント発行
- **ライフサイクル**: アカウント作成から削除まで

## 集約間の関係性

### ID による参照関係

```
UserProfile (user_id)
    ↓参照
    ├── LearningSession
    ├── UserItemRecord
    ├── ItemLearningRecord
    ├── AIGenerationTask
    └── ChatSession

VocabularyEntry/Item (item_id)
    ↓参照
    ├── UserItemRecord
    ├── ItemLearningRecord
    └── ChatSession
```

### イベントによる連携

```
Learning Context
    ├── → Learning Algorithm Context (CorrectnessJudged)
    └── → Progress Context (SessionCompleted, ItemMasteryUpdated)

Vocabulary Context
    └── → AI Integration Context (AIGenerationRequested)

Learning Algorithm Context
    └── → Progress Context (StatisticsUpdated)

User Context
    └── → All Contexts (AccountDeleted)
```

## 検証結果と改善提案

### 現状の良い点

1. **適切な境界設定**

   - 各集約は明確な境界と責任を持つ
   - トランザクション境界が明確

2. **小さな集約**

   - ほとんどの集約が適切なサイズ
   - 並行性の問題が起きにくい

3. **イベント駆動**
   - 集約間の結合度が低い
   - 結果整合性で十分な箇所は適切に分離

### 検証結果と設計の妥当性

1. **UserItemRecord と ItemLearningRecord の分離**

   - 両方とも「ユーザー × 項目」の学習状態を管理
   - **責務が明確に異なるため、現状の分離は妥当**
     - UserItemRecord：UI 表示用（マスタリー状態など）
     - ItemLearningRecord：SM-2 アルゴリズム計算用（次回出題タイミング）
   - 変更理由が異なる（UI 要求 vs アルゴリズム変更）

2. **Progress Context の集約なし設計**

   - 唯一集約を持たないコンテキスト
   - **CQRS の Query 側の典型的なパターンで適切**
     - 純粋な読み取り専用コンテキスト
     - 守るべき不変条件が存在しない
     - 「学習活動の鏡」としての役割に特化

3. **VocabularyEntry の軽量化設計**
   - VocabularyEntry は「インデックス」的役割（軽量）
   - VocabularyItem が実際の詳細データを保持（独立集約）
   - **複雑さを回避するために意図的に分離された良い設計**

## 結論

現在の集約設計は概ね適切であり、DDD の原則に従っています。
特に、小さな集約とイベント駆動の設計により、スケーラブルで保守しやすいシステムになっています。

次のステップとして、これらの集約に対するリポジトリとドメインサービスの設計に進むことができます。

## 更新履歴

- 2025-07-27: 初版作成（全コンテキストの集約を整理）
- 2025-07-28: 潜在的な改善点を検証し、現在の設計の妥当性を確認
