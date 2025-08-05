# Google Pub/Sub エミュレータ セットアップガイド

## 概要

Effect プロジェクトでは、イベント駆動アーキテクチャの実装に Google Pub/Sub を使用しています。ローカル開発環境では Google Pub/Sub エミュレータを使用することで、クラウドサービスに接続することなく開発・テストが可能です。

## セットアップ手順

### 1. 環境変数の設定

`.env.example` をコピーして `.env` ファイルを作成します：

```bash
cp .env.example .env
```

以下の環境変数が設定されていることを確認します：

```bash
# Google Pub/Sub エミュレータ
PUBSUB_EMULATOR_PORT=8085
PUBSUB_PROJECT_ID=effect-local
```

### 2. Docker Compose での起動

プロジェクトルートで以下のコマンドを実行します：

```bash
docker compose up -d pubsub
```

エミュレータが正常に起動したか確認：

```bash
docker compose ps pubsub
```

### 3. エミュレータの動作確認

エミュレータが正常に動作していることを確認：

```bash
curl http://localhost:8085
```

正常な場合、以下のようなレスポンスが返ります：

```
Ok
```

## 使用方法

### アプリケーションからの接続

各マイクロサービスは自動的に以下の環境変数を使用してエミュレータに接続します：

- `PUBSUB_EMULATOR_HOST`: pubsub:8085（Docker ネットワーク内）
- `GOOGLE_CLOUD_PROJECT`: effect-local

### トピックとサブスクリプション

アプリケーションは自動的に必要なトピックとサブスクリプションを作成します。各 Bounded Context ごとに以下のトピックが作成されます：

- `effect-learning`: Learning Context のイベント
- `effect-vocabulary`: Vocabulary Context のイベント
- `effect-user`: User Context のイベント
- `effect-progress`: Progress Context のイベント
- `effect-algorithm`: Learning Algorithm Context のイベント
- `effect-ai`: AI Integration Context のイベント

## トラブルシューティング

### エミュレータが起動しない場合

1. ポート 8085 が他のプロセスで使用されていないか確認：

   ```bash
   lsof -i :8085
   ```

2. Docker のログを確認：

   ```bash
   docker compose logs pubsub
   ```

### 接続できない場合

1. 環境変数が正しく設定されているか確認：

   ```bash
   docker compose exec learning-service env | grep PUBSUB
   ```

2. ネットワークの接続性を確認：

   ```bash
   docker compose exec learning-service ping pubsub
   ```

## 本番環境への移行

本番環境では実際の Google Cloud Pub/Sub サービスを使用します：

1. Google Cloud プロジェクトの作成
2. Pub/Sub API の有効化
3. サービスアカウントの作成と認証設定
4. 環境変数の更新（`PUBSUB_EMULATOR_HOST` を削除）

詳細は [Google Cloud Pub/Sub ドキュメント](https://cloud.google.com/pubsub/docs) を参照してください。

## 参考リンク

- [Google Pub/Sub エミュレータ公式ドキュメント](https://cloud.google.com/pubsub/docs/emulator)
- [google-cloud-pubsub Rust クレート](https://docs.rs/google-cloud-pubsub/latest/google_cloud_pubsub/)
