//! Progress コマンド

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Progress コマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProgressCommand {
    /// 学習開始
    StartLearning { user_id: Uuid },

    /// アイテム完了記録
    RecordItemCompletion {
        user_id:            Uuid,
        vocabulary_item_id: Uuid,
        accuracy:           f32,
        time_spent:         i32,
    },

    /// セッション完了記録
    RecordSessionCompletion {
        user_id:       Uuid,
        items_count:   i32,
        correct_count: i32,
        duration:      i32,
    },

    /// ストリーク更新
    UpdateStreak { user_id: Uuid, new_streak: i32 },

    /// アチーブメント解除
    UnlockAchievement {
        user_id:        Uuid,
        achievement_id: String,
    },

    /// デイリーゴール完了
    CompleteDailyGoal { user_id: Uuid },

    /// 日次リセット
    ResetDaily { user_id: Uuid },

    /// 週次リセット
    ResetWeekly { user_id: Uuid },
}
