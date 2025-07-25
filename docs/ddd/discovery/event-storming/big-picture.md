# EventStorming - Big Picture

## セッション情報

- **日付**: 2025-07-25
- **参加者**: 開発者（ドメインエキスパート兼）、AI アシスタント
- **目的**: Effect の全体的なビジネスフローを理解し、主要なイベントを特定する

## タイムライン

### 1. ユーザー登録から初回学習まで

```
[ユーザーが登録した] → [プロフィールが作成された] → [初期設定が完了した]
                                                    ↓
[チュートリアルが表示された] ← [学習目標が設定された]
        ↓
[最初の単語セットが選択された] → [初回学習セッションが開始された]
```

### 2. 日常的な学習フロー

```
[復習リマインダーが送信された] → [ユーザーがアプリを開いた]
                            ↓
                    [今日の学習単語が準備された]
                            ↓
[学習セッションが開始された] → [問題が表示された] → [回答が送信された]
        ↓                                          ↓
[フィードバックが表示された] ← [回答が評価された]
        ↓
[次の問題が表示された] → ... → [セッションが完了した]
                                    ↓
                        [学習統計が更新された]
                                    ↓
                        [次回復習日が計算された]
```

### 3. 単語管理フロー

```
[新しい単語が登録された] → [単語情報が入力された] → [単語が保存された]
                                            ↓
                                    [単語が分類された]
                                            ↓
[他のユーザーが単語を編集した] → [編集履歴が記録された]
        ↓
[単語情報が更新された] → [変更が通知された]
```

### 4. 進捗確認フロー

```
[進捗画面が開かれた] → [統計データが集計された] → [グラフが表示された]
                                            ↓
                                    [達成バッジが獲得された]
                                            ↓
                                    [SNSに共有された]
```

## 主要なドメインイベント

### 🟠 ドメインイベント（オレンジ付箋）

#### User Domain

- ユーザーが登録した
- プロフィールが更新された
- 学習目標が設定された
- 設定が変更された

#### Learning Domain

- 学習セッションが開始された
- 問題が生成された
- 回答が送信された
- 回答が評価された
- セッションが完了した
- 復習スケジュールが更新された

#### Word Domain

- 単語が登録された
- 単語情報が更新された
- 例文が追加された
- 単語が分類された
- 単語がお気に入りに追加された

#### Progress Domain

- 学習記録が作成された
- 統計が更新された
- ストリークが更新された
- マイルストーンが達成された
- レポートが生成された

## ホットスポット 🔥

### 1. 学習アルゴリズムの精度

**問題**: SM-2 アルゴリズムが個人差に対応できているか
**議論**: パラメータの調整機能が必要か

### 2. 単語の品質管理

**問題**: 協調編集による誤った情報の混入
**議論**: レビュープロセスの必要性

### 3. モチベーション維持

**問題**: 学習の継続が困難
**議論**: ゲーミフィケーション要素の追加

### 4. スケーラビリティ

**問題**: ユーザー増加時のパフォーマンス
**議論**: 非同期処理の必要性

## 外部システムとの連携

### 📥 入力

- OAuth プロバイダー（Google, GitHub）
- 外部辞書 API
- 音声合成 API（Google TTS）

### 📤 出力

- メール通知
- プッシュ通知
- 分析データのエクスポート

## ポリシー（ビジネスルール）

### 💡 主要なポリシー

1. **復習タイミングポリシー**

   - 「回答の正確性に基づいて次回復習日を決定する」
   - トリガー: 回答が評価された

2. **単語選択ポリシー**

   - 「復習期限と新規単語のバランスを取る」
   - トリガー: 学習セッションが開始された

3. **ストリーク管理ポリシー**

   - 「24 時間以内に学習すればストリーク継続」
   - トリガー: セッションが完了した

4. **難易度調整ポリシー**
   - 「正答率に基づいて単語の難易度を調整」
   - トリガー: 統計が更新された

## 時間的制約

### ⏰ タイムアウト

- セッション開始後 60 分で自動終了
- 回答待ち 30 秒でスキップ
- 統計集計は日次バッチ

### 📅 スケジュール

- 復習リマインダー: 毎日設定時刻
- 週次レポート: 日曜日
- 月次サマリー: 月末

## 発見された境界

### 🔲 潜在的な境界づけられたコンテキスト

1. **Learning Context**

   - 学習セッション管理
   - アルゴリズム実行
   - 問題生成

2. **Word Management Context**

   - 単語 CRUD
   - 分類管理
   - 協調編集

3. **User Context**

   - 認証・認可
   - プロフィール管理
   - 設定管理

4. **Progress Context**

   - 統計集計
   - レポート生成
   - 達成管理

5. **Notification Context**
   - リマインダー
   - 通知配信
   - スケジュール管理

## 次のステップ

1. **Design Level EventStorming**

   - 各コンテキストの詳細なフロー
   - コマンド、アグリゲートの特定

2. **ユビキタス言語の精緻化**

   - 用語の明確な定義
   - チーム内での合意形成

3. **コンテキストマップの作成**
   - 境界の確定
   - 統合パターンの選択

## 振り返り

### うまくいったこと

- 全体的なフローが明確になった
- 主要なイベントを網羅的に特定できた
- ホットスポットが明確になった

### 改善点

- 実際のユーザーフィードバックがない
- 非機能要件の議論が不足
- エッジケースの検討が必要

## 更新履歴

- 2025-07-25: 初回セッション実施
