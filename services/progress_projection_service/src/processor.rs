use std::time::Duration;

use tokio::time;
use tracing::info;

use crate::config::Config;

pub async fn run(config: Config) -> crate::error::Result<()> {
    info!("イベントプロセッサー開始");
    info!("バッチサイズ: {}", config.processor.batch_size);
    info!("ポーリング間隔: {}ms", config.processor.poll_interval_ms);

    // イベント処理ループ
    let mut interval = time::interval(Duration::from_millis(config.processor.poll_interval_ms));

    loop {
        interval.tick().await;

        // TODO: 実装予定
        // 1. Event Store から未処理イベントを取得
        // 2. 各イベントを適切な Projection Handler に渡す
        // 3. Read Model を更新
        // 4. チェックポイントを更新

        // 現在は動作確認のためのダミー処理
        info!("イベント処理チェック（未実装）");
    }
}
