# Learning Algorithm Context Bounded Context Canvas

## 1. Name

Learning Algorithm Context

## 2. Purpose

科学的に実証された SM-2（SuperMemo 2）アルゴリズムを基盤として、個人の学習パターンに最適化された復習スケジュールと項目選定を実現する。
85% ルールに基づく動的な難易度調整により、学習効果を最大化する。

## 3. Strategic Classification

- **Domain Type**: Core Domain
- **Business Model**: Revenue Generator
- **Evolution Stage**: Custom Built

### 分類の理由

- **Core Domain**: 学習効果の最大化という本質的価値を提供。SM-2 アルゴリズムの効果的な実装と 85% ルールによる最適化が差別化要因
- **Revenue Generator**: 優れた学習アルゴリズムがユーザーの学習成果を向上させ、継続率とビジネス価値に直結
- **Custom Built**: 標準的な SM-2 に加え、独自の最適化（85% ルール、反応時間考慮）を実装

## 4. Domain Roles

- **Analysis Engine**: 学習データの分析と統計処理
- **Decision Engine**: 復習タイミングと項目選定の決定
- **Optimization Engine**: 個人の学習パターンに基づく最適化

### 役割の詳細

| 役割                | 説明                                     |
| ------------------- | ---------------------------------------- |
| Analysis Engine     | 正答率、反応時間、学習パターンを分析     |
| Decision Engine     | 次回復習日の計算、学習項目の優先順位付け |
| Optimization Engine | 85% ルールに基づく難易度調整、個人最適化 |

## 5. Inbound Communication

### メッセージ/イベント

| 名前                   | 送信元           | 契約タイプ | 説明                               |
| ---------------------- | ---------------- | ---------- | ---------------------------------- |
| CorrectnessJudged      | Learning Context | 非同期     | 項目の正誤判定結果（反応時間含む） |
| ItemSelectionRequested | Learning Context | 同期       | 学習項目の選定要求                 |
| SessionCompleted       | Learning Context | 非同期     | セッション完了通知（統計更新用）   |
| UserSettingsChanged    | User Context     | 非同期     | 学習設定の変更                     |

### 統合パターン

- Learning Context: Partnership（双方向の密接な協調）
- User Context: Customer-Supplier（User が Supplier）

## 6. Outbound Communication

### メッセージ/イベント

| 名前                    | 宛先             | 契約タイプ | 説明                       |
| ----------------------- | ---------------- | ---------- | -------------------------- |
| ItemsSelected           | Learning Context | 同期       | 選定された学習項目のリスト |
| ReviewScheduleUpdated   | Progress Context | 非同期     | 復習スケジュールの更新通知 |
| DifficultyAdjusted      | Progress Context | 非同期     | 難易度係数の調整通知       |
| LearningStatsCalculated | Progress Context | 非同期     | 計算された学習統計         |

### 統合パターン

- Learning Context: Partnership（ItemsSelected は同期、その他は非同期）
- Progress Context: Published Language（イベント公開）

## 7. Ubiquitous Language

### 主要な用語

| 用語               | 英語                    | 定義                                   |
| ------------------ | ----------------------- | -------------------------------------- |
| 難易度係数         | Easiness Factor (EF)    | 項目の学習難易度を表す係数（1.3〜2.5） |
| 復習間隔           | Review Interval         | 次回復習までの日数                     |
| 品質評価           | Quality Rating          | 回答の質（0-5）、反応時間を考慮        |
| 85% ルール         | 85% Rule                | 最適な学習効果のための目標正答率       |
| 連続正解数         | Repetition Count        | 連続して正解した回数                   |
| 項目選定戦略       | Item Selection Strategy | 復習/新規/弱点項目の選定ロジック       |
| 最適復習タイミング | Optimal Review Timing   | 忘却曲線に基づく最適な復習時期         |

### ドメインコンセプト

SM-2 アルゴリズムを基盤に、個人の学習パターンを分析し、最適な復習タイミングと項目選定を行う。85% の正答率を維持することで、適切な認知負荷と学習効果のバランスを実現する。

## 8. Business Decisions

### 主要なビジネスルール

1. 初回復習間隔は 1 日、その後は EF に基づいて計算
2. 品質評価 3 未満の場合、復習間隔をリセット
3. セッションの正答率が 85% から大きく乖離する場合、難易度を動的調整
4. 期限切れ項目（復習予定日を過ぎた項目）を優先的に選定
5. 85% ルールは固定値として適用（ユーザー設定不可）

### ポリシー

- **科学的根拠**: エビデンスベースの学習理論に基づく
- **個人最適化**: 画一的でなく個人の学習パターンに適応
- **透明性**: アルゴリズムの動作を説明可能に

### アルゴリズム詳細

```
SM-2 計算式:
- I(1) = 1
- I(2) = 6
- I(n) = I(n-1) * EF （n > 2）

EF 更新式:
- EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
- EF は 1.3 以上に制限

品質評価（q）の決定:
- 5: 即座に正解（< 1秒）
- 4: 素早く正解（< 2秒）
- 3: 正解（< 5秒）
- 2: 正解だが遅い（>= 5秒）
- 1: 不正解だが惜しい
- 0: 完全に不正解
```

## 9. Assumptions

### 技術的前提

- 学習履歴データへの高速アクセスが可能
- 統計計算のための十分な計算リソース
- リアルタイムでの項目選定が可能（100ms 以内）

### ビジネス的前提

- ユーザーは定期的に学習を継続する
- SM-2 アルゴリズムは英語学習にも有効
- 85% ルールは全レベルのユーザーに適用可能
- 反応時間は理解度の有効な指標

## 10. Verification Metrics

### 定量的指標

| メトリクス       | 目標値     | 測定方法                         |
| ---------------- | ---------- | -------------------------------- |
| 平均正答率       | 85% ± 5%   | 全セッションの正答率分布         |
| 記憶定着率       | 80% 以上   | LongTerm 状態の項目数 / 全項目数 |
| 項目選定応答時間 | 100ms 以内 | API レスポンスタイム             |
| アルゴリズム精度 | 90% 以上   | 実際の復習間隔と最適間隔の一致率 |

### 定性的指標

- アルゴリズムの透明性と説明可能性
- 個人差への適応性
- 学習負荷の適切性（ユーザーフィードバック）

## 11. Open Questions

### 設計上の疑問

- [ ] 複数デバイス間での学習履歴の同期方法は？
- [ ] グループ学習や比較機能を追加する場合の設計は？
- [ ] EF の初期値をユーザーレベルに応じて調整すべきか？
- [ ] 学習中断が多いユーザーへの対応は？

### 実装上の課題

- [ ] 大量の学習履歴データの効率的な処理方法は？
- [ ] リアルタイム統計計算のパフォーマンス最適化は？
- [ ] A/B テストによるアルゴリズム改善の仕組みは？
- [ ] 機械学習による更なる最適化の余地は？

---

## 改訂履歴

- 2025-07-29: 初版作成
- 2025-07-29: ItemsSelected を同期通信として定義
  - 理由：Learning Context との密接な連携により、即座の応答が必要
  - Learning Context Canvas の変更と整合性を保つ
