//! アプリケーション層

pub mod health_check_service;
pub mod vocabulary_query_service;

pub use health_check_service::HealthCheckService;
pub use vocabulary_query_service::VocabularyQueryService;
