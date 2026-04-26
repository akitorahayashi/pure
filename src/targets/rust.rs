use crate::error::AppError;

use super::category::Category;
use super::item::CleanupItem;
use super::name_matcher::NameMatcherTarget;
use super::target::{CleanupTarget, ScanScope};

const RUST_TARGETS: &[&str] = &["target"];

pub struct RustTarget(NameMatcherTarget);

impl RustTarget {
    pub fn new() -> Self {
        Self(NameMatcherTarget::new(Category::Rust, RUST_TARGETS))
    }
}

impl Default for RustTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanupTarget for RustTarget {
    fn category(&self) -> Category {
        self.0.category()
    }

    fn discover(&self, scope: &ScanScope) -> Result<Vec<CleanupItem>, AppError> {
        self.0.discover(scope)
    }

    fn list(&self, scope: &ScanScope) -> Result<Vec<String>, AppError> {
        self.0.list(scope)
    }
}
