//! Shared infrastructure components for the Effect application.
//!
//! This crate contains database connections, message bus, and other
//! infrastructure.

pub mod event_bus;

// Re-export commonly used types
pub use event_bus::PubSubEventBus;
