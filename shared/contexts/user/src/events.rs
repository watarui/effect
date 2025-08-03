//! User Context のドメインイベント
//!
//! このモジュールは User Context 内のドメインイベントを定義します。
//! Proto ファイルから生成されたコードを使用し、
//! 必要に応じて拡張トレイトを実装します。

use shared_kernel::{DomainEvent, EventMetadata, IntegrationEvent};

// Proto 生成コードを含める
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/effect.events.user.rs"));
    include!(concat!(env!("OUT_DIR"), "/effect.services.user.rs"));
}

// Proto 型を再エクスポート
pub use proto::*;

// DomainEvent トレイトの実装（Proto 生成型用）
impl DomainEvent for UserEvent {
    fn event_type(&self) -> &str {
        match &self.event {
            Some(user_event::Event::UserSignedUp(_)) => "UserSignedUp",
            Some(user_event::Event::ProfileUpdated(_)) => "UserProfileUpdated",
            Some(user_event::Event::LearningGoalSet(_)) => "UserLearningGoalSet",
            Some(user_event::Event::UserRoleChanged(_)) => "UserRoleChanged",
            Some(user_event::Event::UserDeleted(_)) => "UserDeleted",
            Some(user_event::Event::UserSignedIn(_)) => "UserSignedIn",
            Some(user_event::Event::UserSignedOut(_)) => "UserSignedOut",
            Some(user_event::Event::SessionRefreshed(_)) => "UserSessionRefreshed",
            None => "UserEventUnknown",
        }
    }

    fn metadata(&self) -> &EventMetadata {
        // Proto の EventMetadata を shared_kernel の EventMetadata に変換する必要がある
        // 一時的にパニックを返す（後で実装）
        todo!("Convert proto EventMetadata to shared_kernel EventMetadata")
    }
}

/// 統合イベントの定義
/// 他のコンテキストに公開されるイベント
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum UserIntegrationEvent {
    /// ユーザーが登録された
    UserRegistered {
        event_id:     String,
        occurred_at:  chrono::DateTime<chrono::Utc>,
        user_id:      String,
        email:        String,
        display_name: String,
        initial_role: String,
    },
    /// プロフィールが更新された
    ProfileUpdated {
        event_id:       String,
        occurred_at:    chrono::DateTime<chrono::Utc>,
        user_id:        String,
        updated_fields: Vec<String>,
    },
    /// 学習統計が更新された
    LearningStatsUpdated {
        event_id:            String,
        occurred_at:         chrono::DateTime<chrono::Utc>,
        user_id:             String,
        period:              String,
        items_studied:       u32,
        study_time_minutes:  u32,
        average_quality:     f32,
        perfect_recalls:     u32,
        milestones_achieved: Vec<String>,
    },
}

impl IntegrationEvent for UserIntegrationEvent {
    fn event_type(&self) -> &str {
        match self {
            Self::UserRegistered { .. } => "UserRegistered",
            Self::ProfileUpdated { .. } => "UserProfileUpdated",
            Self::LearningStatsUpdated { .. } => "UserLearningStatsUpdated",
        }
    }

    fn source_context(&self) -> &str {
        "user"
    }

    fn event_id(&self) -> &str {
        match self {
            Self::UserRegistered { event_id, .. }
            | Self::ProfileUpdated { event_id, .. }
            | Self::LearningStatsUpdated { event_id, .. } => event_id,
        }
    }

    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::UserRegistered { occurred_at, .. }
            | Self::ProfileUpdated { occurred_at, .. }
            | Self::LearningStatsUpdated { occurred_at, .. } => *occurred_at,
        }
    }
}
