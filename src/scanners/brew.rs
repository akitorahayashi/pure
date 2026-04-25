use std::path::PathBuf;

use dirs_next as dirs;

use crate::error::AppError;
use crate::model::{Category, ScanItem};

use super::CategoryScanner;

pub struct BrewScanner {}

impl BrewScanner {
    pub fn new() -> Self {
        Self {}
    }

    fn brew_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Library/Caches/Homebrew"));
        }
        paths.push(PathBuf::from("/opt/homebrew/Library/Caches"));
        paths.push(PathBuf::from("/usr/local/Homebrew/Library/Logs"));
        paths
    }

    fn collect_directories(&self, paths: Vec<PathBuf>) -> Result<Vec<ScanItem>, AppError> {
        let mut items = Vec::new();
        for path in paths {
            if path.exists() {
                items.push(ScanItem::directory(Category::Brew, path, 0));
            }
        }
        Ok(items)
    }
}

impl Default for BrewScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl CategoryScanner for BrewScanner {
    fn scan(&self, _roots: &[PathBuf], _verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        self.collect_directories(Self::brew_paths())
    }

    fn category(&self) -> Category {
        Category::Brew
    }

    fn list_targets(&self, _roots: &[PathBuf]) -> Result<Vec<String>, AppError> {
        let mut targets = Vec::new();
        let paths = Self::brew_paths();

        for path in paths {
            if path.exists() {
                targets.push(format!("{} (exists)", path.display()));
            }
        }

        Ok(targets)
    }
}
