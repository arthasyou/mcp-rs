use std::io;

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("url error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("system error: {0}")]
    SystemError(String),

    #[error("invalid parameters: {0}")]
    InvalidParameters(String),
}

pub type Result<T> = core::result::Result<T, Error>;
