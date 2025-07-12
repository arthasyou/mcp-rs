pub mod annotation;
pub mod content;
pub mod prompt;
pub mod protocol;
pub mod resource;
pub mod role;
pub mod tool;
pub mod utils;

pub use annotation::Annotation;
pub use mcp_error as error;
pub use protocol::result::InitializeResult;
pub use resource::{MimeType, Resource, ResourceContents};
pub use role::Role;
pub use tool::{Tool, ToolCall};
