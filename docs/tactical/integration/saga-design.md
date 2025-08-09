# Saga パターンの使用機会

## 概要

Effect プロジェクトにおいて、Saga パターンを適用できる機会を分析しました。
Saga パターンは、マイクロサービス環境での分散トランザクション管理に適しており、
本プロジェクトのアーキテクチャ学習目的に合致します。

## Saga パターンとは

Saga パターンは、複数のサービスにまたがる長時間実行トランザクションを管理するパターンです。
各ステップが成功または失敗し、失敗時には補償トランザクション（compensating transaction）を
実行してシステムを一貫した状態に戻します。

## 実装機会

### 1. AI 生成タスクの補償処理（最推奨）

新しい語彙項目を追加する際の、外部 AI サービスとの連携処理。

```
開始: 新しい語彙項目の追加リクエスト
↓
Step 1: Vocabulary Context - 基本情報で VocabularyEntry 作成
↓
Step 2: AI Integration Context - 詳細情報（意味、例文、発音）を生成
↓
Step 3: Vocabulary Context - AI 生成結果で VocabularyEntry を更新
↓
完了: 充実した語彙項目が登録される

補償フロー（AI 生成失敗時）:
- Step 3 補償: スキップ（更新が行われていないため）
- Step 2 補償: エラーログ記録、リトライ情報の保存
- Step 1 補償: VocabularyEntry を削除または基本情報のみの状態に戻す
```

**選定理由:**

- 外部サービス（AI API）との連携で失敗リスクが高い
- 補償処理が明確で実装しやすい
- 実践的な学習価値が高い

### 2. 学習セッション完了の分散処理

複数のコンテキストを横断する学習セッション完了処理。

```
開始: 学習セッション完了
↓
Step 1: Learning Context - セッション結果を保存
↓
Step 2: Learning Algorithm Context - 各項目のSM-2アルゴリズム更新
        (イベント: CorrectnessJudgedを受信して処理)
↓
Step 3: AI Integration Context - 次回の推奨項目を生成（オプション）
↓
完了: 全コンテキストで整合性の取れた状態

補償フロー（いずれかのステップ失敗時）:
- Step 3 補償: AI生成タスクをキャンセル
- Step 2 補償: アルゴリズム更新をロールバック
- Step 1 補償: セッションを未完了ステータスに戻す

※ Progress Context はイベント駆動で自動更新されるため、
Sagaのステップには含まない（補償処理不要）
```

**特徴:**

- 複数コンテキストの協調が必要
- データ整合性が重要
- イベントソーシングとの組み合わせ学習

### 3. 新規ユーザーのオンボーディング

新規ユーザー登録時の初期データセットアップ。

```
開始: 新規ユーザー登録
↓
Step 1: User Context - プロファイル作成
↓
Step 2: Learning Algorithm Context - 初期学習記録セットアップ
        (新規ユーザー用のデフォルトSM-2パラメータ設定)
↓
Step 3: AI Integration Context - レベルチェックや推奨項目生成
↓
完了: ユーザーが学習を開始できる状慎

補償フロー（失敗時）:
- Step 3 補償: AIタスクをキャンセル
- Step 2 補償: 学習記録を削除
- Step 1 補償: ユーザープロファイルを削除

※ Progress Context は UserCreated イベントを受信して
自動的に初期統計を生成するため、Sagaに含まない
```

**特徴:**

- 一連の初期化処理をトランザクショナルに実行
- ユーザー体験の向上
- 部分的な状態を防ぐ

### 4. 項目の一括インポート（将来機能）

CSV やその他のソースから複数項目を一括登録。

```
開始: インポートファイルのアップロード
↓
Step 1: ファイル検証とパース
↓
Step 2: Vocabulary Context - 各項目の基本情報を登録
↓
Step 3: AI Integration Context - 各項目の詳細を並列生成
↓
Step 4: Learning Algorithm Context - 学習記録の初期化
↓
完了: 全項目が利用可能な状態

補償フロー:
- 失敗した項目のみロールバック
- 成功した項目は維持
- 部分的成功のレポート生成
```

**特徴:**

- 大量データの処理
- 部分的成功の許容
- 並列処理との組み合わせ

## 実装方針

### Saga パターンの種類

**Orchestration Saga** を採用します：

- **saga-executor** サービスが中央コーディネーターとして機能
- 各ステップの状態を永続化して管理
- 補償処理を明示的に定義
- 実装の見通しが良く、デバッグしやすい

### 技術選択

```rust
// Saga 定義の例
pub struct VocabularyEnrichmentSaga {
    saga_id: SagaId,
    state: SagaState,
    steps: Vec<SagaStep>,
    compensation_steps: Vec<CompensationStep>,
}

pub enum SagaState {
    Running,
    Completed,
    Compensating,
    Failed,
    Compensated,
}
```

### 実装優先順位

1. **Phase 1**: AI 生成タスクの Saga
   - 最も実用的で学習価値が高い
   - 外部サービス連携のパターンを学習

2. **Phase 2**: 学習セッション完了 Saga
   - 複数コンテキストの協調を学習
   - イベントソーシングとの統合

3. **Phase 3**: その他の Saga
   - 必要に応じて追加実装

## アーキテクチャ学習の観点

Saga パターンの実装を通じて以下を学習：

1. **分散トランザクション管理**
   - 2PC（Two-Phase Commit）の代替アプローチ
   - 結果整合性の実現

2. **補償処理の設計**
   - 各操作の逆操作を定義
   - 部分的失敗からの回復

3. **状態管理**
   - Saga の実行状態の永続化
   - 障害からの復旧

4. **イベント駆動との統合**
   - Saga のステップ完了をイベントとして発行
   - 他のパターンとの組み合わせ

## 関連ドキュメント

- `/services/saga-executor/` - Saga 実行サービス（実装予定）
- `/docs/ddd/design/event-storming-design-level/` - 各コンテキストの詳細設計

## 更新履歴

- 2025-07-29: 初版作成 - Saga パターンの使用機会を分析
