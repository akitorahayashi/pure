use std::path::{Path, PathBuf};

use byte_unit::{Byte, UnitType};

/// Format bytes into a human-readable string.
pub fn format_bytes(size: u64) -> String {
    if size == 0 {
        "0 B".to_string()
    } else {
        let adjusted = Byte::from_u64(size).get_appropriate_unit(UnitType::Decimal);
        format!("{adjusted:#.2}")
    }
}

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
            vec![home]
        } else {
            vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
        }
    } else {
        explicit.to_vec()
    }
}
