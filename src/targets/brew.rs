use std::path::PathBuf;

use dirs_next as dirs;

use crate::error::AppError;

use super::category::Category;
use super::item::CleanupItem;
use super::target::{CleanupTarget, ScanScope};

pub struct BrewTarget;

impl BrewTarget {
    pub fn new() -> Self {
        Self
    }

    fn brew_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Library/Caches/Homebrew"));
            paths.push(home.join("Library/Logs/Homebrew"));
        }
        paths
    }
}

impl Default for BrewTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanupTarget for BrewTarget {
    fn category(&self) -> Category {
        Category::Brew
    }

    fn discover(&self, _scope: &ScanScope) -> Result<Vec<CleanupItem>, AppError> {
        let mut items = Vec::new();
        for path in Self::brew_paths() {
            if path.exists() {
                items.push(CleanupItem::directory(Category::Brew, path, 0));
            }
        }
        Ok(items)
    }

    fn list(&self, _scope: &ScanScope) -> Result<Vec<String>, AppError> {
        let mut targets = Vec::new();
        for path in Self::brew_paths() {
            if path.exists() {
                targets.push(format!("{} (exists)", path.display()));
            }
        }
        Ok(targets)
    }
}
