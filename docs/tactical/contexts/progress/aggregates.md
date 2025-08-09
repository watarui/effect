# Progress Context - 集約設計

## 概要

Progress Context は純粋な CQRS/Event Sourcing アーキテクチャを採用しており、従来の意味での「集約」は存在しません。すべてのデータは他のコンテキストから発行されたイベントから導出される投影（Read Model）として管理されます。

## 集約を持たない理由

- **Write Model 不要**: 自身でビジネスロジックを持たない
- **イベント受信専用**: 他コンテキストのイベントを集約するのみ
- **投影中心**: すべてのデータはイベントから生成される派生データ

## 投影（Read Model）の概要

Progress Context で管理される主要な投影：

### 基本統計

- **DailyStats**: 日別学習統計
- **WeeklyStats**: 週別傾向分析
- **MonthlyStats**: 月別サマリー

### カテゴリ別統計

- **ItemStats**: 個別項目の学習状況
- **DomainStats**: R/W/L/S 別の進捗
- **LevelStats**: CEFR レベル別の習熟度

### 特殊統計

- **StreakProjection**: 連続学習記録
- **MilestoneProjection**: マイルストーン達成状況
- **PerformanceProjection**: パフォーマンス指標

## イベントからの投影生成

各投影はイベントハンドラーによって更新されます：

1. **イベント受信**: 他コンテキストからのドメインイベント
2. **投影更新**: 該当する Read Model を更新
3. **キャッシュ無効化**: 関連するキャッシュをクリア

この設計により、新しい統計視点を後から追加することが容易になります。
