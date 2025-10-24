use std::path::PathBuf;

use crate::config::Config;
use crate::error::AppError;
use crate::model::{Category, ScanReport};
use crate::scanner::Scanner;
use crate::utils::{display_path, format_bytes};

pub struct ScanOptions {
    pub categories: Vec<Category>,
    pub roots: Vec<PathBuf>,
    pub verbose: bool,
}

pub fn execute_scan(options: ScanOptions) -> Result<ScanReport, AppError> {
    let config = Config::load()?;
    let scanner = Scanner::new(config)?;
    let report = scanner.scan(&options.categories, &options.roots, options.verbose)?;

    print_report(&report, &options);
    Ok(report)
}

fn print_report(report: &ScanReport, options: &ScanOptions) {
    println!("Scan results:");
    for category in &options.categories {
        if let Some(category_report) = report.report_for(*category) {
            let total = category_report.total_size();
            println!(
                "- {:<8} {:>10} across {} item(s)",
                category,
                format_bytes(total),
                category_report.items.len()
            );
            if options.verbose {
                for item in &category_report.items {
                    println!(
                        "    • {:<60} {}",
                        display_path(item.path_str()),
                        format_bytes(item.size)
                    );
                }
            }
        }
    }
    println!("Total reclaimable: {}", format_bytes(report.total_size()));
}
