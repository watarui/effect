# User Context Bounded Context Canvas

## 1. Name

User Context

## 2. Purpose

認証プロバイダーを抽象化した認証基盤を提供し、ユーザープロファイルと学習設定を管理する。
現在は Firebase Auth + Google OAuth を実装として使用するが、将来的に他の認証プロバイダーへの切り替えや追加が容易な設計とする。
シンプルな権限管理（Admin/User）により、システム全体のアクセス制御を実現する。

## 3. Strategic Classification

- **Domain Type**: Generic Subdomain
- **Business Model**: Cost Reducer
- **Evolution Stage**: Commodity

### 分類の理由

- **Generic Subdomain**: 認証・認可は一般的な機能であり、ビジネスの差別化要因ではない
- **Cost Reducer**: Firebase Auth を活用することで開発・運用コストを削減
- **Commodity**: 標準的な OAuth 認証とプロファイル管理機能

## 4. Domain Roles

- **Authentication Manager**: 認証プロバイダーの抽象化と管理
- **Identity Provider**: ユーザー認証と識別
- **Access Control**: 権限管理とアクセス制御
- **Profile Manager**: ユーザー情報と設定の管理

### 役割の詳細

| 役割                    | 説明                                                           |
| ----------------------- | -------------------------------------------------------------- |
| Authentication Manager  | AuthenticationPort インターフェースによる認証の抽象化           |
| Identity Provider       | 認証プロバイダー（現在は Firebase Auth）を通じた認証           |
| Access Control         | Admin/User のシンプルな権限管理                                |
| Profile Manager        | 学習目標、難易度設定などの基本情報管理                         |

## 5. Inbound Communication

### メッセージ/イベント

| 名前                      | 送信元               | 契約タイプ | 説明                           |
| ------------------------- | -------------------- | ---------- | ------------------------------ |
| SignUpRequest             | Frontend/API Gateway | 同期       | 新規ユーザー登録               |
| SignInRequest             | Frontend/API Gateway | 同期       | ユーザーログイン               |
| UpdateProfileRequest      | Frontend/API Gateway | 同期       | プロファイル更新               |
| UpdateLearningGoalRequest | Frontend/API Gateway | 同期       | 学習目標の更新                 |
| DeleteAccountRequest      | Frontend/API Gateway | 同期       | アカウント削除要求             |

### 統合パターン

- Frontend/API Gateway: Direct Call（同期的な API 呼び出し）
- すべて同期的な要求/応答パターン

## 6. Outbound Communication

### メッセージ/イベント

| 名前                        | 宛先              | 契約タイプ | 説明                             |
| --------------------------- | ----------------- | ---------- | -------------------------------- |
| UserSignedUp                | All Contexts      | 非同期     | 新規ユーザー登録の通知           |
| UserSignedIn                | Progress Context  | 非同期     | ログインの通知（統計用）         |
| ProfileUpdated              | Learning Context  | 非同期     | プロファイル更新の通知           |
| LearningGoalSet             | Algorithm Context | 非同期     | 学習目標設定の通知               |
| AccountDeleted              | All Contexts      | 非同期     | アカウント削除（カスケード削除） |

### 統合パターン

- All Contexts: Published Language（イベント発行）
- 認証後は UserId で他コンテキストと連携

## 7. Ubiquitous Language

### 主要な用語

| 用語             | 英語                | 定義                                   |
| ---------------- | ------------------- | -------------------------------------- |
| ユーザーID       | User ID             | システム全体で一意のユーザー識別子     |
| 権限             | Role                | Admin または User の権限レベル         |
| 学習目標         | Learning Goal       | IELTS スコアや CEFR レベルの目標       |
| 難易度設定       | Difficulty Preference | 学習コンテンツの難易度（A1-C2）      |
| アカウント状態   | Account Status      | Active または Deleted                  |
| プロバイダー     | OAuth Provider      | 認証プロバイダー（Google）            |

### ドメインコンセプト

**ポート&アダプターアーキテクチャ**により認証を抽象化：

1. **AuthenticationPort**: 認証操作の抽象インターフェース
   - `authenticate`: 認証処理
   - `verify_token`: トークン検証
   - `refresh_token`: トークン更新
   - `revoke_access`: アクセス取り消し

2. **FirebaseAuthAdapter**: Firebase 実装（現在のアダプター）
   - Google OAuth による認証
   - Firebase Admin SDK でのトークン検証
   - Anti-Corruption Layer として Firebase 固有の詳細を隠蔽

3. **将来の拡張性**
   - 新しい認証プロバイダーはアダプターを追加するだけ
   - ドメインロジックは変更不要
   - 複数プロバイダーの並行運用も可能

ユーザープロファイルは最小限の情報のみ保持し、複雑な機能は他のコンテキストに委譲する。

## 8. Business Decisions

### 主要なビジネスルール

1. 認証プロバイダーは抽象化されているが、現在は Google OAuth のみサポート
2. 最初に登録したユーザーが自動的に Admin になる
3. Admin は他のユーザーの権限を変更可能（自分以外）
4. アカウント削除は論理削除（deleted_at フィールド）
5. デフォルトの難易度は B1（中級）
6. 認証プロバイダーの切り替えは設定変更のみで対応可能

### ポリシー

- **抽象化優先**: 認証プロバイダーへの依存を最小化
- **シンプル優先**: 認証の詳細は認証プロバイダーに委譲
- **プライバシー重視**: 最小限の情報のみ保持
- **論理削除**: データは保持しつつ deleted_at で管理

### 権限ルール（RBAC: Role-Based Access Control）

```
Admin権限:
- すべてのユーザーデータを閲覧可能
- 他のユーザーの権限を変更可能
- システム設定の変更（将来実装）

User権限:
- 自分のデータのみアクセス可能
- 自分のプロファイルのみ編集可能
```

現在はシンプルな2ロール構成だが、これも RBAC の一種である。

## 9. Assumptions

### 技術的前提

- 認証プロバイダーインターフェースが実装されている
- 少なくとも 1 つの認証アダプター（現在は Firebase）が利用可能
- OAuth トークンが有効
- ユーザー数は少数（家族数名程度）
- 高度なセキュリティ要件なし

### ビジネス的前提

- ユーザーは Google アカウントを持っている
- 複雑な権限管理は不要
- ユーザー登録後すぐに学習を開始できる
- プロファイル情報の変更は稀

## 10. Verification Metrics

### 定量的指標

| メトリクス         | 目標値      | 測定方法                      |
| ------------------ | ----------- | ----------------------------- |
| 認証成功率         | 99.9% 以上  | 成功ログイン数 / 試行数        |
| 認証応答時間       | 200ms 以内  | Firebase Auth のレスポンス時間 |
| プロファイル取得時間 | 50ms 以内   | DB クエリ時間                  |
| トークン検証時間   | 100ms 以内  | JWT 検証時間                   |

### 定性的指標

- 認証フローのシンプルさ
- エラーメッセージの分かりやすさ
- 権限管理の適切性
- Firebase 依存の抽象化レベル

## 11. Open Questions

### 設計上の疑問

- [x] 認証プロバイダーの抽象化は実装済み
- [ ] メール/パスワード認証の需要はあるか？
- [ ] より細粒度の権限管理（リソース別、操作別）が必要になるか？
- [ ] プロファイルに追加すべき情報はあるか？
- [ ] 複数の認証プロバイダーの同時利用（同一ユーザー）を許可するか？

### 実装上の課題

- [x] 認証プロバイダーの切り替え可能な設計
- [ ] 認証プロバイダーのオフライン時の対応は？
- [ ] トークンのキャッシュ戦略は？
- [ ] Admin ユーザーが存在しない場合の対応は？
- [ ] 削除されたユーザーの再登録は許可するか？

---

## 改訂履歴

- 2025-07-30: 初版作成
- 2025-08-09: 認証プロバイダー抽象化の設計を追加
