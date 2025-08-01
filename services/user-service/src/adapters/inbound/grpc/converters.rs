//! Proto メッセージとドメインモデルの変換

use std::str::FromStr;

use chrono::{DateTime, TimeZone, Utc};
use common_types::UserId;
use prost_types::Timestamp;
use tonic::Status;

use crate::{
    application::errors::ApplicationError,
    domain::{
        aggregates::user::User,
        commands::{
            ChangeUserRole,
            CreateUser,
            DeleteUser,
            TargetLevelUpdate,
            UpdateUserEmail,
            UpdateUserProfile,
        },
        value_objects::{user_profile::CefrLevel, user_role::UserRole},
    },
};

// Proto 定義を含むモジュール（build.rs で生成される）
pub mod proto {
    #![allow(missing_docs)]
    #![allow(clippy::pedantic)]
    #![allow(clippy::nursery)]
    #![allow(clippy::clone_on_ref_ptr)]
    tonic::include_proto!("user_service");
}

use proto::{
    CefrLevel as ProtoCefrLevel,
    TargetLevelUpdate as ProtoTargetLevelUpdate,
    UserRole as ProtoUserRole,
};

/// ドメインの `UserRole` から Proto への変換
impl From<UserRole> for ProtoUserRole {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::User => Self::User,
            UserRole::Admin => Self::Admin,
        }
    }
}

/// Proto の `UserRole` からドメインへの変換
impl TryFrom<ProtoUserRole> for UserRole {
    type Error = Status;

    fn try_from(role: ProtoUserRole) -> Result<Self, Self::Error> {
        match role {
            ProtoUserRole::Unspecified => {
                Err(Status::invalid_argument("User role must be specified"))
            },
            ProtoUserRole::User => Ok(Self::User),
            ProtoUserRole::Admin => Ok(Self::Admin),
        }
    }
}

/// ドメインの `CefrLevel` から Proto への変換
impl From<CefrLevel> for ProtoCefrLevel {
    fn from(level: CefrLevel) -> Self {
        match level {
            CefrLevel::A1 => Self::A1,
            CefrLevel::A2 => Self::A2,
            CefrLevel::B1 => Self::B1,
            CefrLevel::B2 => Self::B2,
            CefrLevel::C1 => Self::C1,
            CefrLevel::C2 => Self::C2,
        }
    }
}

/// Proto の `CefrLevel` からドメインへの変換
impl TryFrom<ProtoCefrLevel> for CefrLevel {
    type Error = Status;

    fn try_from(level: ProtoCefrLevel) -> Result<Self, Self::Error> {
        match level {
            ProtoCefrLevel::Unspecified => {
                Err(Status::invalid_argument("CEFR level must be specified"))
            },
            ProtoCefrLevel::A1 => Ok(Self::A1),
            ProtoCefrLevel::A2 => Ok(Self::A2),
            ProtoCefrLevel::B1 => Ok(Self::B1),
            ProtoCefrLevel::B2 => Ok(Self::B2),
            ProtoCefrLevel::C1 => Ok(Self::C1),
            ProtoCefrLevel::C2 => Ok(Self::C2),
        }
    }
}

/// `DateTime` から Proto Timestamp への変換
#[must_use]
#[allow(clippy::cast_possible_wrap)]
pub const fn datetime_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos:   dt.timestamp_subsec_nanos() as i32,
    }
}

/// Proto Timestamp から `DateTime` への変換
///
/// # Errors
///
/// タイムスタンプが無効な場合は `Status::invalid_argument` を返します
pub fn timestamp_to_datetime(ts: &Timestamp) -> Result<DateTime<Utc>, Status> {
    Utc.timestamp_opt(ts.seconds, u32::try_from(ts.nanos).unwrap_or(0))
        .single()
        .ok_or_else(|| Status::invalid_argument("Invalid timestamp"))
}

/// ドメインの User から Proto への変換
#[must_use]
pub fn user_to_proto(user: &User) -> proto::User {
    proto::User {
        id:         user.id().to_string(),
        email:      user.email().to_string(),
        profile:    Some(proto::UserProfile {
            display_name:          user.profile().display_name().to_string(),
            current_level:         ProtoCefrLevel::from(user.profile().current_level()) as i32,
            target_level:          user
                .profile()
                .target_level()
                .map(|l| ProtoCefrLevel::from(l) as i32),
            questions_per_session: u32::from(user.profile().questions_per_session()),
            created_at:            Some(datetime_to_timestamp(user.profile().created_at())),
            updated_at:            Some(datetime_to_timestamp(user.profile().updated_at())),
        }),
        role:       ProtoUserRole::from(user.role()) as i32,
        created_at: Some(datetime_to_timestamp(user.created_at())),
        updated_at: Some(datetime_to_timestamp(user.updated_at())),
    }
}

/// Proto リクエストからドメインコマンドへの変換
#[must_use]
pub fn create_user_request_to_command(req: proto::CreateUserRequest) -> CreateUser {
    CreateUser {
        email:         req.email,
        display_name:  req.display_name,
        is_first_user: req.is_first_user,
    }
}

/// Proto の `TargetLevelUpdate` からドメインへの変換
///
/// # Errors
///
/// CEFR レベルが無効な場合は `Status::invalid_argument` を返します
pub fn proto_target_level_update_to_domain(
    update: Option<ProtoTargetLevelUpdate>,
) -> Result<TargetLevelUpdate, Status> {
    match update {
        None => Ok(TargetLevelUpdate::NoChange),
        Some(proto_update) => match proto_update.update {
            None => Ok(TargetLevelUpdate::NoChange),
            Some(proto::target_level_update::Update::NoChange(_)) => {
                Ok(TargetLevelUpdate::NoChange)
            },
            Some(proto::target_level_update::Update::SetLevel(level)) => {
                let cefr_level = ProtoCefrLevel::try_from(level)
                    .map_err(|_| Status::invalid_argument("Invalid CEFR level"))?
                    .try_into()?;
                Ok(TargetLevelUpdate::Set(cefr_level))
            },
            Some(proto::target_level_update::Update::Clear(_)) => Ok(TargetLevelUpdate::Clear),
        },
    }
}

/// Proto リクエストからドメインコマンドへの変換
///
/// # Errors
///
/// ユーザー ID や CEFR レベルが無効な場合は `Status::invalid_argument`
/// を返します
pub fn update_profile_request_to_command(
    req: proto::UpdateProfileRequest,
) -> Result<UpdateUserProfile, Status> {
    let user_id =
        UserId::from_str(&req.user_id).map_err(|_| Status::invalid_argument("Invalid user ID"))?;

    let current_level = match req.current_level {
        None => None,
        Some(level) => {
            let proto_level = ProtoCefrLevel::try_from(level)
                .map_err(|_| Status::invalid_argument("Invalid CEFR level"))?;
            Some(proto_level.try_into()?)
        },
    };

    let target_level = proto_target_level_update_to_domain(req.target_level)?;

    Ok(UpdateUserProfile {
        user_id,
        display_name: req.display_name,
        current_level,
        target_level,
        questions_per_session: req
            .questions_per_session
            .map(|q| u8::try_from(q).unwrap_or(10)),
    })
}

/// Proto リクエストからドメインコマンドへの変換
///
/// # Errors
///
/// ユーザー ID、実行者 ID、またはロールが無効な場合は
/// `Status::invalid_argument` を返します
pub fn change_role_request_to_command(
    req: &proto::ChangeRoleRequest,
) -> Result<ChangeUserRole, Status> {
    let user_id =
        UserId::from_str(&req.user_id).map_err(|_| Status::invalid_argument("Invalid user ID"))?;
    let executed_by = UserId::from_str(&req.executed_by)
        .map_err(|_| Status::invalid_argument("Invalid executed_by ID"))?;
    let new_role = ProtoUserRole::try_from(req.new_role)
        .map_err(|_| Status::invalid_argument("Invalid user role"))?
        .try_into()?;

    Ok(ChangeUserRole {
        user_id,
        new_role,
        executed_by,
    })
}

/// Proto リクエストからドメインコマンドへの変換
///
/// # Errors
///
/// ユーザー ID が無効な場合は `Status::invalid_argument` を返します
pub fn update_email_request_to_command(
    req: proto::UpdateEmailRequest,
) -> Result<UpdateUserEmail, Status> {
    let user_id =
        UserId::from_str(&req.user_id).map_err(|_| Status::invalid_argument("Invalid user ID"))?;

    Ok(UpdateUserEmail {
        user_id,
        new_email: req.new_email,
    })
}

/// Proto リクエストからドメインコマンドへの変換
///
/// # Errors
///
/// ユーザー ID または実行者 ID が無効な場合は `Status::invalid_argument`
/// を返します
pub fn delete_user_request_to_command(
    req: &proto::DeleteUserRequest,
) -> Result<DeleteUser, Status> {
    let user_id =
        UserId::from_str(&req.user_id).map_err(|_| Status::invalid_argument("Invalid user ID"))?;
    let executed_by = UserId::from_str(&req.executed_by)
        .map_err(|_| Status::invalid_argument("Invalid executed_by ID"))?;

    Ok(DeleteUser {
        user_id,
        executed_by,
    })
}

/// `ApplicationError` から gRPC Status への変換
impl From<ApplicationError> for Status {
    fn from(err: ApplicationError) -> Self {
        match err {
            ApplicationError::UserNotFound => Self::not_found("User not found"),
            ApplicationError::EmailAlreadyExists => Self::already_exists("Email already exists"),
            ApplicationError::InvalidEmail => Self::invalid_argument("Invalid email format"),
            ApplicationError::InvalidProfile(msg) => Self::invalid_argument(msg),
            ApplicationError::Authentication(msg) => Self::unauthenticated(msg),
            ApplicationError::PermissionDenied => Self::permission_denied("Permission denied"),
            ApplicationError::Repository(msg) => Self::internal(format!("Repository error: {msg}")),
            ApplicationError::EventPublishing(msg) => {
                Self::internal(format!("Event publishing error: {msg}"))
            },
            ApplicationError::Internal(msg) => Self::internal(msg),
        }
    }
}
