use super::{CategoryScanner, GenericScanner};
use crate::error::AppError;
use crate::model::{Category, ScanItem};
use std::path::PathBuf;

const PYTHON_TARGETS: &[&str] =
    &["__pycache__", ".pytest_cache", ".ruff_cache", ".mypy_cache", ".venv", ".uv-cache"];

pub struct PythonScanner(GenericScanner);

impl PythonScanner {
    pub fn new() -> Self {
        Self(GenericScanner::new(Category::Python, PYTHON_TARGETS))
    }
}

impl Default for PythonScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for PythonScanner {
    type Target = GenericScanner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CategoryScanner for PythonScanner {
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
