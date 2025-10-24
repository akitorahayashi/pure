use super::{CategoryScanner, GenericScanner};
use crate::error::AppError;
use crate::model::{Category, ScanItem};
use std::path::PathBuf;

const RUST_TARGETS: &[&str] = &["target"];

pub struct RustScanner(GenericScanner);

impl RustScanner {
    pub fn new(exclude: Option<globset::GlobSet>) -> Self {
        Self(GenericScanner::new(Category::Rust, RUST_TARGETS, exclude))
    }
}

impl std::ops::Deref for RustScanner {
    type Target = GenericScanner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CategoryScanner for RustScanner {
    fn scan(&self, roots: &[PathBuf], verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        self.0.scan(roots, verbose)
    }

    fn category(&self) -> Category {
        self.0.category()
    }

    fn list_targets(&self, roots: &[PathBuf]) -> Result<Vec<String>, AppError> {
        self.0.list_targets(roots)
    }
}
