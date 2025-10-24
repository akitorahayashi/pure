use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::AppError;
use crate::format::format_bytes;
use crate::model::{Category, ScanReport};
use crate::path::display_path;
use crate::scanners::*;

pub struct ScanOptions {
    pub categories: Vec<Category>,
    pub roots: Vec<PathBuf>,
    pub verbose: bool,
    pub list: bool,
}

pub fn execute_scan(options: ScanOptions) -> Result<ScanReport, AppError> {
    let config = Config::load()?;
    let exclude = config.compile_excludes()?;

    if options.list {
        let list_results = list_targets(&options.categories, &options.roots, exclude)?;
        print_list_results(&list_results);
        // Return empty report for --list mode
        Ok(ScanReport::new())
    } else {
        let report =
            scan_categories(&options.categories, &options.roots, options.verbose, exclude)?;
        print_report(&report, &options);
        Ok(report)
    }
}

fn scan_categories(
    categories: &[Category],
    roots: &[PathBuf],
    verbose: bool,
    exclude: Option<globset::GlobSet>,
) -> Result<ScanReport, AppError> {
    let scanners: Vec<Box<dyn CategoryScanner>> = vec![
        Box::new(XcodeScanner::new(exclude.clone())),
        Box::new(PythonScanner::new(exclude.clone())),
        Box::new(RustScanner::new(exclude.clone())),
        Box::new(NodejsScanner::new(exclude.clone())),
        Box::new(BrewScanner::new(exclude.clone())),
    ];

    // Filter scanners to only those requested
    let filtered_scanners: Vec<_> =
        scanners.into_iter().filter(|scanner| categories.contains(&scanner.category())).collect();

    // Run scanners in parallel
    let results: Result<Vec<_>, AppError> = filtered_scanners
        .par_iter()
        .map(|scanner| {
            let items = scanner.scan(roots, verbose)?;
            Ok((scanner.category(), items))
        })
        .collect();

    // Collect results into report
    let mut report = ScanReport::new();
    for (category, items) in results? {
        report.add_items(category, items);
    }

    Ok(report)
}

fn list_targets(
    categories: &[Category],
    roots: &[PathBuf],
    exclude: Option<globset::GlobSet>,
) -> Result<BTreeMap<Category, Vec<String>>, AppError> {
    let scanners: Vec<Box<dyn CategoryScanner>> = vec![
        Box::new(XcodeScanner::new(exclude.clone())),
        Box::new(PythonScanner::new(exclude.clone())),
        Box::new(RustScanner::new(exclude.clone())),
        Box::new(NodejsScanner::new(exclude.clone())),
        Box::new(BrewScanner::new(exclude.clone())),
    ];

    // Filter scanners to only those requested
    let filtered_scanners: Vec<_> =
        scanners.into_iter().filter(|scanner| categories.contains(&scanner.category())).collect();

    // Run scanners in parallel for listing
    let results: Result<Vec<_>, AppError> = filtered_scanners
        .par_iter()
        .map(|scanner| {
            let targets = scanner.list_targets(roots)?;
            Ok((scanner.category(), targets))
        })
        .collect();

    // Collect results into map
    let mut result_map = BTreeMap::new();
    for (category, targets) in results? {
        result_map.insert(category, targets);
    }

    Ok(result_map)
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

fn print_list_results(results: &BTreeMap<Category, Vec<String>>) {
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
