//! Vocabulary Search Service

pub mod application;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod ports;
pub mod server;

pub use config::Config;
pub use error::{Result, SearchError};
pub use server::run;
