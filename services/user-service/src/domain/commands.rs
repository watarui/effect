//! ユーザー関連のコマンド

use common_types::UserId;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{
    learning_goal::LearningGoal,
    user_profile::CefrLevel,
    user_role::UserRole,
};

/// ユーザー作成コマンド
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateUser {
    /// Email アドレス
    pub email:         String,
    /// 表示名
    pub display_name:  String,
    /// 最初のユーザーかどうか（Admin 権限付与の判定用）
    pub is_first_user: bool,
}

/// プロフィール更新コマンド
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateUserProfile {
    /// 対象ユーザー ID
    pub user_id:               UserId,
    /// 新しい表示名（None の場合は変更なし）
    pub display_name:          Option<String>,
    /// 新しい現在レベル（None の場合は変更なし）
    pub current_level:         Option<CefrLevel>,
    /// 1セッションあたりの問題数（None の場合は変更なし）
    pub questions_per_session: Option<u8>,
}

/// ロール変更コマンド
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeUserRole {
    /// 対象ユーザー ID
    pub user_id:     UserId,
    /// 新しいロール
    pub new_role:    UserRole,
    /// 実行者のユーザー ID（管理者権限の確認用）
    pub executed_by: UserId,
}

/// Email 更新コマンド
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateUserEmail {
    /// 対象ユーザー ID
    pub user_id:   UserId,
    /// 新しい Email アドレス
    pub new_email: String,
}

/// ユーザー削除コマンド
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteUser {
    /// 対象ユーザー ID
    pub user_id:     UserId,
    /// 実行者のユーザー ID（本人または管理者）
    pub executed_by: UserId,
}

/// 学習目標設定コマンド
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetLearningGoal {
    /// 対象ユーザー ID
    pub user_id: UserId,
    /// 学習目標
    pub goal:    Option<LearningGoal>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_user_command_should_serialize() {
        // Given
        let command = CreateUser {
            email:         "test@example.com".to_string(),
            display_name:  "Test User".to_string(),
            is_first_user: false,
        };

        // When
        let json = serde_json::to_string(&command).unwrap();
        let deserialized: CreateUser = serde_json::from_str(&json).unwrap();

        // Then
        assert_eq!(command, deserialized);
    }

    #[test]
    fn update_user_profile_command_with_partial_updates() {
        // Given
        let user_id = UserId::new();
        let command = UpdateUserProfile {
            user_id,
            display_name: Some("New Name".to_string()),
            current_level: None,
            questions_per_session: Some(25),
        };

        // Then
        assert_eq!(command.display_name, Some("New Name".to_string()));
        assert_eq!(command.current_level, None);
        assert_eq!(command.questions_per_session, Some(25));
    }
}
