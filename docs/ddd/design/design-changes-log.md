# DDD 設計変更記録

## 概要

このドキュメントは、Effect プロジェクトの DDD 設計プロセスにおける変更内容と、各設計文書間の不整合を管理するための記録です。
Canvas 作成など新しい設計作業を進めながら、過去の成果物との整合性を保つために使用します。

作成日: 2025-07-29

## 変更履歴

### 2025-07-31: 学習セッションを 1 問刻みの問題数ベースに統一

- **変更内容**: 学習セッションを純粋な問題数ベースに変更、ポモドーロへの言及を削除
- **決定事項**:
  - セッション単位：1-100 問（1 問単位で自由設定）
  - デフォルト：50 問
  - よく使う設定：25、50、75 問（UI での提案）
  - ポモドーロ・時間への言及を削除
- **理由**:
  - 最もシンプルな設計（制約なし）
  - 完全なユーザーの自由度
  - 実装の単純性（余計なバリデーション不要）
  - SRS 分野のベストプラクティス（Anki 等の成功事例）
- **影響範囲**:
  - ubiquitous-language.md：学習セッションの定義（L100-127）
  - learning-context Canvas：Purpose（L10）、用語（L76）、ビジネスルール（L91,削除 L95）、前提（削除 L114）
  - event-storming-design-level/learning-context.md：主要な責務（L9）、集約説明（L18）、SessionConfig（L141）、バリデーション（L509）

### 2025-07-30: PlantUML 自動画像生成の GitHub Actions 設定

- **変更内容**: PlantUML ファイルから SVG 画像を自動生成する CI/CD パイプライン構築
- **決定事項**:
  - `.github/workflows/generate-plantuml.yml` を作成
  - main ブランチへの push/PR マージで自動実行
  - 各 `.puml` ファイルと同じディレクトリ内の `svg/` サブディレクトリに SVG を保存
  - 例: `context-map.puml` → `svg/context-map.svg`
- **理由**:
  - PlantUML をインストールしていない環境でも図を確認可能
  - 常に最新の図が自動的に生成される
  - GitHub 上で直接 SVG を表示できる
- **技術詳細**:
  - PlantUML v1.2024.0 を使用
  - [skip ci] タグで無限ループを防止
  - github-actions[bot] として自動コミット

### 2025-07-30: 戦略的 DDD ドキュメントの再構成

- **変更内容**: DDD ドキュメントの重複解消と責務の明確化
- **決定事項**:
  - context-map.md を簡素化（図と統合パターン一覧のみに）
  - domain-classification.md を新規作成（Core/Supporting/Generic の分類を統合）
  - shared-kernel.md を新規作成（共有概念を独立管理）
  - bounded-contexts.md を簡素化（詳細は Canvas に委譲）
  - domain-types.md を削除（内容を domain-classification.md に統合）
- **理由**:
  - 各ドキュメントの責務を明確化し、重複を解消
  - Single Source of Truth の原則に従う
  - メンテナンス性の向上
- **影響範囲**:
  - /docs/ddd/strategic/ 配下のドキュメント構成が変更
  - 関連ドキュメントへの参照パスの更新が必要

### 2025-07-30: AI Integration Context の完全非同期化

- **変更内容**: AI Integration Context を完全非同期処理（イベント駆動アーキテクチャ）に変更
- **決定事項**:
  - 全ての AI 要求を非同期処理に変更（即座にタスク ID を返却）
  - WebSocket/SSE によるリアルタイム進捗通知を実装
  - TaskQueue 集約を追加してタスク管理を強化
  - ProcessingMode を最初から Asynchronous に設定
- **理由**:
  - より良いユーザー体験（待ち時間中も他の操作が可能）
  - スケーラビリティの向上（大量の AI 要求を並列処理）
  - エラー耐性の向上（タスクキューによるリトライ管理）
  - 学習を妨げない（AI 生成中も学習セッションを継続可能）
- **影響範囲**:
  - bounded-context-canvas/ai-integration-context.md を更新
  - event-storming-design-level/ai-integration-context.md を更新
  - Vocabulary Context、Learning Context との統合パターンが変更
  - リポジトリ設計に TaskQueue の永続化が必要

### 2025-07-30: AI Integration Context 非同期化に伴う整合性修正

- **変更内容**: 非同期化に伴う関連ドキュメントの整合性修正
- **決定事項**:
  - AI Integration Context リポジトリ設計に TaskQueueRepository と NotificationRepository を追加
  - Vocabulary Context に TaskCreatedAck とタスク ID 管理を追加
  - Learning Context に AI 機能の将来拡張を Open Questions として記載
  - context-map.md の統合パターンを Event-Driven に更新
- **理由**:
  - 非同期処理の実装に必要な永続化設計を明確化
  - タスク ID による非同期処理の追跡を可能に
  - WebSocket/SSE によるリアルタイム通知の基盤を整備
- **影響範囲**:
  - repositories/ai-integration-context-repositories.md: 3 つのリポジトリインターフェースを定義
  - bounded-context-canvas/vocabulary-context.md: タスク管理の追記
  - bounded-context-canvas/learning-context.md: 将来拡張の明記
  - strategic/context-map.md: Event-Driven Partnership への更新

### 2025-07-30: Progress Context Canvas 作成と IELTS スコア推定の見直し

- **変更内容**: Progress Context の Bounded Context Canvas を作成
- **決定事項**:
  - IELTS スコア推定を実装対象から除外（Open Questions へ移動）
  - 代替指標として CEFR レベル分布と進捗スコア（0-100）を採用
  - 純粋な CQRS/イベントソーシングの実装例として位置付け
- **理由**:
  - アーキテクチャ学習の本質に集中するため
  - 実装の複雑さを軽減
  - CEFR レベルと進捗スコアで学力認識は十分可能
- **影響範囲**:
  - event-storming-design-level/progress-context.md との不整合が発生
  - GraphQL スキーマの修正が必要

### 2025-07-29: Learning Algorithm Context Canvas 作成

- **変更内容**: Learning Algorithm Context の Bounded Context Canvas を作成
- **決定事項**:
  - ItemsSelected は同期通信として確定（Learning Context との整合性）
  - Partnership パターンの詳細を定義
  - SM-2 アルゴリズムの詳細仕様を文書化
  - 85% ルールによる動的調整を明確化
- **影響範囲**:
  - 項目選定ロジックの実装方針
  - Learning Context との統合設計

### 2025-07-29: ItemsSelected の同期化

- **変更内容**: ItemsSelected を非同期から同期に変更
- **理由**: UX の一貫性を優先（学習開始時に即座に項目リストが必要）
- **決定場所**: Learning Context Bounded Context Canvas
- **影響範囲**:
  - Learning Context と Learning Algorithm Context の統合パターン
  - イベントフローの設計
  - 実装時の通信方式

### 2025-07-27: Learning Algorithm Context の追加

- **変更内容**: 6 つ目のコンテキストとして Learning Algorithm Context を追加
- **理由**: 学習アルゴリズムの責務を明確に分離
- **影響範囲**:
  - 全体のコンテキストマップ
  - Learning Context の責務範囲

### 2025-07-xx: Progress Context の責務簡素化

- **変更内容**: Progress Context を純粋なイベントソーシングに特化
- **理由**: 統計計算は各コンテキストに委譲し、責務を明確化
- **影響範囲**:
  - Progress Context の設計
  - 統計情報の算出責任の移譲

## 未反映の変更リスト

### 高優先度

~~0. **AI Integration Context の非同期化に伴う関連更新**~~ （2025-07-30 完了）

    ~~- 対象: `/docs/ddd/design/repositories/ai-integration-context-repositories.md`~~
    ~~- 内容: TaskQueueRepository の追加、非同期処理のための永続化設計~~
    ~~- 理由: TaskQueue 集約の永続化が必要~~

~~1. **IELTS スコア推定の除外**~~ （2025-07-30 完了）

    ~~- 対象: `/docs/ddd/design/event-storming-design-level/progress-context.md`~~
    ~~- 内容: IeltsEstimation 関連のコード・ロジックを削除または Open Questions へ移動~~
    ~~- 理由: Canvas での決定事項を反映~~

~~2. **ItemsSelected の同期化**~~ （2025-07-30 完了）

    ~~- 対象: `/docs/ddd/design/event-storming-design-level/learning-context.md`~~
    ~~- 対象: `/docs/ddd/design/event-storming-design-level/learning-algorithm-context.md`~~
    ~~- 内容: 非同期イベントから同期 API 呼び出しに変更~~

~~3. **コンテキスト間の関係パターン**~~ （2025-07-30 完了）

    ~~- 対象: `/docs/ddd/strategic/context-map.md`~~
    ~~- 内容: Learning Context と Learning Algorithm Context の関係を Partnership に更新~~
    ~~- 内容: AI Integration Context との関係を Event-Driven に更新~~

~~4. **イベント名の統一**~~ （2025-07-30 完了）

    ~~- 対象: 全 event-storming-design-level ドキュメント~~
    ~~- 内容: 命名規則の統一（例: SessionStarted → LearningSessionStarted）~~
    - 追記: 冗長性を避けるため、プレフィックスを削除し、代わりに DomainEvent wrapper を追加

### 中優先度

~~4. **集約の責務説明の明確化**~~ （既に対応済み）

    ~~- 対象: `/docs/ddd/design/aggregate-identification.md`~~
    ~~- 内容: UserItemRecord と ItemLearningRecord の責務の違いを明確に説明~~
    - 注記: aggregate-identification.md の 201-208 行目に既に明確な説明がある

~~5. **Progress Context の責務範囲**~~ （既に対応済み）

    ~~- 対象: `/docs/ddd/strategic/bounded-contexts.md`~~
    ~~- 内容: 最新の設計（純粋なイベントソーシング）に合わせて更新~~
    - 注記: 27行目に既に「CQRS/Event Sourcing パターンを採用」と記載

~~6. **AI Integration Context の戦略的分類**~~ （既に対応済み）

    ~~- 対象: `/docs/ddd/strategic/context-map.md`~~
    ~~- 内容: Generic Subdomain として統一~~
    - 注記: context-map.md と domain-classification.md で既に Generic として分類

### 低優先度

~~7. **ドメイン用語の統一**~~ （2025-07-30 完了）

    ~~- 対象: 全ドキュメント~~
    ~~- 内容:~~
        ~~- 「項目」→「Item」に統一~~
        ~~- 「覚えた感」→「Sense of Mastery」に統一~~
        ~~- MasteryStatus の値を統一~~
    - 注記: 調査の結果、「項目（Item）」は既に適切に定義済み、「覚えた感」に英語表記を追加、MasteryStatus は統一済み

~~8. **ユビキタス言語の更新**~~ （2025-07-30 完了）

    ~~- 対象: `/docs/ddd/discovery/ubiquitous-language.md`~~
    ~~- 内容: 最新の用語定義に更新~~
    - 注記: ドメインイベントセクションを DomainEvent wrapper パターンに更新、「覚えた感」に「Sense of Mastery」を追加

## 追加で発見された不整合（2025-07-29）

### 高優先度

~~9. **Progress Context の設計不整合**~~ （既に対応済み）

    ~~- bounded-contexts.md：統計計算の責務あり（line 101-104）~~
    ~~- progress-context.md：純粋なイベントソーシング、集約なし（line 26-28）~~
    ~~- 内容：Progress Context は集約を持たない純粋な Read Model として統一すべき~~
    - 注記: bounded-contexts.md は既に簡素化され、Progress Context は「CQRS/Event Sourcing パターン」として正しく記載

~~10. **Vocabulary Context の設計アプローチ不整合**~~ （既に対応済み）

    ~~- bounded-contexts.md：単純な CRUD 操作（line 36）~~
    ~~- vocabulary-context.md：Wikipedia 方式、楽観的ロック、イベントソーシング（line 9-12）~~
    ~~- 内容：最新の Wikipedia 方式の設計に統一~~
    - 注記: bounded-contexts.md の15行目に既に「Wikipedia スタイル」と記載されている

~~11. **domain-types.md の存在**~~ （既に対応済み）

    ~~- 対象：`/docs/ddd/strategic/domain-types.md`~~
    ~~- 内容：このファイルは使用されていない（空またはテンプレート）~~
    ~~- アクション：削除または内容を追加~~
    - 注記: ファイルは既に削除済み、domain-classification.md が代替として機能

### 中優先度

~~12. **イベント名の不統一（詳細）**~~ （2025-07-30 完了）

    ~~- ubiquitous-language.md：`VocabularyItemRegistered`（line 238）~~
    ~~- context-map.md：`ItemRegistered`（line 257）~~
    ~~- vocabulary-context.md：異なるイベント体系~~
    ~~- 内容：プレフィックスルールを決定（例：`{Context}_{Action}`）~~
    - 注記: 調査の結果、DomainEvent wrapper パターンが既に採用済み。ubiquitous-language.md を実装に合わせて修正（ItemCreated、EntryCreated）

~~13. **セッション時間の不整合**~~ （2025-07-31 完了）

    ~~- ubiquitous-language.md：25 分のポモドーロ単位（line 101）~~
    ~~- learning-context.md：最大 100 問（設定可能）、約 25 分（line 18）~~
    ~~- 内容：時間ベースか問題数ベースかを明確化~~

~~14. **共有カーネルの定義場所**~~ （2025-07-31 確認済み）

    ~~- context-map.md：Shared Kernel セクションあり（line 210-221）~~
    ~~- 他のドキュメント：参照なし~~
    ~~- 内容：共有型の定義場所を統一、各コンテキストから参照~~
    - 注記: 調査の結果、shared-kernel.md が適切に存在し、context-map.md の L85 から参照されている。
    現状の構成が DDD の原則に合致しているため、変更不要と判断

15. **統合パターンの表記不統一**

- context-map.md：Customer-Supplier、Publisher-Subscriber など
- Canvas：同期/非同期の観点も含む
- 内容：統合パターンの表記方法を統一

### 低優先度

16. **CreatedBy の型定義不整合**

- ubiquitous-language.md：概念のみ（line 44）
- vocabulary-context.md：詳細な enum 定義（line 77-81）
- 内容：実装詳細をどこまで設計文書に含めるか統一

17. **認証方式の表記**

- User Context：Firebase/Google OAuth（複数箇所）
- 一部：Firebase Auth + Google OAuth
- 内容：表記を統一

18. **ドメインイベントのグルーピング**

- ubiquitous-language.md：ドメインごとにグループ化（line 231-256）
- context-map.md：コンテキストごとにグループ化（line 225-267）
- 内容：イベントの整理方法を統一

19. **更新履歴の記載方法**

- 一部：詳細な更新内容
- 一部：日付のみ
- 内容：更新履歴の記載レベルを統一

20. **マークダウンのコードブロック言語指定**

- 一部：`rust`
- 一部：言語指定なし
- 内容：コードブロックの言語指定を統一

## 更新対象ドキュメント一覧

| ドキュメント                                                                 | 更新内容                    | 優先度 | 備考           |
| ---------------------------------------------------------------------------- | --------------------------- | ------ | -------------- |
| `/docs/ddd/strategic/context-map.md`                                         | コンテキスト関係、AI の分類 | 高     | 全体像に影響   |
| `/docs/ddd/design/event-storming-design-level/learning-context.md`           | ItemsSelected の同期化      | 高     | 実装に直接影響 |
| `/docs/ddd/design/event-storming-design-level/learning-algorithm-context.md` | ItemsSelected の同期化      | 高     | 実装に直接影響 |
| `/docs/ddd/strategic/bounded-contexts.md`                                    | Progress Context の責務     | 中     | 概念理解に影響 |
| `/docs/ddd/design/aggregate-identification.md`                               | 集約の責務説明              | 中     | 設計理解に影響 |
| `/docs/ddd/discovery/ubiquitous-language.md`                                 | 用語の統一                  | 低     | 可読性向上     |
| 各 event-storming ドキュメント                                               | イベント名の統一            | 低     | 一貫性向上     |

## Canvas 完成後の更新計画

### Phase 1: 重要な概念の更新（1-2 時間）

1. context-map.md の更新

   - コンテキスト間の関係を最新化
   - 戦略的分類を統一

2. ItemsSelected の同期化
   - 関連する event-storming ドキュメントを更新
   - 統合パターンの説明を修正

### Phase 2: 責務と境界の明確化（1-2 時間）

3. bounded-contexts.md の更新

   - 各コンテキストの責務を最新化
   - 特に Progress Context の簡素化を反映

4. aggregate-identification.md の更新
   - 集約間の責務の違いを明確に説明

### Phase 3: 詳細の統一（1 時間）

5. イベント名の統一

   - 命名規則を決定
   - 全ドキュメントで統一

6. ドメイン用語の統一
   - ユビキタス言語の更新
   - 全ドキュメントで表記統一

## 設計原則の確認

この更新作業を通じて、以下の DDD 原則を維持します：

- **ユビキタス言語の一貫性**: チーム全体で同じ用語を使用
- **境界づけられたコンテキストの独立性**: 各コンテキストの自律性を保持
- **継続的な改善**: 設計は進化するものとして受け入れる
- **ドキュメントは生きている**: 実装と設計の乖離を防ぐ

## 未実装機能の検討事項

### メディアファイル管理（2025-07-30 追加）

- **対象**: Vocabulary Context
- **内容**: 画像ファイルと音声ファイルのフィールド追加
- **背景**: 必須要件だが、現在の設計には含まれていない
- **想定フィールド**:

  ```rust
  // VocabularyItem に追加
  image_url: Option<String>,        // イラスト画像のURL
  audio_url: Option<String>,        // 発音音声ファイルのURL
  thumbnail_url: Option<String>,    // サムネイル画像のURL
  ```

- **技術的検討事項**:
  - **ストレージ**: AWS S3 を想定（他の選択肢：Google Cloud Storage、Azure Blob）
  - **CDN 配信**: CloudFront で配信（キャッシュとパフォーマンス最適化）
  - **アクセス制御**:
    - 公開アクセス（CDN 経由）
    - または署名付き URL（期限付きアクセス）
  - **ファイル形式**:
    - 画像：WebP、JPEG、PNG
    - 音声：MP3、AAC（ブラウザ互換性を考慮）
  - **アップロード処理**:
    - 直接アップロード or API 経由
    - ファイルサイズ制限
    - ウイルススキャン
- **他コンテキストへの影響**:
  - AI Integration Context：画像生成後の保存処理
  - Learning Context：音声再生機能の実装
  - Progress Context：メディア利用統計の追跡
- **優先度**: 高（必須要件）
- **実装時期**: Canvas 完成後、技術選定フェーズで詳細設計

#### アーキテクチャ学習の観点からの推奨

**推奨：実装しない、または最小限の実装に留める**

**理由**：

1. **学習目標から外れる**

   - DDD/CQRS/Event Sourcing の本質的な学習には寄与しない
   - インフラ層の実装に時間を取られ、ドメインロジックの学習が疎かになる
   - ファイルアップロード処理は定型的で学習価値が低い

2. **実装の複雑さとコスト**

   - マルチパートアップロードの実装
   - ファイル検証、サイズ制限、ウイルススキャン
   - ストレージ料金、CDN 転送料金（少量なら無料枠内だが）
   - エラーハンドリングの複雑化

3. **代替案で十分**

   ```rust
   // 案1: 外部URLを直接参照（実装なし）
   image_url: Option<String>,  // "https://example.com/apple.jpg"

   // 案2: テキスト説明のみ（AI生成時に使用）
   image_description: Option<String>,  // "赤いリンゴのイラスト"
   pronunciation_guide: Option<String>,  // "/ˈæp.əl/" (IPA表記)
   ```

**もし実装する場合の最小限アプローチ**：

- アップロード機能なし（URL 直接入力のみ）
- CDN なし（Cloud Storage の公開 URL を直接利用）
- リサイズ・最適化処理なし
- 署名付き URL なし（公開アクセスのみ）

#### Google Cloud での実装（参考）

AWS の代わりに Google Cloud を使用する場合：

- **ストレージ**: Cloud Storage（S3 相当）
- **CDN**: Cloud CDN（CloudFront 相当）
- **アクセス制御**: IAM + 署名付き URL（同じ概念）
- **料金体系**: ほぼ同等（無料枠あり）

## メモ

- Canvas 作成中に発見した変更は、このドキュメントに随時追記する
- 大きな設計変更が発生した場合は、即座に影響範囲を評価する
- 実装フェーズに入る前に、必ず全ての高優先度項目を更新する
