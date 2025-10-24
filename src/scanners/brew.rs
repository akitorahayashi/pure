use std::path::PathBuf;

use dirs_next as dirs;

use crate::error::AppError;
use crate::model::{Category, ScanItem};

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

    fn is_excluded(&self, path: &std::path::Path) -> bool {
        if let Some(set) = &self.exclude {
            let candidate = if path.is_absolute() {
                path.to_string_lossy().to_string()
            } else {
                match std::env::current_dir() {
                    Ok(cwd) => {
                        let joined = cwd.join(path);
                        joined.to_string_lossy().to_string()
                    }
                    Err(_) => path.to_string_lossy().to_string(),
                }
            };
            set.is_match(&candidate)
        } else {
            false
        }
    }

    fn path_size(&self, path: &std::path::Path, verbose: bool) -> Result<u64, AppError> {
        if path.is_file() {
            Ok(path.metadata()?.len())
        } else {
            let mut total = 0u64;
            let mut walker = walkdir::WalkDir::new(path).into_iter();
            while let Some(entry) = walker.next() {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        if verbose {
                            eprintln!("Skipping {:?}: {}", err.path(), err);
                        }
                        continue;
                    }
                };

                let entry_path = entry.path();
                if self.is_excluded(entry_path) {
                    if entry.file_type().is_dir() {
                        walker.skip_current_dir();
                    }
                    continue;
                }

                if entry.file_type().is_file() {
                    match entry.metadata() {
                        Ok(metadata) => {
                            total = total.saturating_add(metadata.len());
                        }
                        Err(err) => {
                            if verbose {
                                eprintln!("Skipping {}: {}", entry_path.display(), err);
                            }
                        }
                    }
                }
            }
            Ok(total)
        }
    }

    fn collect_directories(
        &self,
        paths: Vec<PathBuf>,
        verbose: bool,
    ) -> Result<Vec<ScanItem>, AppError> {
        let mut items = Vec::new();
        for path in paths {
            if self.is_excluded(&path) {
                continue;
            }
            if path.exists() {
                let size = match self.path_size(&path, verbose) {
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
            if !self.is_excluded(&path) && path.exists() {
                targets.push(format!("{} (exists)", path.display()));
            }
        }

        Ok(targets)
    }
}
