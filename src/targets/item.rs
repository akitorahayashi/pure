use std::path::{Path, PathBuf};

use super::category::Category;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct CleanupItem {
    pub category: Category,
    pub path: PathBuf,
    pub size: u64,
    pub kind: ItemKind,
}

impl CleanupItem {
    pub fn directory(category: Category, path: PathBuf, size: u64) -> Self {
        Self { category, path, size, kind: ItemKind::Directory }
    }

    pub fn file(category: Category, path: PathBuf, size: u64) -> Self {
        Self { category, path, size, kind: ItemKind::File }
    }

    pub fn is_zero(&self) -> bool {
        self.size == 0
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
