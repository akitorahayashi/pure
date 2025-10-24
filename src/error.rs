use std::io;

use thiserror::Error;

/// Application-wide error type for the pure CLI.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unknown category '{0}'")]
    InvalidCategory(String),

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Failed to launch editor: {0}")]
    Editor(String),

    #[error("Failed to parse configuration: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Failed to write configuration: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("Invalid exclude pattern: {0}")]
    Glob(#[from] globset::Error),
}

impl AppError {
    pub fn config<S: Into<String>>(msg: S) -> Self {
        AppError::Config(msg.into())
    }
}
