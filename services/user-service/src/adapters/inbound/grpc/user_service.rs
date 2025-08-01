//! gRPC サーバー実装

use std::{str::FromStr, sync::Arc};

use common_types::UserId;
use tonic::{Request, Response, Status};

use super::converters::{
    change_role_request_to_command,
    create_user_request_to_command,
    delete_user_request_to_command,
    proto::{
        ChangeRoleRequest,
        ChangeRoleResponse,
        CreateUserRequest,
        CreateUserResponse,
        DeleteUserRequest,
        DeleteUserResponse,
        GetUserByEmailRequest,
        GetUserByEmailResponse,
        GetUserRequest,
        GetUserResponse,
        UpdateEmailRequest,
        UpdateEmailResponse,
        UpdateProfileRequest,
        UpdateProfileResponse,
        user_service_server::UserService,
    },
    update_email_request_to_command,
    update_profile_request_to_command,
    user_to_proto,
};
use crate::ports::inbound::UserUseCase;

/// gRPC ユーザーサービス実装
#[allow(clippy::module_name_repetitions)]
pub struct UserServiceImpl<U: UserUseCase> {
    use_case: Arc<U>,
}

impl<U: UserUseCase> UserServiceImpl<U> {
    /// 新しい gRPC サービスを作成
    pub const fn new(use_case: Arc<U>) -> Self {
        Self { use_case }
    }
}

#[tonic::async_trait]
impl<U> UserService for UserServiceImpl<U>
where
    U: UserUseCase + 'static,
    U::Error: Into<Status>,
{
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let command = create_user_request_to_command(request.into_inner());

        let user = self
            .use_case
            .create_user(command)
            .await
            .map_err(Into::into)?;

        Ok(Response::new(CreateUserResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let user_id = UserId::from_str(&request.into_inner().user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        let user = self.use_case.get_user(&user_id).await.map_err(Into::into)?;

        Ok(Response::new(GetUserResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn get_user_by_email(
        &self,
        request: Request<GetUserByEmailRequest>,
    ) -> Result<Response<GetUserByEmailResponse>, Status> {
        let email = request.into_inner().email;

        let user = self
            .use_case
            .get_user_by_email(&email)
            .await
            .map_err(Into::into)?;

        Ok(Response::new(GetUserByEmailResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn update_profile(
        &self,
        request: Request<UpdateProfileRequest>,
    ) -> Result<Response<UpdateProfileResponse>, Status> {
        let command = update_profile_request_to_command(request.into_inner())?;

        let user = self
            .use_case
            .update_profile(command)
            .await
            .map_err(Into::into)?;

        Ok(Response::new(UpdateProfileResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn change_role(
        &self,
        request: Request<ChangeRoleRequest>,
    ) -> Result<Response<ChangeRoleResponse>, Status> {
        let command = change_role_request_to_command(&request.into_inner())?;

        let user = self
            .use_case
            .change_role(command)
            .await
            .map_err(Into::into)?;

        Ok(Response::new(ChangeRoleResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn update_email(
        &self,
        request: Request<UpdateEmailRequest>,
    ) -> Result<Response<UpdateEmailResponse>, Status> {
        let command = update_email_request_to_command(request.into_inner())?;

        let user = self
            .use_case
            .update_email(command)
            .await
            .map_err(Into::into)?;

        Ok(Response::new(UpdateEmailResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let command = delete_user_request_to_command(&request.into_inner())?;

        self.use_case
            .delete_user(command)
            .await
            .map_err(Into::into)?;

        Ok(Response::new(DeleteUserResponse {}))
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
        application::use_cases::UseCaseImpl,
    };

    fn create_service()
    -> UserServiceImpl<UseCaseImpl<InMemoryRepository, InMemoryPublisher, MockAuthProvider>> {
        let repository = Arc::new(InMemoryRepository::new());
        let event_publisher = Arc::new(InMemoryPublisher::new());
        let auth_provider = Arc::new(MockAuthProvider::new());
        let use_case = Arc::new(UseCaseImpl::new(repository, event_publisher, auth_provider));

        UserServiceImpl::new(use_case)
    }

    #[tokio::test]
    async fn create_user_should_succeed() {
        // Given
        let service = create_service();
        let request = Request::new(CreateUserRequest {
            email:         "test@example.com".to_string(),
            display_name:  "Test User".to_string(),
            is_first_user: true,
        });

        // When
        let response = service.create_user(request).await;

        // Then
        assert!(response.is_ok());
        let user = response.unwrap().into_inner().user.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.profile.as_ref().unwrap().display_name, "Test User");
    }

    #[tokio::test]
    async fn get_user_should_return_not_found() {
        // Given
        let service = create_service();
        let request = Request::new(GetUserRequest {
            user_id: UserId::new().to_string(),
        });

        // When
        let response = service.get_user(request).await;

        // Then
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::NotFound);
    }
}
