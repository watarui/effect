//! User Context イベント

use chrono::{DateTime, Utc};
use common_types::UserId;
use serde::{Deserialize, Serialize};

use crate::EventMetadata;

// 一時的な型定義 - 後で common-types に移動予定
/// CEFR レベル（ヨーロッパ言語共通参照枠）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CefrLevel {
    /// 初級前半
    A1,
    /// 初級後半
    A2,
    /// 中級前半
    B1,
    /// 中級後半
    B2,
    /// 上級前半
    C1,
    /// 上級後半
    C2,
}

/// 学習目標（簡略版）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LearningGoal {
    /// IELTS スコア目標
    IeltsScore {
        /// 総合スコア（0.0-9.0）
        overall: f32,
    },
    /// TOEFL スコア目標
    ToeflScore {
        /// 総合スコア（0-120）
        total: u8,
    },
    /// TOEIC スコア目標
    ToeicScore {
        /// 総合スコア（10-990）
        total: u16,
    },
    /// 英検レベル目標（級名）
    EikenLevel(String),
    /// 一般的な CEFR レベル目標
    GeneralLevel(CefrLevel),
    /// 特定の目標なし
    NoSpecificGoal,
}

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
    /// プロフィールが更新された
    ProfileUpdated {
        /// イベントメタデータ
        metadata:              EventMetadata,
        /// ユーザー ID
        user_id:               UserId,
        /// 表示名
        display_name:          Option<String>,
        /// 現在の CEFR レベル
        current_level:         Option<CefrLevel>,
        /// 学習目標
        learning_goal:         Option<LearningGoal>,
        /// 1セッションあたりの問題数
        questions_per_session: Option<u8>,
        /// イベント発生日時
        occurred_at:           DateTime<Utc>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_updated_event_should_create_successfully() {
        // Given
        let user_id = UserId::new();
        let metadata = EventMetadata::new();
        let display_name = Some("Updated Name".to_string());
        let current_level = Some(CefrLevel::B2);
        let learning_goal = Some(LearningGoal::GeneralLevel(CefrLevel::C1));
        let questions_per_session = Some(30);
        let occurred_at = Utc::now();

        // When
        let event = UserEvent::ProfileUpdated {
            metadata: metadata.clone(),
            user_id,
            display_name: display_name.clone(),
            current_level,
            learning_goal: learning_goal.clone(),
            questions_per_session,
            occurred_at,
        };

        // Then
        match event {
            UserEvent::ProfileUpdated {
                metadata: event_metadata,
                user_id: event_user_id,
                display_name: event_display_name,
                current_level: event_current_level,
                learning_goal: event_learning_goal,
                questions_per_session: event_questions_per_session,
                occurred_at: event_occurred_at,
            } => {
                assert_eq!(event_metadata.event_id, metadata.event_id);
                assert_eq!(event_user_id, user_id);
                assert_eq!(event_display_name, display_name);
                assert_eq!(event_current_level, current_level);
                assert_eq!(event_learning_goal, learning_goal);
                assert_eq!(event_questions_per_session, questions_per_session);
                assert_eq!(event_occurred_at, occurred_at);
            },
            _ => unreachable!("Should have created ProfileUpdated event"),
        }
    }

    #[test]
    fn profile_updated_event_should_serialize_and_deserialize() {
        // Given
        let event = UserEvent::ProfileUpdated {
            metadata:              EventMetadata::new(),
            user_id:               UserId::new(),
            display_name:          Some("Test User".to_string()),
            current_level:         Some(CefrLevel::A2),
            learning_goal:         Some(LearningGoal::IeltsScore { overall: 7.0 }),
            questions_per_session: Some(25),
            occurred_at:           Utc::now(),
        };

        // When
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: UserEvent = serde_json::from_str(&json).unwrap();

        // Then
        match deserialized {
            UserEvent::ProfileUpdated {
                display_name,
                current_level,
                learning_goal,
                questions_per_session,
                ..
            } => {
                assert_eq!(display_name, Some("Test User".to_string()));
                assert_eq!(current_level, Some(CefrLevel::A2));
                assert_eq!(
                    learning_goal,
                    Some(LearningGoal::IeltsScore { overall: 7.0 })
                );
                assert_eq!(questions_per_session, Some(25));
            },
            _ => unreachable!("Deserialized event should be ProfileUpdated"),
        }
    }

    #[test]
    fn profile_updated_event_should_handle_none_fields() {
        // Given
        let event = UserEvent::ProfileUpdated {
            metadata:              EventMetadata::new(),
            user_id:               UserId::new(),
            display_name:          None,
            current_level:         None,
            learning_goal:         None,
            questions_per_session: None,
            occurred_at:           Utc::now(),
        };

        // When
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: UserEvent = serde_json::from_str(&json).unwrap();

        // Then
        match deserialized {
            UserEvent::ProfileUpdated {
                display_name,
                current_level,
                learning_goal,
                questions_per_session,
                ..
            } => {
                assert_eq!(display_name, None);
                assert_eq!(current_level, None);
                assert_eq!(learning_goal, None);
                assert_eq!(questions_per_session, None);
            },
            _ => unreachable!("Deserialized event should be ProfileUpdated"),
        }
    }

    #[test]
    fn cefr_level_should_serialize_correctly() {
        // Given
        let levels = vec![
            CefrLevel::A1,
            CefrLevel::A2,
            CefrLevel::B1,
            CefrLevel::B2,
            CefrLevel::C1,
            CefrLevel::C2,
        ];

        // When & Then
        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: CefrLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, deserialized);
        }
    }

    #[test]
    fn learning_goal_should_serialize_correctly() {
        // Given
        let goals = vec![
            LearningGoal::IeltsScore { overall: 7.5 },
            LearningGoal::ToeflScore { total: 100 },
            LearningGoal::ToeicScore { total: 800 },
            LearningGoal::EikenLevel("準1級".to_string()),
            LearningGoal::GeneralLevel(CefrLevel::B2),
            LearningGoal::NoSpecificGoal,
        ];

        // When & Then
        for goal in goals {
            let json = serde_json::to_string(&goal).unwrap();
            let deserialized: LearningGoal = serde_json::from_str(&json).unwrap();
            assert_eq!(goal, deserialized);
        }
    }
}
