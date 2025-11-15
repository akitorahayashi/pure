use super::CategoryScanner;
use crate::error::AppError;
use crate::model::{Category, ItemKind, ScanItem};
use crate::path::is_excluded;
use dirs_next as dirs;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct XcodeScanner {
    exclude: Option<globset::GlobSet>,
    current: bool,
}

impl XcodeScanner {
    pub fn new(exclude: Option<globset::GlobSet>, current: bool) -> Self {
        Self { exclude, current }
    }

    fn global_safe_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let lib = home.join("Library");
            paths.push(lib.join("Developer/Xcode/DerivedData"));
            paths.push(lib.join("Caches/com.apple.dt.Xcode"));
            paths.push(lib.join("Developer/Xcode/DocumentationCache"));
            paths.push(lib.join("Developer/Xcode/DocumentationIndex"));
            paths.push(lib.join("Developer/Xcode/UserData/Previews"));
            paths.push(lib.join("Caches/org.swift.swiftpm"));
            paths.push(lib.join("org.swift.swiftpm"));
            paths.push(lib.join("Developer/CoreSimulator/Caches"));
        }
        paths
    }

    fn add_path(&self, path: &Path, items: &mut Vec<ScanItem>) {
        if is_excluded(path, self.exclude.as_ref()) {
            return;
        }

        let kind = if path.is_file() { ItemKind::File } else { ItemKind::Directory };
        items.push(ScanItem { category: Category::Xcode, path: path.to_path_buf(), size: 0, kind });
    }

    fn collect_swiftpm_artifacts(&self, parent: &Path, items: &mut Vec<ScanItem>) {
        const ARTIFACTS: &[&str] = &[".build", ".swiftpm", "Package.resolved"];
        for artifact in ARTIFACTS {
            let artifact_path = parent.join(artifact);
            if artifact_path.exists() {
                self.add_path(&artifact_path, items);
            }
        }
    }

    fn scan_global_caches(&self) -> Vec<ScanItem> {
        let mut items = Vec::new();
        for path in Self::global_safe_paths() {
            if path.exists() {
                self.add_path(&path, &mut items);
            }
        }
        items
    }

    fn scan_local_projects(&self, roots: &[PathBuf], verbose: bool) -> Vec<ScanItem> {
        let mut items = Vec::new();
        let mut processed_packages: HashSet<PathBuf> = HashSet::new();

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

                let file_name = entry.file_name().to_string_lossy();

                if entry.file_type().is_dir() && file_name == "DerivedData" {
                    self.add_path(path, &mut items);
                    walker.skip_current_dir();
                    continue;
                }

                if entry.file_type().is_file()
                    && file_name == "Package.swift"
                    && let Some(parent) = path.parent()
                    && processed_packages.insert(parent.to_path_buf())
                {
                    self.collect_swiftpm_artifacts(parent, &mut items);
                }
            }
        }

        items
    }

    fn list_global_targets(&self) -> Vec<String> {
        let mut targets = Vec::new();
        for path in Self::global_safe_paths() {
            if path.exists() && !is_excluded(&path, self.exclude.as_ref()) {
                targets.push(format!("{} (exists)", path.display()));
            }
        }
        targets
    }

    fn list_local_targets(&self, roots: &[PathBuf]) -> Vec<String> {
        let mut targets = Vec::new();
        let mut derived_data = 0usize;
        let mut swiftpm_projects = 0usize;

        for root in roots {
            if !root.exists() {
                continue;
            }

            let mut walker = WalkDir::new(root).max_depth(10).into_iter();
            while let Some(entry) = walker.next() {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        eprintln!("Skipping {:?}: {}", err.path(), err);
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

                let file_name = entry.file_name().to_string_lossy();
                if entry.file_type().is_dir() && file_name == "DerivedData" {
                    derived_data += 1;
                    walker.skip_current_dir();
                } else if entry.file_type().is_file() && file_name == "Package.swift" {
                    swiftpm_projects += 1;
                }
            }
        }

        if derived_data > 0 {
            targets.push(format!(
                "DerivedData ({} location{} found)",
                derived_data,
                if derived_data == 1 { "" } else { "s" }
            ));
        }

        if swiftpm_projects > 0 {
            targets.push(format!(
                "SwiftPM Projects (.build, .swiftpm, Package.resolved) ({} location{} found)",
                swiftpm_projects,
                if swiftpm_projects == 1 { "" } else { "s" }
            ));
        }

        targets
    }
}

impl CategoryScanner for XcodeScanner {
    fn scan(&self, roots: &[PathBuf], verbose: bool) -> Result<Vec<ScanItem>, AppError> {
        let mut items = self.scan_local_projects(roots, verbose);
        if !self.current {
            let mut global_items = self.scan_global_caches();
            items.append(&mut global_items);
        }
        Ok(items)
    }

    fn category(&self) -> Category {
        Category::Xcode
    }

    fn list_targets(&self, roots: &[PathBuf]) -> Result<Vec<String>, AppError> {
        let mut targets = self.list_local_targets(roots);
        if !self.current {
            let mut global = self.list_global_targets();
            targets.append(&mut global);
        }
        Ok(targets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use serial_test::serial;
    use std::env;

    #[test]
    fn scan_detects_local_derived_data() {
        let temp = TempDir::new().unwrap();
        let project_root = temp.child("workspace");
        project_root.create_dir_all().unwrap();
        let derived = project_root.child("DerivedData/cache");
        derived.create_dir_all().unwrap();
        derived.child("foo.txt").write_str("cache").unwrap();

        let scanner = XcodeScanner::new(None, false);
        let items =
            scanner.scan(&[project_root.path().to_path_buf()], true).expect("scan succeeds");

        assert!(
            items.iter().any(|item| item.path.ends_with("DerivedData")),
            "expected DerivedData directory to be reported"
        );
    }

    #[test]
    fn scan_detects_swiftpm_artifacts_only_with_package_swift() {
        let temp = TempDir::new().unwrap();
        let roots = temp.child("workspace");
        roots.create_dir_all().unwrap();

        let pkg = roots.child("AppWithPackage");
        pkg.create_dir_all().unwrap();
        pkg.child("Package.swift").write_str("// swift package").unwrap();
        pkg.child(".build/output.o").write_str("bin").unwrap();
        pkg.child(".swiftpm/config").write_str("cfg").unwrap();
        pkg.child("Package.resolved").write_str("deps").unwrap();

        let no_pkg = roots.child("AppWithoutPackage");
        no_pkg.create_dir_all().unwrap();
        no_pkg.child(".build/output.o").write_str("bin").unwrap();

        let scanner = XcodeScanner::new(None, false);
        let items = scanner.scan(&[roots.path().to_path_buf()], true).expect("scan succeeds");

        assert!(
            items.iter().any(|item| item.path.to_string_lossy().contains("AppWithPackage/.build")),
            ".build directory should be reported when Package.swift exists"
        );
        assert!(
            !items
                .iter()
                .any(|item| item.path.to_string_lossy().contains("AppWithoutPackage/.build")),
            "Projects without Package.swift should be ignored"
        );
    }

    #[test]
    #[serial]
    fn scan_global_caches_respects_current_flag() {
        let temp_home = TempDir::new().unwrap();
        let derived = temp_home.child("Library/Developer/Xcode/DerivedData/project");
        derived.create_dir_all().unwrap();
        derived.child("foo.txt").write_str("cache").unwrap();

        let original_home = env::var("HOME").ok();
        unsafe {
            env::set_var("HOME", temp_home.path());
        }

        let scanner = XcodeScanner::new(None, false);
        let items = scanner.scan(&[], false).expect("scan succeeds");
        assert!(
            items.iter().any(|item| item
                .path
                .to_string_lossy()
                .contains("Library/Developer/Xcode/DerivedData")),
            "global caches should be detected when not in current-only mode"
        );

        let current_scanner = XcodeScanner::new(None, true);
        let current_items = current_scanner.scan(&[], false).expect("scan succeeds");
        assert!(current_items.is_empty(), "--current should skip global caches");

        if let Some(home) = original_home {
            unsafe {
                env::set_var("HOME", home);
            }
        } else {
            unsafe {
                env::remove_var("HOME");
            }
        }
    }
}
