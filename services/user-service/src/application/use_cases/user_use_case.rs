//! ユーザーユースケース実装

use std::sync::Arc;

use async_trait::async_trait;
use common_types::UserId;
use domain_events::{DomainEvent, EventMetadata, UserEvent};

use crate::{
    application::errors::ApplicationError,
    domain::{
        aggregates::user::User,
        commands::{ChangeUserRole, CreateUser, DeleteUser, SetLearningGoal, UpdateUserProfile},
        events::UserEventBuilder,
        services::{UserDomainService, UserDomainServiceImpl},
        value_objects::email::Email,
    },
    ports::{
        inbound::UserUseCase,
        outbound::{AuthProvider, EventPublisher, UserRepository},
    },
};

/// ユーザーユースケース実装
pub struct UseCaseImpl<R, E, A>
where
    R: UserRepository,
    E: EventPublisher,
    A: AuthProvider,
{
    repository:      Arc<R>,
    event_publisher: Arc<E>,
    _auth_provider:  Arc<A>, // 将来の認証機能拡張用
    domain_service:  Arc<UserDomainServiceImpl<R>>,
}

impl<R, E, A> UseCaseImpl<R, E, A>
where
    R: UserRepository,
    E: EventPublisher,
    A: AuthProvider,
{
    /// 新しいユーザーユースケースを作成
    pub fn new(repository: Arc<R>, event_publisher: Arc<E>, auth_provider: Arc<A>) -> Self {
        let domain_service = Arc::new(UserDomainServiceImpl::new(Arc::clone(&repository)));
        Self {
            repository,
            event_publisher,
            _auth_provider: auth_provider,
            domain_service,
        }
    }
}

#[async_trait]
impl<R, E, A> UserUseCase for UseCaseImpl<R, E, A>
where
    R: UserRepository + 'static,
    E: EventPublisher + 'static,
    A: AuthProvider + 'static,
{
    type Error = ApplicationError;

    async fn create_user(&self, command: CreateUser) -> Result<User, Self::Error> {
        // Email を検証
        let email = Email::new(&command.email)?;

        // Email の重複チェック
        if self
            .repository
            .find_by_email(email.as_str())
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?
            .is_some()
        {
            return Err(ApplicationError::EmailAlreadyExists);
        }

        // ドメインサービスを使用して初期ロールを決定
        let initial_role = self
            .domain_service
            .determine_initial_role(command.is_first_user)
            .await
            .map_err(|e| ApplicationError::DomainLogic(e.to_string()))?;

        // ユーザーを作成
        let user = User::new_with_role(UserId::new(), email, &command.display_name, initial_role)?;

        // リポジトリに保存
        self.repository
            .save(&user)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // イベントを発行
        let event = UserEventBuilder::account_created(&user);
        self.event_publisher
            .publish(&event)
            .await
            .map_err(|e| ApplicationError::EventPublishing(e.to_string()))?;

        Ok(user)
    }

    async fn get_user(&self, user_id: &UserId) -> Result<User, Self::Error> {
        self.repository
            .find_by_id(user_id)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?
            .ok_or(ApplicationError::UserNotFound)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User, Self::Error> {
        self.repository
            .find_by_email(email)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?
            .ok_or(ApplicationError::UserNotFound)
    }

    async fn update_profile(&self, command: UpdateUserProfile) -> Result<User, Self::Error> {
        // ユーザーを取得
        let mut user = self.get_user(&command.user_id).await?;

        // プロフィールを更新
        user.update_profile(|profile| {
            if let Some(display_name) = &command.display_name {
                profile.update_display_name(display_name)?;
            }

            if let Some(current_level) = command.current_level {
                profile.update_current_level(current_level);
            }

            if let Some(questions) = command.questions_per_session {
                profile.update_questions_per_session(questions)?;
            }

            Ok(())
        })?;

        // リポジトリに保存
        self.repository
            .save(&user)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // プロフィール更新イベントを発行
        let event = UserEventBuilder::profile_updated(&user);
        self.event_publisher
            .publish(&event)
            .await
            .map_err(|e| ApplicationError::EventPublishing(e.to_string()))?;

        Ok(user)
    }

    async fn set_learning_goal(&self, command: SetLearningGoal) -> Result<User, Self::Error> {
        // ユーザーを取得
        let mut user = self.get_user(&command.user_id).await?;

        // 学習目標を設定
        user.update_profile(|profile| {
            profile.set_learning_goal(command.goal.clone());
            Ok(())
        })?;

        // リポジトリに保存
        self.repository
            .save(&user)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // TODO: 学習目標設定イベントを発行（domain-events に追加後）

        Ok(user)
    }

    async fn change_role(&self, command: ChangeUserRole) -> Result<User, Self::Error> {
        // 実行者を取得して権限を確認
        let executor = self.get_user(&command.executed_by).await?;
        if !executor.is_admin() {
            return Err(ApplicationError::PermissionDenied);
        }

        // 対象ユーザーを取得
        let mut user = self.get_user(&command.user_id).await?;

        // ロールを変更
        user.change_role(command.new_role);

        // リポジトリに保存
        self.repository
            .save(&user)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // TODO: ロール変更イベントを発行（domain-events に追加後）

        Ok(user)
    }

    async fn delete_user(&self, command: DeleteUser) -> Result<(), Self::Error> {
        // 実行者を取得
        let executor = self.get_user(&command.executed_by).await?;

        // 権限を確認（本人または管理者）
        if command.user_id != command.executed_by && !executor.is_admin() {
            return Err(ApplicationError::PermissionDenied);
        }

        // 削除対象のユーザーを取得（イベント発行用）
        let user_to_delete = self.get_user(&command.user_id).await?;
        let user_email = user_to_delete.email().to_string();

        // ユーザーを削除
        self.repository
            .delete(&command.user_id)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // イベントを発行
        // UserDeleted イベントを手動で作成（email と deleted_by_user_id を正しく設定）
        use domain_events::{UserDeleted, user_event};
        let event = DomainEvent::User(UserEvent {
            event: Some(user_event::Event::UserDeleted(UserDeleted {
                metadata:           Some(EventMetadata::new(command.user_id.to_string())),
                user_id:            command.user_id.to_string(),
                email:              user_email,
                deleted_by_user_id: command.executed_by.to_string(),
                deleted_at:         Some(
                    prost_types::Timestamp::from(std::time::SystemTime::now()),
                ),
            })),
        });
        self.event_publisher
            .publish(&event)
            .await
            .map_err(|e| ApplicationError::EventPublishing(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapters::outbound::{
            auth::mock::Provider as MockAuthProvider,
            event::memory::InMemoryPublisher,
            repository::memory::InMemoryRepository,
        },
        domain::value_objects::user_role::UserRole,
    };

    fn create_use_case() -> UseCaseImpl<InMemoryRepository, InMemoryPublisher, MockAuthProvider> {
        let repository = Arc::new(InMemoryRepository::new());
        let event_publisher = Arc::new(InMemoryPublisher::new());
        let auth_provider = Arc::new(MockAuthProvider::new());

        UseCaseImpl::new(repository, event_publisher, auth_provider)
    }

    #[tokio::test]
    async fn create_user_should_create_admin_for_first_user() {
        // Given
        let use_case = create_use_case();
        let command = CreateUser {
            email:         "admin@example.com".to_string(),
            display_name:  "Admin User".to_string(),
            is_first_user: false, // リポジトリが空なので自動的に Admin になるはず
        };

        // When
        let result = use_case.create_user(command).await;

        // Then
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.role(), UserRole::Admin);
        assert!(user.is_admin());
    }

    #[tokio::test]
    async fn create_user_should_fail_with_duplicate_email() {
        // Given
        let use_case = create_use_case();
        let command1 = CreateUser {
            email:         "test@example.com".to_string(),
            display_name:  "Test User 1".to_string(),
            is_first_user: false,
        };
        let command2 = CreateUser {
            email:         "test@example.com".to_string(),
            display_name:  "Test User 2".to_string(),
            is_first_user: false,
        };

        // When
        let result1 = use_case.create_user(command1).await;
        let result2 = use_case.create_user(command2).await;

        // Then
        assert!(result1.is_ok());
        assert!(result2.is_err());
        assert!(matches!(
            result2.unwrap_err(),
            ApplicationError::EmailAlreadyExists
        ));
    }

    #[tokio::test]
    async fn change_role_should_require_admin_permission() {
        // Given
        let use_case = create_use_case();

        // 管理者を作成
        let admin = use_case
            .create_user(CreateUser {
                email:         "admin@example.com".to_string(),
                display_name:  "Admin".to_string(),
                is_first_user: true,
            })
            .await
            .unwrap();

        // 一般ユーザーを作成
        let user = use_case
            .create_user(CreateUser {
                email:         "user@example.com".to_string(),
                display_name:  "User".to_string(),
                is_first_user: false,
            })
            .await
            .unwrap();

        // When - 一般ユーザーが他のユーザーのロールを変更しようとする
        let result = use_case
            .change_role(ChangeUserRole {
                user_id:     *admin.id(),
                new_role:    UserRole::User,
                executed_by: *user.id(),
            })
            .await;

        // Then
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::PermissionDenied
        ));
    }

    #[tokio::test]
    async fn delete_user_should_allow_self_deletion() {
        // Given
        let use_case = create_use_case();
        let user = use_case
            .create_user(CreateUser {
                email:         "user@example.com".to_string(),
                display_name:  "User".to_string(),
                is_first_user: false,
            })
            .await
            .unwrap();

        // When
        let result = use_case
            .delete_user(DeleteUser {
                user_id:     *user.id(),
                executed_by: *user.id(),
            })
            .await;

        // Then
        assert!(result.is_ok());

        // ユーザーが削除されたことを確認
        let get_result = use_case.get_user(user.id()).await;
        assert!(get_result.is_err());
        assert!(matches!(
            get_result.unwrap_err(),
            ApplicationError::UserNotFound
        ));
    }

    #[tokio::test]
    async fn update_profile_should_publish_profile_updated_event() {
        // Given
        let use_case = create_use_case();
        let user = use_case
            .create_user(CreateUser {
                email:         "user@example.com".to_string(),
                display_name:  "Original Name".to_string(),
                is_first_user: false,
            })
            .await
            .unwrap();

        let command = UpdateUserProfile {
            user_id:               *user.id(),
            display_name:          Some("Updated Name".to_string()),
            current_level:         Some(crate::domain::value_objects::user_profile::CefrLevel::B2),
            questions_per_session: Some(30),
        };

        // When
        let result = use_case.update_profile(command).await;

        // Then
        assert!(result.is_ok());

        let updated_user = result.unwrap();
        assert_eq!(updated_user.profile().display_name(), "Updated Name");
        assert_eq!(
            updated_user.profile().current_level(),
            crate::domain::value_objects::user_profile::CefrLevel::B2
        );
        assert_eq!(updated_user.profile().questions_per_session(), 30);

        // イベントが発行されたことを確認
        let events = use_case.event_publisher.get_published_events().await;
        assert_eq!(events.len(), 2); // UserSignedUp + ProfileUpdated

        // ProfileUpdated イベントの内容を確認
        let profile_updated_event = &events[1];
        match profile_updated_event {
            domain_events::DomainEvent::User(user_event) => match &user_event.event {
                Some(domain_events::user_event::Event::ProfileUpdated(updated)) => {
                    assert_eq!(updated.user_id, updated_user.id().to_string());
                    assert_eq!(updated.display_name, Some("Updated Name".to_string()));
                    assert_eq!(
                        updated.current_level,
                        Some(domain_events::CefrLevel::B2 as i32)
                    );
                    assert_eq!(updated.questions_per_session, Some(30));
                },
                _ => unreachable!("Should be ProfileUpdated event"),
            },
            _ => unreachable!("Should be User event"),
        }
    }
}
