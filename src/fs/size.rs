use std::path::Path;

use walkdir::WalkDir;

use crate::error::AppError;

pub fn path_size(path: &Path, verbose: bool) -> Result<u64, AppError> {
    if path.is_file() {
        Ok(path.metadata()?.len())
    } else {
        let mut total = 0u64;
        for entry in WalkDir::new(path).into_iter() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    if verbose {
                        eprintln!("Skipping {:?}: {}", err.path(), err);
                    }
                    continue;
                }
            };

            if entry.file_type().is_file() {
                match entry.metadata() {
                    Ok(metadata) => {
                        total = total.saturating_add(metadata.len());
                    }
                    Err(err) => {
                        if verbose {
                            eprintln!("Skipping {}: {}", entry.path().display(), err);
                        }
                    }
                }
            }
        }
        Ok(total)
    }
}
