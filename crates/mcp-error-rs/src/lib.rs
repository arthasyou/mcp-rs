use std::io;

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("url error: {0}")]
    Url(#[from] url::ParseError),

    #[error("serde_json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("system error: {0}")]
    System(String),

    #[error("Invalid UTF-8 sequence: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Transport was not connected or is already closed")]
    NotConnected,

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Unsupported message type. JsonRpcMessage can only be Request or Notification.")]
    UnsupportedMessage,

    #[error("Stdio process error: {0}")]
    StdioProcessError(String),

    #[error("SSE connection error: {0}")]
    SseConnection(String),

    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },

    #[error("config error: {0}")]
    ServiceError(#[from] service_utils_rs::error::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

pub type BoxError = Box<dyn std::error::Error + Sync + Send>;
