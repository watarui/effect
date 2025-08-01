//! ユーザー関連のドメインイベント

use chrono::Utc;
use common_types::UserId;
use domain_events::{CefrLevel, DomainEvent, EventMetadata, LearningGoal, UserEvent};

use crate::domain::aggregates::user::User;

/// User 集約からイベントを生成するヘルパー
pub struct UserEventBuilder;

impl UserEventBuilder {
    /// `AccountCreated` イベントを生成
    #[must_use]
    pub fn account_created(user: &User) -> DomainEvent {
        DomainEvent::User(UserEvent::AccountCreated {
            metadata: EventMetadata::new(),
            user_id:  *user.id(),
            email:    user.email().to_string(),
        })
    }

    /// `AccountDeleted` イベントを生成
    #[must_use]
    pub fn account_deleted(user_id: &UserId) -> DomainEvent {
        DomainEvent::User(UserEvent::AccountDeleted {
            metadata: EventMetadata::new(),
            user_id:  *user_id,
        })
    }

    /// `ProfileUpdated` イベントを生成
    #[must_use]
    pub fn profile_updated(user: &User) -> DomainEvent {
        let profile = user.profile();

        DomainEvent::User(UserEvent::ProfileUpdated {
            metadata:              EventMetadata::new(),
            user_id:               *user.id(),
            display_name:          Some(profile.display_name().to_string()),
            current_level:         Some(Self::convert_cefr_level(profile.current_level())),
            learning_goal:         profile.learning_goal().map(Self::convert_learning_goal),
            questions_per_session: Some(profile.questions_per_session()),
            occurred_at:           Utc::now(),
        })
    }

    /// ローカルの `CefrLevel` を domain-events の `CefrLevel` に変換
    const fn convert_cefr_level(
        level: crate::domain::value_objects::user_profile::CefrLevel,
    ) -> CefrLevel {
        match level {
            crate::domain::value_objects::user_profile::CefrLevel::A1 => CefrLevel::A1,
            crate::domain::value_objects::user_profile::CefrLevel::A2 => CefrLevel::A2,
            crate::domain::value_objects::user_profile::CefrLevel::B1 => CefrLevel::B1,
            crate::domain::value_objects::user_profile::CefrLevel::B2 => CefrLevel::B2,
            crate::domain::value_objects::user_profile::CefrLevel::C1 => CefrLevel::C1,
            crate::domain::value_objects::user_profile::CefrLevel::C2 => CefrLevel::C2,
        }
    }

    /// ローカルの `LearningGoal` を domain-events の `LearningGoal` に変換
    fn convert_learning_goal(
        goal: &crate::domain::value_objects::learning_goal::LearningGoal,
    ) -> LearningGoal {
        match goal {
            crate::domain::value_objects::learning_goal::LearningGoal::IeltsScore(ielts) => {
                LearningGoal::IeltsScore {
                    overall: ielts.overall,
                }
            },
            crate::domain::value_objects::learning_goal::LearningGoal::ToeflScore(toefl) => {
                LearningGoal::ToeflScore { total: toefl.total }
            },
            crate::domain::value_objects::learning_goal::LearningGoal::ToeicScore(toeic) => {
                LearningGoal::ToeicScore { total: toeic.total }
            },
            crate::domain::value_objects::learning_goal::LearningGoal::EikenLevel(level) => {
                LearningGoal::EikenLevel(level.to_string())
            },
            crate::domain::value_objects::learning_goal::LearningGoal::GeneralLevel(level) => {
                LearningGoal::GeneralLevel(Self::convert_cefr_level(*level))
            },
            crate::domain::value_objects::learning_goal::LearningGoal::NoSpecificGoal => {
                // NoSpecificGoal は実際にはマッピングされないが、
                // 型の完全性のために一時的な値を返す
                LearningGoal::EikenLevel("No Specific Goal".to_string())
            },
        }
    }

    // TODO: ロール変更などの追加イベントは
    // shared の domain-events に追加後に実装
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::email::Email;

    #[test]
    fn user_event_builder_should_create_account_created_event() {
        // Given
        let user = User::create(
            UserId::new(),
            Email::new("test@example.com").unwrap(),
            "Test User",
            false,
        )
        .unwrap();

        // When
        let event = UserEventBuilder::account_created(&user);

        // Then
        match event {
            DomainEvent::User(UserEvent::AccountCreated { user_id, email, .. }) => {
                assert_eq!(user_id, *user.id());
                assert_eq!(email, "test@example.com");
            },
            _ => unreachable!("Should be AccountCreated event"),
        }
    }

    #[test]
    fn user_event_builder_should_create_account_deleted_event() {
        // Given
        let user_id = UserId::new();

        // When
        let event = UserEventBuilder::account_deleted(&user_id);

        // Then
        match event {
            DomainEvent::User(UserEvent::AccountDeleted { user_id: id, .. }) => {
                assert_eq!(id, user_id);
            },
            _ => unreachable!("Should be AccountDeleted event"),
        }
    }

    #[test]
    fn user_event_builder_should_create_profile_updated_event() {
        // Given
        let user = User::create(
            UserId::new(),
            Email::new("test@example.com").unwrap(),
            "Test User",
            false,
        )
        .unwrap();

        // When
        let event = UserEventBuilder::profile_updated(&user);

        // Then
        match event {
            DomainEvent::User(UserEvent::ProfileUpdated {
                user_id,
                display_name,
                current_level,
                questions_per_session,
                learning_goal,
                ..
            }) => {
                assert_eq!(user_id, *user.id());
                assert_eq!(display_name, Some("Test User".to_string()));
                assert_eq!(current_level, Some(CefrLevel::B1)); // デフォルトレベル
                assert_eq!(questions_per_session, Some(50)); // デフォルト値
                assert!(learning_goal.is_none()); // デフォルトは None
            },
            _ => unreachable!("Should be ProfileUpdated event"),
        }
    }

    #[test]
    fn user_event_builder_should_create_profile_updated_event_with_learning_goal() {
        use crate::domain::value_objects::learning_goal::{IeltsScore, LearningGoal};

        // Given
        let mut user = User::create(
            UserId::new(),
            Email::new("test@example.com").unwrap(),
            "Test User",
            false,
        )
        .unwrap();

        // プロフィールを更新して学習目標を設定
        let ielts_goal = IeltsScore::new(7.5, Some(8.0), Some(7.0), Some(6.5), Some(7.5)).unwrap();
        user.update_profile(|profile| {
            profile.update_display_name("Updated Name")?;
            profile.update_current_level(crate::domain::value_objects::user_profile::CefrLevel::B2);
            profile.set_learning_goal(Some(LearningGoal::IeltsScore(ielts_goal.clone())));
            profile.update_questions_per_session(30)
        })
        .unwrap();

        // When
        let event = UserEventBuilder::profile_updated(&user);

        // Then
        match event {
            DomainEvent::User(UserEvent::ProfileUpdated {
                user_id,
                display_name,
                current_level,
                questions_per_session,
                learning_goal,
                ..
            }) => {
                assert_eq!(user_id, *user.id());
                assert_eq!(display_name, Some("Updated Name".to_string()));
                assert_eq!(current_level, Some(CefrLevel::B2));
                assert_eq!(questions_per_session, Some(30));

                // 学習目標の確認
                match learning_goal {
                    Some(domain_events::LearningGoal::IeltsScore { overall }) => {
                        assert!((overall - 7.5).abs() < f32::EPSILON);
                    },
                    _ => unreachable!("Should be IeltsScore learning goal"),
                }
            },
            _ => unreachable!("Should be ProfileUpdated event"),
        }
    }
}
