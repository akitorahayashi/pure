//! Library entry point for the prf CLI (purify).

pub mod app;
pub mod error;
pub mod fs;
pub mod output;
pub mod targets;

#[path = "cli/mod.rs"]
mod cli_entry;

pub use cli_entry::run as cli;
pub use error::AppError;
