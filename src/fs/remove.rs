use std::fs;
use std::io;
use std::path::Path;

use walkdir::WalkDir;

use crate::error::AppError;
use crate::targets::item::ItemKind;

pub fn remove_item(path: &Path, kind: ItemKind, verbose: bool) -> Result<(), AppError> {
    match kind {
        ItemKind::Directory => safe_remove_dir_all(path, verbose),
        ItemKind::File => match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(AppError::Io(err)),
        },
    }
}

pub fn safe_remove_dir_all(path: &Path, verbose: bool) -> Result<(), AppError> {
    let mut files_to_remove = Vec::new();
    let mut dirs_to_remove = Vec::new();

    for entry_result in WalkDir::new(path).into_iter() {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(err) => {
                if verbose {
                    eprintln!("Skipping due to error: {}", err);
                }
                continue;
            }
        };

        if entry.file_type().is_file() {
            files_to_remove.push(entry.into_path());
        } else if entry.file_type().is_dir() {
            dirs_to_remove.push(entry.into_path());
        }
    }

    for file in &files_to_remove {
        match fs::remove_file(file) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(AppError::Io(err)),
        }
    }

    dirs_to_remove.sort_by_key(|candidate| std::cmp::Reverse(candidate.as_os_str().len()));
    for dir in &dirs_to_remove {
        match fs::remove_dir(dir) {
            Ok(()) => {}
            Err(err)
                if err.kind() == io::ErrorKind::NotFound
                    || err.kind() == io::ErrorKind::DirectoryNotEmpty => {}
            Err(err) => return Err(AppError::Io(err)),
        }
    }

    Ok(())
}
