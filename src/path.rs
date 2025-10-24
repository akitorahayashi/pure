use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use dirs_next as dirs;
use walkdir::WalkDir;

/// Replace the home directory prefix with `~` to make output easier to read.
pub fn display_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir()
        && let Ok(stripped) = path.strip_prefix(&home)
    {
        let mut display = PathBuf::from("~");
        display.push(stripped);
        return display.display().to_string();
    }

    path.display().to_string()
}

pub fn resolve_roots(explicit: &[PathBuf]) -> Vec<PathBuf> {
    if explicit.is_empty() {
        if let Some(home) = dirs::home_dir() {
            vec![home.join("Desktop")]
        } else {
            vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
        }
    } else {
        explicit.to_vec()
    }
}

pub fn resolve_roots_with_current(explicit: &[PathBuf], current: bool) -> Vec<PathBuf> {
    if current {
        vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
    } else if explicit.is_empty() {
        if let Some(home) = dirs::home_dir() {
            vec![home.join("Desktop")]
        } else {
            vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
        }
    } else {
        explicit.to_vec()
    }
}

pub fn is_excluded(path: &Path, exclude: Option<&globset::GlobSet>) -> bool {
    if let Some(set) = exclude {
        if path.is_absolute() {
            return set.is_match(path);
        }

        match std::env::current_dir() {
            Ok(cwd) => set.is_match(cwd.join(path)),
            Err(e) => {
                eprintln!(
                    "Warning: could not get current directory to check exclusion for relative path {}: {}",
                    path.display(),
                    e
                );
                false
            }
        }
    } else {
        false
    }
}

pub fn path_size(
    path: &Path,
    exclude: Option<&globset::GlobSet>,
    verbose: bool,
) -> Result<u64, AppError> {
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
            if is_excluded(entry_path, exclude) {
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

pub fn safe_remove_dir_all(
    path: &Path,
    exclude: Option<&globset::GlobSet>,
    verbose: bool,
) -> Result<(), AppError> {
    let mut files_to_remove = Vec::new();
    let mut dirs_to_remove = Vec::new();

    let mut walker = WalkDir::new(path).into_iter();
    while let Some(entry_result) = walker.next() {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(err) => {
                if verbose {
                    eprintln!("Skipping due to error: {}", err);
                }
                continue;
            }
        };

        let entry_path = entry.path();
        if is_excluded(entry_path, exclude) {
            if entry.file_type().is_dir() {
                walker.skip_current_dir();
            }
            continue;
        }

        if entry.file_type().is_file() {
            files_to_remove.push(entry.into_path());
        } else if entry.file_type().is_dir() {
            dirs_to_remove.push(entry.into_path());
        }
    }

    // Remove files first.
    for file in &files_to_remove {
        match fs::remove_file(file) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(AppError::Io(err)),
        }
    }

    // Remove directories, deepest first.
    dirs_to_remove.sort_by_key(|p| std::cmp::Reverse(p.as_os_str().len()));
    for dir in &dirs_to_remove {
        match fs::remove_dir(dir) {
            Ok(()) => {}
            Err(err)
                if err.kind() == io::ErrorKind::NotFound
                    || err.kind() == io::ErrorKind::DirectoryNotEmpty =>
            {
                // Not empty can happen if it contains an excluded item.
            }
            Err(err) => return Err(AppError::Io(err)),
        }
    }

    Ok(())
}
