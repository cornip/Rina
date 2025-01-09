mod types;
mod store;
pub mod models;
mod error;
pub use types::{Source, ChannelType, MessageMetadata, MessageContent};
pub use store::KnowledgeBase;
pub use models::{Document, Message, Account, Channel, Conversation};
pub use error::ConversionError;
