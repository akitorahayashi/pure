use super::{CategoryScanner, GenericScanner};
use crate::error::AppError;
use crate::model::{Category, ScanItem};
use std::path::PathBuf;

const XCODE_TARGETS: &[&str] = &["DerivedData"];

pub struct XcodeScanner(GenericScanner);

impl XcodeScanner {
    pub fn new(exclude: Option<globset::GlobSet>) -> Self {
        Self(GenericScanner::new(Category::Xcode, XCODE_TARGETS, exclude))
    }
}

impl std::ops::Deref for XcodeScanner {
    type Target = GenericScanner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CategoryScanner for XcodeScanner {
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
