use std::io;

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("serde_json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("system error: {0}")]
    System(String),

    #[error("invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Mcp error: {0}")]
    McpError(#[from] mcp_core::error::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Invalid UTF-8 sequence: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = core::result::Result<T, Error>;

pub type BoxError = Box<dyn std::error::Error + Sync + Send>;
