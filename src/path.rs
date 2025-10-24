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
            if let Err(err) = fs::remove_file(entry_path)
                && err.kind() != io::ErrorKind::NotFound
            {
                return Err(AppError::Io(err));
            }
        } else if entry.file_type().is_dir() {
            // Remove directory after its contents are removed
            // WalkDir visits in depth-first order, so subdirs are removed first
            if let Err(err) = fs::remove_dir(entry_path)
                && err.kind() != io::ErrorKind::NotFound
            {
                return Err(AppError::Io(err));
            }
        }
    }
    Ok(())
}
