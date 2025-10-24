use std::path::{Path, PathBuf};

use dirs_next as dirs;

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
