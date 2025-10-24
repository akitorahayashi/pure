use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Xcode,
    Python,
    Rust,
    Nodejs,
    Brew,
}

impl Category {
    pub const ALL: [Category; 5] = [
        Category::Xcode,
        Category::Python,
        Category::Rust,
        Category::Nodejs,
        Category::Brew,
    ];

    pub fn from_name(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "xcode" => Some(Category::Xcode),
            "python" => Some(Category::Python),
            "rust" => Some(Category::Rust),
            "nodejs" => Some(Category::Nodejs),
            "brew" => Some(Category::Brew),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Xcode => "xcode",
            Category::Python => "python",
            Category::Rust => "rust",
            Category::Nodejs => "nodejs",
            Category::Brew => "brew",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Category::Xcode => "Xcode",
            Category::Python => "Python",
            Category::Rust => "Rust",
            Category::Nodejs => "NodeJS",
            Category::Brew => "Homebrew",
        }
    }
}

impl std::str::FromStr for Category {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Category::from_name(s).ok_or_else(|| format!("Unknown category '{s}'"))
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct ScanItem {
    pub category: Category,
    pub path: PathBuf,
    pub size: u64,
    pub kind: ItemKind,
}

impl ScanItem {
    pub fn directory(category: Category, path: PathBuf, size: u64) -> Self {
        ScanItem { category, path, size, kind: ItemKind::Directory }
    }

    pub fn file(category: Category, path: PathBuf, size: u64) -> Self {
        ScanItem { category, path, size, kind: ItemKind::File }
    }

    pub fn is_zero(&self) -> bool {
        self.size == 0
    }

    pub fn path_str(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Clone)]
pub struct CategoryReport {
    pub category: Category,
    pub items: Vec<ScanItem>,
}

impl CategoryReport {
    pub fn new(category: Category) -> Self {
        Self { category, items: Vec::new() }
    }

    pub fn total_size(&self) -> u64 {
        self.items.iter().map(|item| item.size).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ScanReport {
    pub categories: BTreeMap<Category, CategoryReport>,
}

impl ScanReport {
    pub fn new() -> Self {
        Self { categories: BTreeMap::new() }
    }

    pub fn add_items(&mut self, category: Category, mut items: Vec<ScanItem>) {
        let entry =
            self.categories.entry(category).or_insert_with(|| CategoryReport::new(category));
        entry.items.append(&mut items);
    }

    pub fn total_size(&self) -> u64 {
        self.categories.values().map(CategoryReport::total_size).sum()
    }

    pub fn categories(&self) -> Vec<Category> {
        self.categories.keys().copied().collect()
    }

    pub fn report_for(&self, category: Category) -> Option<&CategoryReport> {
        self.categories.get(&category)
    }

    pub fn subset(&self, categories: &[Category]) -> Self {
        let mut subset = ScanReport::new();
        for category in categories {
            if let Some(report) = self.categories.get(category) {
                subset.categories.insert(*category, report.clone());
            }
        }
        subset
    }

    pub fn is_empty(&self) -> bool {
        self.categories.values().all(CategoryReport::is_empty)
    }
}

impl Default for ScanReport {
    fn default() -> Self {
        ScanReport::new()
    }
}
