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
            SetLearningGoal,
            UpdateUserEmail,
            UpdateUserProfile,
        },
        value_objects::{
            account_status::AccountStatus,
            learning_goal::{EikenLevel, IeltsScore, LearningGoal, ToeflScore, ToeicScore},
            user_profile::CefrLevel,
            user_role::UserRole,
        },
    },
};

// Proto 定義を含むモジュール（build.rs で生成される）
/// Proto 定義のモジュール
pub mod proto {
    /// 共通型定義
    pub mod common {
        #![allow(missing_docs)]
        #![allow(clippy::pedantic)]
        #![allow(clippy::nursery)]
        #![allow(clippy::clone_on_ref_ptr)]
        tonic::include_proto!("effect.common");
    }

    /// サービス定義
    pub mod services {
        /// ユーザーサービス
        pub mod user {
            #![allow(missing_docs)]
            #![allow(clippy::pedantic)]
            #![allow(clippy::nursery)]
            #![allow(clippy::clone_on_ref_ptr)]
            tonic::include_proto!("effect.services.user");
        }
    }

    /// イベント定義
    pub mod events {
        /// ユーザーイベント
        pub mod user {
            #![allow(missing_docs)]
            #![allow(clippy::pedantic)]
            #![allow(clippy::nursery)]
            #![allow(clippy::clone_on_ref_ptr)]
            tonic::include_proto!("effect.events.user");
        }
    }
}

use proto::{
    common::{
        AccountStatus as ProtoAccountStatus,
        CefrLevel as ProtoCefrLevel,
        UserRole as ProtoUserRole,
    },
    services::user::{
        ChangeRoleRequest,
        CreateUserRequest,
        DeleteUserRequest,
        EikenLevel as ProtoEikenLevel,
        IeltsScore as ProtoIeltsScore,
        LearningGoal as ProtoLearningGoal,
        SetLearningGoalRequest,
        ToeflScore as ProtoToeflScore,
        ToeicScore as ProtoToeicScore,
        UpdateEmailRequest,
        UpdateProfileRequest,
        User as ProtoUser,
        UserProfile as ProtoUserProfile,
    },
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

/// ドメインの `AccountStatus` から Proto への変換
impl From<AccountStatus> for ProtoAccountStatus {
    fn from(status: AccountStatus) -> Self {
        match status {
            AccountStatus::Active => Self::Active,
            AccountStatus::Deleted => Self::Deleted,
        }
    }
}

/// Proto の `AccountStatus` からドメインへの変換
impl TryFrom<ProtoAccountStatus> for AccountStatus {
    type Error = Status;

    fn try_from(status: ProtoAccountStatus) -> Result<Self, Self::Error> {
        match status {
            ProtoAccountStatus::Unspecified => {
                Err(Status::invalid_argument("Account status must be specified"))
            },
            ProtoAccountStatus::Active => Ok(Self::Active),
            ProtoAccountStatus::Deleted => Ok(Self::Deleted),
        }
    }
}

/// ドメインの `EikenLevel` から Proto への変換
impl From<EikenLevel> for ProtoEikenLevel {
    fn from(level: EikenLevel) -> Self {
        match level {
            EikenLevel::Level5 => Self::EikenLevel5,
            EikenLevel::Level4 => Self::EikenLevel4,
            EikenLevel::Level3 => Self::EikenLevel3,
            EikenLevel::PreLevel2 => Self::Pre2,
            EikenLevel::Level2 => Self::EikenLevel2,
            EikenLevel::PreLevel1 => Self::Pre1,
            EikenLevel::Level1 => Self::EikenLevel1,
        }
    }
}

/// Proto の `EikenLevel` からドメインへの変換
impl TryFrom<ProtoEikenLevel> for EikenLevel {
    type Error = Status;

    fn try_from(level: ProtoEikenLevel) -> Result<Self, Self::Error> {
        match level {
            ProtoEikenLevel::Unspecified => {
                Err(Status::invalid_argument("Eiken level must be specified"))
            },
            ProtoEikenLevel::EikenLevel5 => Ok(Self::Level5),
            ProtoEikenLevel::EikenLevel4 => Ok(Self::Level4),
            ProtoEikenLevel::EikenLevel3 => Ok(Self::Level3),
            ProtoEikenLevel::Pre2 => Ok(Self::PreLevel2),
            ProtoEikenLevel::EikenLevel2 => Ok(Self::Level2),
            ProtoEikenLevel::Pre1 => Ok(Self::PreLevel1),
            ProtoEikenLevel::EikenLevel1 => Ok(Self::Level1),
        }
    }
}

/// ドメインの `LearningGoal` から Proto への変換
fn learning_goal_to_proto(goal: &LearningGoal) -> ProtoLearningGoal {
    use proto::services::user::learning_goal::Goal;

    let goal = match goal {
        LearningGoal::IeltsScore(score) => Goal::IeltsScore(ProtoIeltsScore {
            overall:   score.overall,
            reading:   score.reading,
            listening: score.listening,
            writing:   score.writing,
            speaking:  score.speaking,
        }),
        LearningGoal::ToeflScore(score) => Goal::ToeflScore(ProtoToeflScore {
            total:     u32::from(score.total),
            reading:   score.reading.map(u32::from),
            listening: score.listening.map(u32::from),
            speaking:  score.speaking.map(u32::from),
            writing:   score.writing.map(u32::from),
        }),
        LearningGoal::ToeicScore(score) => Goal::ToeicScore(ProtoToeicScore {
            total:     u32::from(score.total),
            listening: score.listening.map(u32::from),
            reading:   score.reading.map(u32::from),
        }),
        LearningGoal::EikenLevel(level) => Goal::EikenLevel(ProtoEikenLevel::from(*level) as i32),
        LearningGoal::GeneralLevel(level) => {
            Goal::GeneralLevel(ProtoCefrLevel::from(*level) as i32)
        },
        LearningGoal::NoSpecificGoal => Goal::NoSpecificGoal(true),
    };

    ProtoLearningGoal { goal: Some(goal) }
}

/// Proto の `LearningGoal` からドメインへの変換
///
/// # Errors
///
/// 無効な学習目標の場合は `Status::invalid_argument` を返します
fn proto_learning_goal_to_domain(
    goal: Option<ProtoLearningGoal>,
) -> Result<Option<LearningGoal>, Status> {
    use proto::services::user::learning_goal::Goal;

    let Some(goal) = goal else {
        return Ok(None);
    };

    let Some(goal_type) = goal.goal else {
        return Ok(None);
    };

    let domain_goal = match goal_type {
        Goal::IeltsScore(score) => {
            let ielts = IeltsScore::new(
                score.overall,
                score.reading,
                score.listening,
                score.writing,
                score.speaking,
            )
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
            LearningGoal::IeltsScore(ielts)
        },
        Goal::ToeflScore(score) => {
            let toefl = ToeflScore::new(
                u8::try_from(score.total)
                    .map_err(|_| Status::invalid_argument("Invalid TOEFL total score"))?,
                score.reading.map(|s| u8::try_from(s).unwrap_or(0)),
                score.listening.map(|s| u8::try_from(s).unwrap_or(0)),
                score.speaking.map(|s| u8::try_from(s).unwrap_or(0)),
                score.writing.map(|s| u8::try_from(s).unwrap_or(0)),
            )
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
            LearningGoal::ToeflScore(toefl)
        },
        Goal::ToeicScore(score) => {
            let toeic = ToeicScore::new(
                u16::try_from(score.total)
                    .map_err(|_| Status::invalid_argument("Invalid TOEIC total score"))?,
                score.listening.map(|s| u16::try_from(s).unwrap_or(0)),
                score.reading.map(|s| u16::try_from(s).unwrap_or(0)),
            )
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
            LearningGoal::ToeicScore(toeic)
        },
        Goal::EikenLevel(level) => {
            let eiken = ProtoEikenLevel::try_from(level)
                .map_err(|_| Status::invalid_argument("Invalid Eiken level"))?
                .try_into()?;
            LearningGoal::EikenLevel(eiken)
        },
        Goal::GeneralLevel(level) => {
            let cefr = ProtoCefrLevel::try_from(level)
                .map_err(|_| Status::invalid_argument("Invalid CEFR level"))?
                .try_into()?;
            LearningGoal::GeneralLevel(cefr)
        },
        Goal::NoSpecificGoal(_) => LearningGoal::NoSpecificGoal,
    };

    Ok(Some(domain_goal))
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
pub fn user_to_proto(user: &User) -> ProtoUser {
    ProtoUser {
        id:         user.id().to_string(),
        email:      user.email().to_string(),
        profile:    Some(ProtoUserProfile {
            display_name:          user.profile().display_name().to_string(),
            current_level:         ProtoCefrLevel::from(user.profile().current_level()) as i32,
            learning_goal:         user.profile().learning_goal().map(learning_goal_to_proto),
            questions_per_session: u32::from(user.profile().questions_per_session()),
            created_at:            Some(datetime_to_timestamp(user.profile().created_at())),
            updated_at:            Some(datetime_to_timestamp(user.profile().updated_at())),
        }),
        role:       ProtoUserRole::from(user.role()) as i32,
        status:     ProtoAccountStatus::from(user.status()) as i32,
        created_at: Some(datetime_to_timestamp(user.created_at())),
        updated_at: Some(datetime_to_timestamp(user.updated_at())),
        version:    user.version(),
    }
}

/// Proto リクエストからドメインコマンドへの変換
#[must_use]
pub fn create_user_request_to_command(req: CreateUserRequest) -> CreateUser {
    CreateUser {
        email:         req.email,
        display_name:  req.display_name,
        is_first_user: req.is_first_user,
    }
}

/// Proto リクエストからドメインコマンドへの変換
///
/// # Errors
///
/// ユーザー ID や CEFR レベルが無効な場合は `Status::invalid_argument`
/// を返します
pub fn update_profile_request_to_command(
    req: UpdateProfileRequest,
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

    Ok(UpdateUserProfile {
        user_id,
        display_name: req.display_name,
        current_level,
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
pub fn change_role_request_to_command(req: &ChangeRoleRequest) -> Result<ChangeUserRole, Status> {
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
pub fn update_email_request_to_command(req: UpdateEmailRequest) -> Result<UpdateUserEmail, Status> {
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
/// ユーザー ID が無効な場合、または学習目標が無効な場合は
/// `Status::invalid_argument` を返します
pub fn set_learning_goal_request_to_command(
    req: &SetLearningGoalRequest,
) -> Result<SetLearningGoal, Status> {
    let user_id =
        UserId::from_str(&req.user_id).map_err(|_| Status::invalid_argument("Invalid user ID"))?;

    let goal = proto_learning_goal_to_domain(req.goal)?;

    Ok(SetLearningGoal { user_id, goal })
}

/// Proto リクエストからドメインコマンドへの変換
///
/// # Errors
///
/// ユーザー ID または実行者 ID が無効な場合は `Status::invalid_argument`
/// を返します
pub fn delete_user_request_to_command(req: &DeleteUserRequest) -> Result<DeleteUser, Status> {
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
