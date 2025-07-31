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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CorrectnessJudgment {
    /// 正解
    Correct,
    /// 不正解
    Incorrect,
    /// スキップ
    Skipped,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learning_event_should_have_required_fields() {
        let event = LearningEvent::SessionStarted {
            metadata:   EventMetadata::new(),
            session_id: SessionId::new(),
            user_id:    UserId::new(),
            item_count: 50,
        };

        match event {
            LearningEvent::SessionStarted { item_count, .. } => {
                assert_eq!(item_count, 50);
            },
            _ => unreachable!("Expected SessionStarted event"),
        }
    }

    #[test]
    fn correctness_judgment_should_be_comparable() {
        assert_eq!(CorrectnessJudgment::Correct, CorrectnessJudgment::Correct);
        assert_ne!(CorrectnessJudgment::Correct, CorrectnessJudgment::Incorrect);
    }

    #[test]
    fn session_completed_should_track_statistics() {
        let event = LearningEvent::SessionCompleted {
            metadata:        EventMetadata::new(),
            session_id:      SessionId::new(),
            user_id:         UserId::new(),
            completed_count: 50,
            correct_count:   45,
        };

        match event {
            LearningEvent::SessionCompleted {
                completed_count,
                correct_count,
                ..
            } => {
                assert_eq!(completed_count, 50);
                assert_eq!(correct_count, 45);
                assert!(correct_count <= completed_count);
            },
            _ => unreachable!("Expected SessionCompleted event"),
        }
    }
}
