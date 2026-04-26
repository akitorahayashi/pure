use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use dirs_next as dirs;

use crate::targets::category::Category;
use crate::targets::report::ScanReport;

use super::bytes::format_bytes;

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

pub fn print_scan_report(report: &ScanReport, categories: &[Category], verbose: bool) {
    println!("Scan results:");
    for category in categories {
        if let Some(category_report) = report.report_for(*category) {
            let total = category_report.total_size();
            println!(
                "- {:<8} {:>10} across {} item(s)",
                category,
                format_bytes(total),
                category_report.items.len()
            );
            if verbose {
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

pub fn print_list_results(results: &BTreeMap<Category, Vec<String>>) {
    println!("Found cleanup targets:");
    for (category, targets) in results {
        if !targets.is_empty() {
            println!("【{}】", category.display_name());
            for target in targets {
                println!("- {}", target);
            }
            println!();
        }
    }
}

pub fn print_deletion_plan(report: &ScanReport, categories: &[Category], verbose: bool) {
    println!("Deletion plan:");
    for category in categories {
        if let Some(category_report) = report.report_for(*category) {
            println!(
                "- {:<8} {:>10} across {} item(s)",
                category,
                format_bytes(category_report.total_size()),
                category_report.items.len()
            );
            for item in &category_report.items {
                if verbose {
                    println!(
                        "    • {:<60} {}",
                        display_path(item.path_str()),
                        format_bytes(item.size)
                    );
                } else {
                    println!("    • {}", display_path(item.path_str()));
                }
            }
        }
    }
    println!("Total to delete: {}", format_bytes(report.total_size()));
}
