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

    #[error("invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
}

pub type Result<T> = core::result::Result<T, Error>;

pub type BoxError = Box<dyn std::error::Error + Sync + Send>;
