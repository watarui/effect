//! AI Integration Context イベント

use common_types::EventId;
use serde::{Deserialize, Serialize};

use crate::EventMetadata;

/// AI Integration Context のイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
// Task プレフィックスは意図的：ドメインイベントとして「タスク」の概念を明確に表現
#[allow(clippy::enum_variant_names)]
pub enum AIIntegrationEvent {
    /// AI タスクが作成された
    TaskCreated {
        /// イベントメタデータ
        metadata:  EventMetadata,
        /// タスク ID
        task_id:   EventId,
        /// タスクタイプ
        task_type: String,
    },
    /// AI タスクが開始された
    TaskStarted {
        /// イベントメタデータ
        metadata: EventMetadata,
        /// タスク ID
        task_id:  EventId,
        /// AI プロバイダー
        provider: String,
    },
    /// AI タスクが完了した
    TaskCompleted {
        /// イベントメタデータ
        metadata:    EventMetadata,
        /// タスク ID
        task_id:     EventId,
        /// 処理時間（ミリ秒）
        duration_ms: u64,
    },
    /// AI タスクが失敗した
    TaskFailed {
        /// イベントメタデータ
        metadata:    EventMetadata,
        /// タスク ID
        task_id:     EventId,
        /// エラーメッセージ
        error:       String,
        /// リトライ回数
        retry_count: u32,
    },
}
