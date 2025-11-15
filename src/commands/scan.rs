use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::docker_cleanup::{list_targets_docker, scan_docker};
use crate::error::AppError;
use crate::format::format_bytes;
use crate::model::{Category, ItemKind, ScanItem, ScanReport};
use crate::path::{display_path, path_size};
use crate::scanners::*;

pub struct ScanOptions {
    pub categories: Vec<Category>,
    pub roots: Vec<PathBuf>,
    pub verbose: bool,
    pub list: bool,
    pub current: bool,
}

pub fn execute_scan(options: ScanOptions) -> Result<ScanReport, AppError> {
    let config = Config::load()?;
    let exclude = config.compile_excludes()?;

    if options.list {
        let list_results =
            list_targets(&options.categories, &options.roots, options.current, exclude)?;
        print_list_results(&list_results);
        // Return empty report for --list mode
        Ok(ScanReport::new())
    } else {
        let progress = Arc::new(MultiProgress::new());
        let report = scan_categories(
            &options.categories,
            &options.roots,
            options.verbose,
            options.current,
            exclude,
            &progress,
        )?;
        print_report(&report, &options);
        Ok(report)
    }
}

pub(crate) fn scan_categories(
    categories: &[Category],
    roots: &[PathBuf],
    verbose: bool,
    current: bool,
    exclude: Option<globset::GlobSet>,
    progress: &Arc<MultiProgress>,
) -> Result<ScanReport, AppError> {
    let fs_categories: Vec<_> =
        categories.iter().copied().filter(|category| *category != Category::Docker).collect();
    let should_scan_docker = categories.contains(&Category::Docker) && !current;

    if should_scan_docker {
        let (fs_result, docker_result) = rayon::join(
            || {
                run_filesystem_scan(
                    &fs_categories,
                    roots,
                    verbose,
                    current,
                    exclude.clone(),
                    progress,
                )
            },
            || scan_docker(verbose).map(Some),
        );

        let mut report = fs_result?;
        if let Some(docker_items) = docker_result?.filter(|items| !items.is_empty()) {
            report.add_items(Category::Docker, docker_items);
        }
        Ok(report)
    } else {
        run_filesystem_scan(&fs_categories, roots, verbose, current, exclude, progress)
    }
}

fn run_filesystem_scan(
    fs_categories: &[Category],
    roots: &[PathBuf],
    verbose: bool,
    current: bool,
    exclude: Option<globset::GlobSet>,
    progress: &Arc<MultiProgress>,
) -> Result<ScanReport, AppError> {
    if fs_categories.is_empty() {
        return Ok(ScanReport::new());
    }

    let scanners = get_scanners(exclude.clone(), current);
    let filtered_scanners: Vec<_> = scanners
        .into_iter()
        .filter(|scanner| fs_categories.contains(&scanner.category()))
        .collect();

    if filtered_scanners.is_empty() {
        return Ok(ScanReport::new());
    }

    let discovery_style = Arc::new(discovery_spinner_style());
    let discovery_progress = Arc::clone(progress);
    let discovery_results: Result<Vec<Vec<ScanItem>>, AppError> = filtered_scanners
        .par_iter()
        .map(|scanner| {
            let spinner = discovery_progress.add(ProgressBar::new_spinner());
            spinner.set_style((*discovery_style).clone());
            spinner.enable_steady_tick(Duration::from_millis(100));
            spinner.set_message(format!(
                "Discovering targets... ({})",
                scanner.category().display_name()
            ));
            let items = scanner.scan(roots, verbose)?;
            let count = items.len();
            spinner.finish_and_clear();
            discovery_progress.println(format!(
                "✔︎ {} discovery complete ({} item{})",
                scanner.category().display_name(),
                count,
                if count == 1 { "" } else { "s" }
            )).unwrap();
            Ok(items)
        })
        .collect();

    let mut discovered_items: Vec<ScanItem> = discovery_results?.into_iter().flatten().collect();
    if discovered_items.is_empty() {
        return Ok(ScanReport::new());
    }

    let total_items = discovered_items.len();
    let size_bar = progress.add(ProgressBar::new(total_items as u64));
    size_bar.set_style(size_progress_style());
    compute_sizes_parallel(&mut discovered_items, exclude.as_ref(), verbose, Some(&size_bar))?;
    size_bar.finish_and_clear();
    progress.println(format!(
        "{}/{} Size calculation complete ({} item{})",
        total_items, total_items,
        total_items,
        if total_items == 1 { "" } else { "s" }
    )).unwrap();

    let mut grouped: BTreeMap<Category, Vec<ScanItem>> = BTreeMap::new();
    for item in discovered_items {
        grouped.entry(item.category).or_default().push(item);
    }

    let mut report = ScanReport::new();
    for (category, items) in grouped {
        if !items.is_empty() {
            report.add_items(category, items);
        }
    }

    Ok(report)
}

fn compute_sizes_parallel(
    items: &mut [ScanItem],
    exclude: Option<&globset::GlobSet>,
    verbose: bool,
    progress: Option<&ProgressBar>,
) -> Result<(), AppError> {
    items.par_iter_mut().try_for_each(|item| {
        if item.size == 0 {
            item.size = match item.kind {
                ItemKind::Directory => path_size(&item.path, exclude, verbose)?,
                ItemKind::File => item.path.metadata()?.len(),
            };
        }
        if let Some(pb) = progress {
            pb.inc(1);
        }
        Ok(())
    })
}

fn discovery_spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.cyan} {msg}").unwrap().tick_chars("|/-\\")
}

fn size_progress_style() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>6}/{len:>6}")
        .unwrap()
        .progress_chars("=|-")
}

fn list_targets(
    categories: &[Category],
    roots: &[PathBuf],
    current: bool,
    exclude: Option<globset::GlobSet>,
) -> Result<BTreeMap<Category, Vec<String>>, AppError> {
    let docker_list = categories.contains(&Category::Docker);
    let fs_categories: Vec<_> =
        categories.iter().copied().filter(|category| *category != Category::Docker).collect();

    let scanners = get_scanners(exclude.clone(), current);

    // Filter scanners to only those requested
    let filtered_scanners: Vec<_> = scanners
        .into_iter()
        .filter(|scanner| fs_categories.contains(&scanner.category()))
        .collect();

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

    if docker_list && !current {
        let docker_targets = list_targets_docker()?;
        if !docker_targets.is_empty() {
            result_map.insert(Category::Docker, docker_targets);
        }
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

pub fn get_scanners(
    exclude: Option<globset::GlobSet>,
    current: bool,
) -> Vec<Box<dyn CategoryScanner>> {
    let mut scanners: Vec<Box<dyn CategoryScanner>> = vec![
        Box::new(XcodeScanner::new(exclude.clone(), current)),
        Box::new(PythonScanner::new(exclude.clone())),
        Box::new(RustScanner::new(exclude.clone())),
        Box::new(NodejsScanner::new(exclude.clone())),
    ];

    // Only add BrewScanner if not scanning current directory
    if !current {
        scanners.push(Box::new(BrewScanner::new(exclude.clone())));
    }

    scanners
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;

    #[test]
    fn compute_sizes_parallel_assigns_sizes() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("node_modules");
        dir.child("lib").create_dir_all().unwrap();
        dir.child("lib/index.js").write_str("console.log('cache');").unwrap();
        let file = temp.child("cache.log");
        file.write_str("hello").unwrap();

        let mut items = vec![
            ScanItem::directory(Category::Nodejs, dir.path().to_path_buf(), 0),
            ScanItem::file(Category::Nodejs, file.path().to_path_buf(), 0),
        ];

        compute_sizes_parallel(&mut items, None, false, None).expect("size calculation succeeds");

        assert!(
            items.iter().all(|item| item.size > 0),
            "expected non-zero sizes after measurement"
        );
    }
}
