use std::path::PathBuf;

use dirs_next as dirs;

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
    } else {
        resolve_roots(explicit)
    }
}
