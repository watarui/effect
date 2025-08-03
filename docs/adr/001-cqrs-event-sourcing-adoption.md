# ADR 001: CQRS と Event Sourcing の採用

## ステータス

**採択** (2025-08-03)

## コンテキスト

Effect プロジェクトは、英単語学習アプリケーションとして、以下の要件を持っています：

1. **高度な語彙管理**: Wikipedia スタイルの意味管理、AI による自動生成、協調編集
2. **学習進捗の追跡**: SM-2 アルゴリズムによる学習スケジュール、詳細な学習履歴
3. **スケーラビリティ**: 将来的な成長に対応できるアーキテクチャ
4. **学習機会**: マイクロサービス、DDD、最新のアーキテクチャパターンの実践

現在の実装は従来の CRUD ベースであり、以下の課題があります：

- 複雑なドメインロジックとデータアクセスが混在
- 読み取りと書き込みで同じモデルを使用（最適化の余地）
- 完全な履歴管理が困難
- 並行編集の処理が複雑

## 決定

Effect プロジェクトにおいて、**CQRS (Command Query Responsibility Segregation)** と **Event Sourcing** を採用します。

### 採用の範囲

1. **フル Event Sourcing を適用する Context**:
   - Vocabulary Context: 語彙データの完全な変更履歴が必要
   - Progress Context: 学習履歴の追跡と分析が重要

2. **シンプル CQRS を適用する Context**:
   - Learning Context: テスト生成・採点のリクエスト・レスポンス処理
   - User Context: 基本的な CRUD 操作で十分
   - Learning Algorithm Context: SM-2 アルゴリズムの計算処理
   - AI Integration Context: 外部サービスとの連携が中心

### 実装方針

1. **マルチサービス構成**:
   - Command Service: Write Model の管理、イベント生成
   - Query Service: Read Model の提供
   - Projection Service: イベントから Read Model への変換
   - Search Service: 特殊な検索要件への対応（Vocabulary のみ）

2. **技術スタック**:
   - Event Store: PostgreSQL（イベント永続化）
   - Event Bus: Google Pub/Sub（イベント配信）
   - Read Model Storage: PostgreSQL + Redis + Meilisearch
   - 実装言語: Rust（全サービス統一）

## 結果

### 正の結果

1. **完全な監査証跡**: すべての変更が Event として記録される
2. **最適化された読み取り**: 用途別の Read Model により高速な検索・表示
3. **スケーラビリティ**: Command と Query を独立してスケール可能
4. **時系列分析**: Event Stream により学習パターンの分析が容易
5. **学習価値**: 実践的な CQRS/ES の経験を獲得

### 負の結果

1. **複雑性の増加**:
   - サービス数が増加（6 Context × 平均 3 サービス）
   - 結果整合性の管理が必要
   - デバッグが複雑

2. **運用負荷**:
   - 多数のサービスの監視・管理
   - Event Store のバックアップ・リストア
   - スキーマ進化の管理

3. **開発工数**:
   - 初期実装に時間がかかる
   - テストが複雑（統合テスト、イベント順序）

### リスク軽減策

1. **段階的移行**: Vocabulary Context から開始し、経験を積んでから他へ展開
2. **充実したドキュメント**: ADR、Event Catalog、Service 設計書を整備
3. **ローカル開発環境**: Docker Compose で全サービスを簡単に起動
4. **監視とトレーシング**: Google Cloud Trace による分散トレーシング

## 代替案

### 代替案 1: 単一サービス内 CQRS

各 Bounded Context を単一サービスとして実装し、内部で CQRS を適用。

**却下理由**:

- 学習機会が限定的
- 実際の企業環境と乖離
- スケーラビリティの制約

### 代替案 2: 従来の CRUD アーキテクチャ

現在の実装を改善し、CRUD ベースで継続。

**却下理由**:

- 複雑な要件（履歴、並行編集）への対応が困難
- 学習価値が低い
- 将来の拡張性に制限

### 代替案 3: Event Sourcing なしの CQRS

CQRS のみ採用し、Event Sourcing は使用しない。

**却下理由**:

- 完全な履歴管理ができない
- Read Model の再構築が困難
- 並行編集の処理が複雑なまま

## 参考資料

- [Martin Fowler - CQRS](https://martinfowler.com/bliki/CQRS.html)
- [Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
- [Greg Young - CQRS Documents](https://cqrs.files.wordpress.com/2010/11/cqrs_documents.pdf)
- [Microservices.io - Event Sourcing](https://microservices.io/patterns/data/event-sourcing.html)

## 更新履歴

- 2025-08-03: 初版作成
- 2025-08-03: Context リスト修正（Notification → Learning Algorithm）、技術スタック更新（Kafka → Pub/Sub、Elasticsearch → Meilisearch）
