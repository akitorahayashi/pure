use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use dirs_next as dirs;
use walkdir::WalkDir;

use crate::config::Config;
use crate::error::AppError;
use crate::model::{Category, ItemKind, ScanItem, ScanReport};

pub struct Scanner {
    exclude: Option<globset::GlobSet>,
}

impl Scanner {
    pub fn new(config: Config) -> Result<Self, AppError> {
        let exclude = config.compile_excludes()?;
        Ok(Self { exclude })
    }

    pub fn scan(
        &self,
        categories: &[Category],
        roots: &[PathBuf],
        verbose: bool,
    ) -> Result<ScanReport, AppError> {
        let mut report = ScanReport::new();
        for category in categories {
            let items = match category {
                Category::Dev => self.scan_dev(roots, verbose)?,
                Category::System => self.scan_system(verbose)?,
                Category::Logs => self.scan_logs(verbose)?,
                Category::Brew => self.scan_brew(verbose)?,
                Category::Browser => self.scan_browser(verbose)?,
                Category::Trash => self.scan_trash(verbose)?,
            };
            report.add_items(*category, items);
        }
        Ok(report)
    }

    pub fn delete_items(&self, items: &[ScanItem]) -> Result<(), AppError> {
        for item in items {
            if self.is_excluded(&item.path) {
                continue;
            }

            match item.kind {
                ItemKind::Directory => {
                    if let Err(err) = fs::remove_dir_all(&item.path)
                        && err.kind() != io::ErrorKind::NotFound
                    {
                        return Err(AppError::Io(err));
                    }
                }
                ItemKind::File => {
                    if let Err(err) = fs::remove_file(&item.path)
                        && err.kind() != io::ErrorKind::NotFound
                    {
                        return Err(AppError::Io(err));
                    }
                }
            }
        }
        Ok(())
    }

    fn scan_dev(&self, roots: &[PathBuf], verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        let mut items = Vec::new();
        let targets: HashSet<&str> = DEV_TARGETS.iter().copied().collect();
        for root in roots {
            if !root.exists() {
                continue;
            }

            let mut walker = WalkDir::new(root).into_iter();
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

                let path = entry.path();
                if self.is_excluded(path) {
                    if entry.file_type().is_dir() {
                        walker.skip_current_dir();
                    }
                    continue;
                }

                if entry.file_type().is_dir() {
                    let name = entry.file_name().to_string_lossy();
                    if targets.contains(name.as_ref()) {
                        let size = self.path_size(path, verbose)?;
                        items.push(ScanItem::directory(Category::Dev, path.to_path_buf(), size));
                        walker.skip_current_dir();
                    }
                }
            }
        }

        Ok(items)
    }

    fn scan_system(&self, verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        self.collect_directories(Category::System, system_paths(), verbose)
    }

    fn scan_logs(&self, verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        self.collect_directories(Category::Logs, log_paths(), verbose)
    }

    fn scan_brew(&self, verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        self.collect_directories(Category::Brew, brew_paths(), verbose)
    }

    fn scan_browser(&self, verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        self.collect_directories(Category::Browser, browser_paths(), verbose)
    }

    fn scan_trash(&self, verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        self.collect_directories(Category::Trash, trash_paths(), verbose)
    }

    fn collect_directories(
        &self,
        category: Category,
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
                items.push(ScanItem::directory(category, path, size));
            }
        }
        Ok(items)
    }

    fn path_size(&self, path: &Path, verbose: bool) -> Result<u64, AppError> {
        if path.is_file() {
            Ok(path.metadata()?.len())
        } else {
            let mut total = 0u64;
            let mut walker = WalkDir::new(path).into_iter();
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

    fn is_excluded(&self, path: &Path) -> bool {
        if let Some(set) = &self.exclude {
            let candidate = if path.is_absolute() {
                path.to_string_lossy().to_string()
            } else {
                // For relative paths, try to make absolute by joining with current dir
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
}

const DEV_TARGETS: &[&str] = &[
    "__pycache__",
    ".pytest_cache",
    ".ruff_cache",
    ".mypy_cache",
    ".cache",
    "target",
    "node_modules",
    "build",
    ".next",
    ".nuxt",
    ".svelte-kit",
    "DerivedData",
    ".gradle",
    ".venv",
    ".uv-cache",
];

fn system_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join("Library").join("Caches"));
    }
    paths.push(PathBuf::from("/Library/Caches"));
    paths
}

fn log_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join("Library/Logs"));
    }
    paths
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

fn browser_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join("Library/Caches/com.apple.Safari"));
        paths.push(home.join("Library/Application Support/Google/Chrome/Default/Cache"));
        // Firefox caches live under Library/Caches, not Application Support.
        // This aggregates per-profile cache dirs without touching profile data.
        paths.push(home.join("Library/Caches/Firefox/Profiles"));
    }
    paths
}

fn trash_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".Trash"));
    }
    paths
}
