//! ユーザー関連のドメインイベント

use common_types::UserId;
use domain_events::{
    CefrLevel,
    DomainEvent,
    EventMetadata,
    LearningGoal,
    ProfileUpdated,
    UserDeleted,
    UserEvent,
    UserRole,
    UserSignedUp,
    user_event,
};

use crate::domain::aggregates::user::User;

/// User 集約からイベントを生成するヘルパー
pub struct UserEventBuilder;

impl UserEventBuilder {
    /// `UserSignedUp` イベントを生成（アカウント作成時）
    #[must_use]
    pub fn account_created(user: &User) -> DomainEvent {
        DomainEvent::User(UserEvent {
            event: Some(user_event::Event::UserSignedUp(UserSignedUp {
                metadata:     Some(EventMetadata::new(user.id().to_string())),
                user_id:      user.id().to_string(),
                email:        user.email().to_string(),
                display_name: user.profile().display_name().to_string(),
                photo_url:    None,
                initial_role: Self::convert_user_role(user.role()) as i32,
                created_at:   Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            })),
        })
    }

    /// `UserDeleted` イベントを生成（アカウント削除時）
    #[must_use]
    pub fn account_deleted(user_id: &UserId) -> DomainEvent {
        DomainEvent::User(UserEvent {
            event: Some(user_event::Event::UserDeleted(UserDeleted {
                metadata:           Some(EventMetadata::new(user_id.to_string())),
                user_id:            user_id.to_string(),
                email:              String::new(), /* user_use_case で実際の email
                                                    * を設定する必要がある */
                deleted_by_user_id: user_id.to_string(), /* user_use_case で実際の実行者を設定する必要がある */
                deleted_at:         Some(
                    prost_types::Timestamp::from(std::time::SystemTime::now()),
                ),
            })),
        })
    }

    /// `ProfileUpdated` イベントを生成
    #[must_use]
    pub fn profile_updated(user: &User) -> DomainEvent {
        let profile = user.profile();
        let timestamp = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));

        DomainEvent::User(UserEvent {
            event: Some(user_event::Event::ProfileUpdated(ProfileUpdated {
                metadata:              Some(EventMetadata::new(user.id().to_string())),
                user_id:               user.id().to_string(),
                display_name:          Some(profile.display_name().to_string()),
                current_level:         Some(
                    Self::convert_cefr_level(profile.current_level()) as i32
                ),
                learning_goal:         profile.learning_goal().map(Self::convert_learning_goal),
                questions_per_session: Some(u32::from(profile.questions_per_session())),
                occurred_at:           timestamp,
            })),
        })
    }

    /// ローカルの `UserRole` を proto の `UserRole` に変換
    const fn convert_user_role(
        role: crate::domain::value_objects::user_role::UserRole,
    ) -> UserRole {
        match role {
            crate::domain::value_objects::user_role::UserRole::User => UserRole::User,
            crate::domain::value_objects::user_role::UserRole::Admin => UserRole::Admin,
        }
    }

    /// ローカルの `CefrLevel` を proto の `CefrLevel` に変換
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

    /// ローカルの `LearningGoal` を proto の `LearningGoal` に変換
    const fn convert_learning_goal(
        goal: &crate::domain::value_objects::learning_goal::LearningGoal,
    ) -> LearningGoal {
        use domain_events::learning_goal;

        match goal {
            crate::domain::value_objects::learning_goal::LearningGoal::GeneralLevel(level) => {
                LearningGoal {
                    goal: Some(learning_goal::Goal::GeneralLevel(
                        Self::convert_cefr_level(*level) as i32,
                    )),
                }
            },
            crate::domain::value_objects::learning_goal::LearningGoal::NoSpecificGoal => {
                LearningGoal {
                    goal: Some(learning_goal::Goal::NoSpecificGoal(true)),
                }
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
            DomainEvent::User(user_event) => {
                if let Some(user_event::Event::UserSignedUp(user_signed_up)) = user_event.event {
                    assert_eq!(user_signed_up.user_id, user.id().to_string());
                    assert_eq!(user_signed_up.email, "test@example.com");
                    assert_eq!(user_signed_up.display_name, "Test User");
                } else {
                    unreachable!("Should be UserSignedUp event");
                }
            },
            _ => unreachable!("Should be User event"),
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
            DomainEvent::User(user_event) => {
                if let Some(user_event::Event::UserDeleted(user_deleted)) = user_event.event {
                    assert_eq!(user_deleted.user_id, user_id.to_string());
                } else {
                    unreachable!("Should be UserDeleted event");
                }
            },
            _ => unreachable!("Should be User event"),
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
            DomainEvent::User(user_event) => {
                if let Some(user_event::Event::ProfileUpdated(profile_updated)) = user_event.event {
                    assert_eq!(profile_updated.user_id, user.id().to_string());
                    assert_eq!(profile_updated.display_name, Some("Test User".to_string()));
                    assert_eq!(profile_updated.current_level, Some(CefrLevel::B1 as i32)); // デフォルトレベル
                    assert_eq!(profile_updated.questions_per_session, Some(50)); // デフォルト値
                    assert!(profile_updated.learning_goal.is_none()); // デフォルトは None
                } else {
                    unreachable!("Should be ProfileUpdated event");
                }
            },
            _ => unreachable!("Should be User event"),
        }
    }

    #[test]
    fn user_event_builder_should_create_profile_updated_event_with_learning_goal() {
        use crate::domain::value_objects::learning_goal::LearningGoal;

        // Given
        let mut user = User::create(
            UserId::new(),
            Email::new("test@example.com").unwrap(),
            "Test User",
            false,
        )
        .unwrap();

        // プロフィールを更新して学習目標を設定
        user.update_profile(|profile| {
            profile.update_display_name("Updated Name")?;
            profile.update_current_level(crate::domain::value_objects::user_profile::CefrLevel::B2);
            profile.set_learning_goal(Some(LearningGoal::GeneralLevel(
                crate::domain::value_objects::user_profile::CefrLevel::C1,
            )));
            profile.update_questions_per_session(30)
        })
        .unwrap();

        // When
        let event = UserEventBuilder::profile_updated(&user);

        // Then
        match event {
            DomainEvent::User(user_event) => {
                if let Some(user_event::Event::ProfileUpdated(profile_updated)) = user_event.event {
                    assert_eq!(profile_updated.user_id, user.id().to_string());
                    assert_eq!(
                        profile_updated.display_name,
                        Some("Updated Name".to_string())
                    );
                    assert_eq!(profile_updated.current_level, Some(CefrLevel::B2 as i32));
                    assert_eq!(profile_updated.questions_per_session, Some(30));

                    // 学習目標の確認
                    if let Some(learning_goal) = &profile_updated.learning_goal {
                        if let Some(domain_events::learning_goal::Goal::GeneralLevel(level)) =
                            &learning_goal.goal
                        {
                            assert_eq!(*level, CefrLevel::C1 as i32);
                        } else {
                            unreachable!("Should be GeneralLevel learning goal");
                        }
                    } else {
                        unreachable!("Learning goal should be present");
                    }
                } else {
                    unreachable!("Should be ProfileUpdated event");
                }
            },
            _ => unreachable!("Should be User event"),
        }
    }
}
