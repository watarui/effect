//! User Context イベント

use common_types::UserId;
use serde::{Deserialize, Serialize};

use crate::EventMetadata;

/// User Context のイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEvent {
    /// アカウントが作成された
    AccountCreated {
        /// イベントメタデータ
        metadata: EventMetadata,
        /// ユーザー ID
        user_id:  UserId,
        /// メールアドレス
        email:    String,
    },
    /// アカウントが削除された
    AccountDeleted {
        /// イベントメタデータ
        metadata: EventMetadata,
        /// ユーザー ID
        user_id:  UserId,
    },
}
