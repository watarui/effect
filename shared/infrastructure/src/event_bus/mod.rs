//! イベントバス実装
//!
//! このモジュールは [`EventBus`] トレイトの異なるメッセージングシステム向けの
//! 実装を提供します。

pub mod pubsub;

#[allow(clippy::module_name_repetitions)]
pub use pubsub::PubSubEventBus;
