//! Learning Context イベント

use common_types::{ItemId, SessionId, UserId};
use serde::{Deserialize, Serialize};

use crate::EventMetadata;

/// Learning Context のイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningEvent {
    /// 学習セッションが開始された
    SessionStarted {
        /// イベントメタデータ
        metadata:   EventMetadata,
        /// セッション ID
        session_id: SessionId,
        /// ユーザー ID
        user_id:    UserId,
        /// 項目数
        item_count: usize,
    },
    /// 正誤判定が行われた
    CorrectnessJudged {
        /// イベントメタデータ
        metadata:         EventMetadata,
        /// セッション ID
        session_id:       SessionId,
        /// 項目 ID
        item_id:          ItemId,
        /// 判定結果
        judgment:         CorrectnessJudgment,
        /// 応答時間（ミリ秒）
        response_time_ms: u32,
    },
    /// 学習セッションが完了した
    SessionCompleted {
        /// イベントメタデータ
        metadata:        EventMetadata,
        /// セッション ID
        session_id:      SessionId,
        /// ユーザー ID
        user_id:         UserId,
        /// 完了項目数
        completed_count: usize,
        /// 正解数
        correct_count:   usize,
    },
}

/// 正誤判定
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CorrectnessJudgment {
    /// 正解
    Correct,
    /// 不正解
    Incorrect,
    /// スキップ
    Skipped,
}
