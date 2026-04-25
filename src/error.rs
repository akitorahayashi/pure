use std::io;

use thiserror::Error;

/// Application-wide error type for the prf CLI (purify).
#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Unknown category '{0}'")]
    InvalidCategory(String),

    #[error("Operation cancelled by user")]
    Cancelled,
}
