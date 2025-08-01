//! gRPC アダプター

pub mod converters;
pub mod user_service;

pub use converters::proto;
pub use user_service::UserServiceImpl;
