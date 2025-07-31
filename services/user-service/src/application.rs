//! アプリケーション層
//!
//! ユースケースとアプリケーションロジックを含む

pub mod errors;
pub mod use_cases;

pub use errors::ApplicationError as Error;
pub use use_cases::UseCaseImpl;
