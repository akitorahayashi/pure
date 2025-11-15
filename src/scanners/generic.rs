use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use walkdir::WalkDir;

use super::CategoryScanner;
use crate::error::AppError;
use crate::model::{Category, ScanItem};
use crate::path::is_excluded;

pub struct GenericScanner {
    category: Category,
    targets: &'static [&'static str],
    exclude: Option<globset::GlobSet>,
}

impl GenericScanner {
    pub fn new(
        category: Category,
        targets: &'static [&'static str],
        exclude: Option<globset::GlobSet>,
    ) -> Self {
        Self { category, targets, exclude }
    }
}

impl CategoryScanner for GenericScanner {
    fn scan(&self, roots: &[PathBuf], verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        let mut items = Vec::new();
        let target_names: HashSet<&str> = self.targets.iter().copied().collect();

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
                if is_excluded(path, self.exclude.as_ref()) {
                    if entry.file_type().is_dir() {
                        walker.skip_current_dir();
                    }
                    continue;
                }

                if entry.file_type().is_dir() {
                    let name = entry.file_name().to_string_lossy();
                    if target_names.contains(name.as_ref()) {
                        items.push(ScanItem::directory(self.category, path.to_path_buf(), 0));
                        walker.skip_current_dir();
                    }
                }
            }
        }

        Ok(items)
    }

    fn category(&self) -> Category {
        self.category
    }

    fn list_targets(&self, roots: &[PathBuf]) -> Result<Vec<String>, AppError> {
        let mut targets = Vec::new();
        let target_names: HashSet<&str> = self.targets.iter().copied().collect();
        let mut type_counts: BTreeMap<String, usize> = BTreeMap::new();

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
                if is_excluded(path, self.exclude.as_ref()) {
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
            targets.push(format!(
                "{} ({} location{} found)",
                target_type,
                count,
                if count == 1 { "" } else { "s" }
            ));
        }

        Ok(targets)
    }
}
