use std::path::PathBuf;

use walkdir::WalkDir;

use crate::error::AppError;
use crate::model::{Category, ScanItem};

use super::CategoryScanner;

const NODEJS_TARGETS: &[&str] = &[
    "node_modules",
    ".next",
    ".nuxt",
    ".svelte-kit",
];

pub struct NodejsScanner {
    exclude: Option<globset::GlobSet>,
}

impl NodejsScanner {
    pub fn new(exclude: Option<globset::GlobSet>) -> Self {
        Self { exclude }
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
}

impl CategoryScanner for NodejsScanner {
    fn scan(&self, roots: &[PathBuf], verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        let mut items = Vec::new();
        let target_names: std::collections::HashSet<&str> = NODEJS_TARGETS.iter().copied().collect();

        for root in roots {
            if !root.exists() {
                continue;
            }

            let mut walker = WalkDir::new(root).max_depth(10).into_iter();
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
                    if target_names.contains(name.as_ref()) {
                        let size = self.path_size(path, verbose)?;
                        items.push(ScanItem::directory(Category::Nodejs, path.to_path_buf(), size));
                        walker.skip_current_dir();
                    }
                }
            }
        }

        Ok(items)
    }

    fn category(&self) -> Category {
        Category::Nodejs
    }

    fn list_targets(&self, roots: &[PathBuf]) -> Result<Vec<String>, AppError> {
        let mut targets = Vec::new();
        let target_names: std::collections::HashSet<&str> = NODEJS_TARGETS.iter().copied().collect();
        let mut type_counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();

        for root in roots {
            if !root.exists() {
                continue;
            }

            let mut walker = WalkDir::new(root).max_depth(10).into_iter();
            while let Some(entry) = walker.next() {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => continue,
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
                    if target_names.contains(name.as_ref()) {
                        *type_counts.entry(name.to_string()).or_insert(0) += 1;
                        walker.skip_current_dir();
                    }
                }
            }
        }

        for (target_type, count) in type_counts {
            targets.push(format!("{} ({} location{} found)",
                target_type,
                count,
                if count == 1 { "" } else { "s" }
            ));
        }

        Ok(targets)
    }
}