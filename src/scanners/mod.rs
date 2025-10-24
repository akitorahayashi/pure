use std::path::PathBuf;

use crate::error::AppError;
use crate::model::{Category, ScanItem};

pub mod xcode;
pub mod python;
pub mod rust;
pub mod nodejs;
pub mod brew;

pub use xcode::XcodeScanner;
pub use python::PythonScanner;
pub use rust::RustScanner;
pub use nodejs::NodejsScanner;
pub use brew::BrewScanner;

/// Trait that all category scanners must implement
pub trait CategoryScanner: Send + Sync {
    /// Scan for items in this category
    fn scan(&self, roots: &[PathBuf], verbose: bool) -> Result<Vec<ScanItem>, AppError>;

    /// Get the category this scanner handles
    fn category(&self) -> Category;

    /// List existing targets without calculating sizes (fast operation)
    fn list_targets(&self, roots: &[PathBuf]) -> Result<Vec<String>, AppError>;
}
