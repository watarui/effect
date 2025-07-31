//! User コンテキストのドメインイベント
//!
//! shared の domain-events に定義済みのイベントを使用するため、
//! ここでは追加のヘルパー関数やビルダーを定義

use common_types::UserId;
use domain_events::{DomainEvent, EventMetadata, UserEvent};

use crate::domain::aggregates::user::User;

/// User 集約からイベントを生成するヘルパー
pub struct UserEventBuilder;

impl UserEventBuilder {
    /// AccountCreated イベントを生成
    pub fn account_created(user: &User) -> DomainEvent {
        DomainEvent::User(UserEvent::AccountCreated {
            metadata: EventMetadata::new(),
            user_id:  user.id().clone(),
            email:    user.email().to_string(),
        })
    }

    /// AccountDeleted イベントを生成
    pub fn account_deleted(user_id: &UserId) -> DomainEvent {
        DomainEvent::User(UserEvent::AccountDeleted {
            metadata: EventMetadata::new(),
            user_id:  user_id.clone(),
        })
    }

    // TODO: プロフィール更新、ロール変更などの追加イベントは
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
            Email::new("test@example.com".to_string()).unwrap(),
            "Test User".to_string(),
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
            _ => panic!("Expected AccountCreated event"),
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
            _ => panic!("Expected AccountDeleted event"),
        }
    }
}
