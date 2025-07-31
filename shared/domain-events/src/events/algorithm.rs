//! Learning Algorithm Context イベント

use chrono::{DateTime, Utc};
use common_types::{ItemId, UserId};
use serde::{Deserialize, Serialize};

use crate::EventMetadata;

/// Learning Algorithm Context のイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningAlgorithmEvent {
    /// 復習スケジュールが更新された
    ReviewScheduleUpdated {
        /// イベントメタデータ
        metadata:         EventMetadata,
        /// ユーザー ID
        user_id:          UserId,
        /// 項目 ID
        item_id:          ItemId,
        /// 次回復習予定日
        next_review_date: DateTime<Utc>,
        /// 復習間隔（日数）
        interval_days:    u32,
    },
    /// 学習統計が更新された
    StatisticsUpdated {
        /// イベントメタデータ
        metadata:       EventMetadata,
        /// ユーザー ID
        user_id:        UserId,
        /// 総項目数
        total_items:    usize,
        /// 習得済み項目数
        mastered_items: usize,
    },
}
