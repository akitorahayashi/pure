use super::{CategoryScanner, GenericScanner};
use crate::error::AppError;
use crate::model::{Category, ScanItem};
use std::path::PathBuf;

const NODEJS_TARGETS: &[&str] = &["node_modules", ".next", ".nuxt", ".svelte-kit"];

pub struct NodejsScanner(GenericScanner);

impl NodejsScanner {
    pub fn new(exclude: Option<globset::GlobSet>) -> Self {
        Self(GenericScanner::new(Category::Nodejs, NODEJS_TARGETS, exclude))
    }
}

impl std::ops::Deref for NodejsScanner {
    type Target = GenericScanner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CategoryScanner for NodejsScanner {
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
