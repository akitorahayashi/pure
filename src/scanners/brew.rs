use std::path::PathBuf;

use dirs_next as dirs;

use crate::error::AppError;
use crate::model::{Category, ScanItem};
use crate::path::{is_excluded, path_size};

use super::CategoryScanner;

pub struct BrewScanner {
    exclude: Option<globset::GlobSet>,
}

impl BrewScanner {
    pub fn new(exclude: Option<globset::GlobSet>) -> Self {
        Self { exclude }
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

    fn collect_directories(
        &self,
        paths: Vec<PathBuf>,
        verbose: bool,
    ) -> Result<Vec<ScanItem>, AppError> {
        let mut items = Vec::new();
        for path in paths {
            if is_excluded(&path, self.exclude.as_ref()) {
                continue;
            }
            if path.exists() {
                let size = match path_size(&path, self.exclude.as_ref(), verbose) {
                    Ok(size) => size,
                    Err(err) => {
                        if verbose {
                            eprintln!("Skipping {}: {}", path.display(), err);
                        }
                        continue;
                    }
                };
                items.push(ScanItem::directory(Category::Brew, path, size));
            }
        }
        Ok(items)
    }
}

impl CategoryScanner for BrewScanner {
    fn scan(&self, _roots: &[PathBuf], verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        self.collect_directories(Self::brew_paths(), verbose)
    }

    fn category(&self) -> Category {
        Category::Brew
    }

    fn list_targets(&self, _roots: &[PathBuf]) -> Result<Vec<String>, AppError> {
        let mut targets = Vec::new();
        let paths = Self::brew_paths();

        for path in paths {
            if !is_excluded(&path, self.exclude.as_ref()) && path.exists() {
                targets.push(format!("{} (exists)", path.display()));
            }
        }

        Ok(targets)
    }
}
