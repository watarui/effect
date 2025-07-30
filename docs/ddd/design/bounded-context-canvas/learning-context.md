# Learning Context Bounded Context Canvas

## 1. Name

Learning Context

## 2. Purpose

ハイブリッド UI（解答表示 →3 秒自動確認）による「覚えた感」を実現し、効果的な学習体験を提供する。
25 分のポモドーロ単位で学習セッションを管理し、ユーザーの学習プロセスを最適化する。

## 3. Strategic Classification（戦略的分類）

- **Domain Type**: Core Domain
- **Business Model**: Engagement Creator
- **Evolution Stage**: Custom Built

### 分類の理由

- **Core Domain**: 学習体験の中核であり、ビジネス価値の源泉。ハイブリッド UI という独自の学習方式が他の英語学習アプリとの最大の差別化要因
- **Engagement Creator**: ユーザーの学習継続と「覚えた感」の実現によりエンゲージメントを創出
- **Custom Built**: 独自の学習フローと UI パターンを実装しており、既製品では代替不可能

## 4. Domain Roles

- **Execution Context**: 学習セッションの実行と進行管理
- **Gateway Context**: ユーザーインターフェースとドメインロジックの境界
- **Coordination Context**: 他のコンテキスト（Algorithm、Vocabulary）との協調

### 役割の詳細

| 役割                 | 説明                                                  |
| -------------------- | ----------------------------------------------------- |
| Execution Context    | 学習セッションの開始から完了までのワークフローを実行  |
| Gateway Context      | UI からの入力を受け取り、適切なドメインイベントに変換 |
| Coordination Context | 項目選定や正誤判定結果を他コンテキストと連携          |

## 5. Inbound Communication

### メッセージ/イベント

| 名前          | 送信元                     | 契約タイプ | 説明                                     |
| ------------- | -------------------------- | ---------- | ---------------------------------------- |
| ItemsSelected | Learning Algorithm Context | 同期       | 学習する項目のリストを受信               |
| UserSettings  | User Context               | 同期       | ユーザーの学習設定（目標、難易度）を取得 |
| ItemDetails   | Vocabulary Context         | 同期       | 項目の詳細情報（意味、例文）を取得       |

### 統合パターン

- Learning Algorithm Context: Customer-Supplier（Algorithm が Supplier）
- User Context: Customer-Supplier（User が Supplier）
- Vocabulary Context: Partnership（相互依存）

## 6. Outbound Communication（アウトバウンド通信）

### メッセージ/イベント

| 名前                   | 宛先                       | 契約タイプ | 説明                       |
| ---------------------- | -------------------------- | ---------- | -------------------------- |
| SessionStarted         | Progress Context           | 非同期     | セッション開始を通知       |
| SessionCompleted       | Progress Context           | 非同期     | セッション完了と結果を通知 |
| CorrectnessJudged      | Learning Algorithm Context | 非同期     | 項目の正誤判定結果を通知   |
| ItemSelectionRequested | Learning Algorithm Context | 同期       | 新しい項目の選定を要求     |

### 統合パターン

- Progress Context: Published Language（イベント公開）
- Learning Algorithm Context: Partnership（双方向の協調）

## 7. Ubiquitous Language

### 主要な用語

| 用語            | 英語             | 定義                                                    |
| --------------- | ---------------- | ------------------------------------------------------- |
| 学習セッション  | Learning Session | 最大 100 問（設定可能）の学習単位、約 25 分のポモドーロ |
| ハイブリッド UI | Hybrid UI        | 解答表示後 3 秒で自動的に「わかった」と判定する UI 方式 |
| 項目提示        | Item Presentation | 学習項目をユーザーに表示すること                        |
| 正誤判定        | Correctness Judgment | ユーザーの理解度を判定すること                          |
| 覚えた感        | Sense of Mastery | ユーザーが項目を習得したと実感できる状態                |
| マスタリー状態  | Mastery Status   | 項目の習得レベル（Unknown→Tested→ShortTerm→LongTerm）   |

### ドメインコンセプト

学習セッションは、ユーザーが集中して取り組める 25 分単位で設計され、ハイブリッド UI により効率的な学習を実現する。「覚えた感」を重視し、短期記憶から長期記憶への移行を支援する。

## 8. Business Decisions

### 主要なビジネスルール

1. 1 セッションは最大 100 問（ユーザー設定可能）
2. 解答表示後 3 秒経過で自動的に「わかった」と判定
3. ユーザーは 3 秒以内に「わからなかった」を選択可能
4. セッション中断時は進捗を保存しない（完了のみ記録）
5. セッションは約 25 分（ポモドーロ単位）を想定

### ポリシー

- **即座のフィードバック**: 正誤判定は即座にユーザーに伝える
- **ポジティブな学習体験**: 「覚えた感」を演出する UI/UX
- **集中の維持**: セッション中の中断を最小限に

## 9. Assumptions

### 技術的前提

- ユーザーは安定したインターネット接続を持つ
- ブラウザは最新の JavaScript をサポート
- レスポンスタイムは 100ms 以内

### ビジネス的前提

- ユーザーは集中して学習に取り組む意欲がある
- 3 秒は適切なデフォルト確認時間である
- 25 分は適切な学習セッション長である
- ユーザーは正直に「わからなかった」を申告する

## 10. Verification Metrics

### 定量的指標

| メトリクス       | 目標値      | 測定方法                            |
| ---------------- | ----------- | ----------------------------------- |
| セッション完了率 | 80%以上     | 完了セッション数 / 開始セッション数 |
| 平均正答率       | 85%以上     | 正解数 / 総問題数                   |
| 平均応答時間     | 2 秒以内    | 項目提示から判定までの時間          |
| 継続率           | 週 3 回以上 | アクティブユーザー数の推移          |

### 定性的指標

- ユーザー満足度: アンケートによる「覚えた感」の評価
- 学習効果: 長期記憶への移行率
- UI の使いやすさ: ユーザビリティテスト

## 11. Open Questions

### 設計上の疑問

- [ ] 3 秒のタイミングは調整可能にすべきか？（ユーザー設定）
- [ ] セッション中断時の部分保存は必要か？
- [ ] 学習モード（速習/じっくり）の追加は必要か？
- [ ] 項目の難易度をリアルタイムで調整すべきか？

### 実装上の課題

- [ ] オフライン対応はどこまで必要か？
- [ ] 音声読み上げ機能の実装優先度は？
- [ ] マルチデバイス間のセッション同期は必要か？
- [ ] レスポンスタイムの精度をどこまで求めるか？

---

## 改訂履歴

- 2025-07-29: 初版作成
- 2025-07-29: ItemsSelected を非同期から同期に変更
  - 理由：UX の一貫性を優先（学習開始時に即座に項目リストが必要）
  - 非同期パターンの学習は他の箇所（Progress Context のイベントソーシング、ドメインイベント発行）で十分に実践可能
  - アーキテクチャの適材適所（全てを非同期にする必要はない）という設計判断も重要な学習要素
- 2025-07-30: ItemSelectionRequested を非同期から同期に変更
  - 理由：Learning Algorithm Context との整合性を保つ
  - セッション中の項目選定は即座の応答が必要
