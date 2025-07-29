# Bounded Context Canvas テンプレート

## 概要

このテンプレートは [DDD-Crew の Bounded Context Canvas](https://github.com/ddd-crew/bounded-context-canvas) に基づいています。
Bounded Context Canvas は、Domain-Driven Design における境界づけられたコンテキストを設計・文書化するための協調ツールです。

## Bounded Context Canvas とは

### 目的

Bounded Context Canvas は、DDD における境界づけられたコンテキストの重要な側面を体系的に探索し、文書化するための協調ツールです。チーム全体で共通理解を形成し、設計の意思決定を明確にすることを目的としています。

### 主要な構成要素

Canvas は以下の主要セクションで構成されています：

1. **Name（名前）**

   - コンテキストの明確な名前
   - チーム全体で合意された、一貫性のある名称

2. **Purpose（目的）**

   - ビジネス価値と主要な責務の簡潔な説明
   - このコンテキストが存在する理由

3. **Strategic Classification（戦略的分類）**

   - **Domain type（ドメインタイプ）**:
     - Core（コアドメイン）
     - Supporting（支援ドメイン）
     - Generic（汎用サブドメイン）
   - **Business model（ビジネスモデル）**:
     - Revenue generator（収益創出）
     - Engagement creator（エンゲージメント創出）
     - Compliance enforcer（コンプライアンス遵守）
     - Cost reducer（コスト削減）
   - **Evolution stage（進化段階）**:
     - Genesis（創世記 - 新規開発中、実験的）
     - Custom built（カスタム構築 - 独自実装、差別化要因）
     - Product（製品 - 標準化された製品、市場で入手可能）
     - Commodity（コモディティ - 汎用品、どこでも入手可能）

4. **Domain Roles（ドメインの役割）**

   - コンテキストの振る舞いを特徴づける役割
   - 例：
     - Analysis context（分析コンテキスト）
     - System context（システムコンテキスト）
     - Gateway context（ゲートウェイコンテキスト）
     - Execution context（実行コンテキスト）
     - Coordination context（調整コンテキスト）

5. **Inbound Communication（インバウンド通信）**

   - 受信するメッセージ
   - 協力者（他のコンテキストやシステム）
   - 関係タイプ（同期/非同期、パートナーシップ/カスタマー・サプライヤー等）

6. **Outbound Communication（アウトバウンド通信）**

   - 送信するメッセージ
   - 宛先
   - 通信パターン

7. **Ubiquitous Language（ユビキタス言語）**

   - 主要なドメイン用語と定義
   - このコンテキスト内で使用される共通言語

8. **Business Decisions（ビジネス決定）**

   - 重要なビジネスルールとポリシー
   - ドメインロジックの中核となる判断基準

9. **Assumptions（前提条件）**

   - 既知の未知についての明示的な記述
   - 設計時点での仮定事項

10. **Verification Metrics（検証メトリクス）**

    - 設計を測定・検証する方法
    - 成功の指標

11. **Open Questions（未解決の質問）**
    - 設計に関する未解決の疑問
    - 今後検討が必要な事項

## Canvas テンプレート

以下のテンプレートを使用して、各 Bounded Context の Canvas を作成してください。

```md
# [Context Name] Bounded Context Canvas

## 1. Name

[コンテキストの正式名称]

## 2. Purpose

[このコンテキストの存在意義とビジネス価値を 1-2 文で説明]

## 3. Strategic Classification

- **Domain Type**: [Core Domain / Supporting Domain / Generic Subdomain]
- **Business Model**: [Revenue Generator / Engagement Creator / Compliance Enforcer / Cost Reducer]
- **Evolution Stage**: [Genesis / Custom Built / Product / Commodity]

### 分類の理由

[なぜこの分類になるのかを説明]

## 4. Domain Roles

- [役割 1: 説明]
- [役割 2: 説明]

### 役割の詳細

| 役割                 | 説明                             |
| -------------------- | -------------------------------- |
| Execution Context    | 実際の処理を実行する             |
| Analysis Context     | データを分析・集計する           |
| Gateway Context      | 外部との境界を管理する           |
| System Context       | システム的な処理を担当する       |
| Coordination Context | 複数のコンテキスト間の調整を行う |

## 5. Inbound Communication

### メッセージ/イベント

| 名前           | 送信元               | 契約タイプ    | 説明         |
| -------------- | -------------------- | ------------- | ------------ |
| [メッセージ名] | [送信元コンテキスト] | [同期/非同期] | [簡単な説明] |

### 統合パターン

- [Customer-Supplier / Partnership / Shared Kernel / etc.]

## 6. Outbound Communication

### メッセージ/イベント

| 名前           | 宛先               | 契約タイプ    | 説明         |
| -------------- | ------------------ | ------------- | ------------ |
| [メッセージ名] | [宛先コンテキスト] | [同期/非同期] | [簡単な説明] |

### 統合パターン

- [Customer-Supplier / Partnership / Published Language / etc.]

## 7. Ubiquitous Language

### 主要な用語

| 用語     | 英語    | 定義   |
| -------- | ------- | ------ |
| [用語 1] | [英語1] | [定義] |
| [用語 2] | [英語2] | [定義] |

### ドメインコンセプト

[このコンテキストの中核となる概念の説明]

## 8. Business Decisions

### 主要なビジネスルール

1. [ルール 1]
2. [ルール 2]

### ポリシー

- [ポリシー 1: 説明]
- [ポリシー 2: 説明]

## 9. Assumptions

### 技術的前提

- [前提 1]
- [前提 2]

### ビジネス的前提

- [前提 1]
- [前提 2]

## 10. Verification Metrics

### 定量的指標

目標値は、このコンテキストが期待通りに機能しているかを判断するための基準値です。実際の運用で継続的に測定し、設計の妥当性を検証します。

| メトリクス | 目標値 | 測定方法   |
| ---------- | ------ | ---------- |
| [指標 1]   | [目標] | [測定方法] |
| [指標 2]   | [目標] | [測定方法] |

### 定性的指標

- [指標 1: 評価方法]
- [指標 2: 評価方法]

## 11. Open Questions

以下は、現時点で未解決の質問や検討事項です。これらは今後の設計・実装フェーズで順次検討し、解決していく必要があります。

### 設計上の疑問

- [ ] [質問 1]
- [ ] [質問 2]

### 実装上の課題

- [ ] [課題 1]
- [ ] [課題 2]
```
