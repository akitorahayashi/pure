use std::io::{self, Write};
use std::path::PathBuf;

use crate::commands::scan::get_scanners;
use crate::config::Config;
use crate::error::AppError;
use crate::format::format_bytes;
use crate::model::{Category, ScanItem, ScanReport};
use crate::path::{display_path, is_excluded, safe_remove_dir_all};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

pub struct RunOptions {
    pub categories: Option<Vec<Category>>,
    pub all: bool,
    pub roots: Vec<PathBuf>,
    pub verbose: bool,
    pub assume_yes: bool,
    pub current: bool,
}

pub fn execute_run(options: RunOptions) -> Result<(), AppError> {
    let config = Config::load()?;
    let exclude = config.compile_excludes()?;

    let scan_categories = if options.all {
        Category::ALL.to_vec()
    } else if let Some(explicit) = &options.categories {
        explicit.clone()
    } else {
        Category::ALL.to_vec()
    };

    let report = scan_categories_for_run(
        &scan_categories,
        &options.roots,
        options.verbose,
        options.current,
        exclude.clone(),
    )?;

    let selected_categories = if options.all {
        Category::ALL.to_vec()
    } else if let Some(explicit) = options.categories.clone() {
        explicit
    } else {
        match prompt_for_categories(&report) {
            Ok(categories) => categories,
            Err(AppError::Cancelled) => {
                println!("Aborted. No files were deleted.");
                return Ok(());
            }
            Err(err) => return Err(err),
        }
    };

    let subset = report.subset(&selected_categories);

    if subset.total_size() == 0 {
        println!("Nothing to delete. All selected categories are already clean.");
        return Ok(());
    }

    print_summary(&subset, options.verbose);

    if !options.assume_yes && !confirm_deletion(subset.total_size())? {
        println!("Aborted. No files were deleted.");
        return Ok(());
    }

    let items_to_delete: Vec<ScanItem> =
        subset.categories.values().flat_map(|report| &report.items).cloned().collect();

    delete_items(&items_to_delete, exclude)?;

    println!(
        "Attempted to delete {} across {} categor(ies).",
        format_bytes(subset.total_size()),
        selected_categories.len()
    );

    Ok(())
}

fn prompt_for_categories(report: &ScanReport) -> Result<Vec<Category>, AppError> {
    println!(
        "Select categories to delete (comma separated names or numbers). Type 'all' to select everything or press Enter to cancel."
    );
    for (index, category) in Category::ALL.iter().enumerate() {
        let report = report.report_for(*category);
        let size = report.map(|r| r.total_size()).unwrap_or_default();
        println!("  [{}] {:<8} {:>10}", index + 1, category, format_bytes(size));
    }
    print!("Selection: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::Cancelled);
    }
    if trimmed.eq_ignore_ascii_case("all") {
        return Ok(Category::ALL.to_vec());
    }

    let mut selected = Vec::new();
    for token in trimmed.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }

        if let Ok(index) = token.parse::<usize>()
            && index >= 1
            && index <= Category::ALL.len()
        {
            let category = Category::ALL[index - 1];
            if !selected.contains(&category) {
                selected.push(category);
            }
            continue;
        }

        if let Ok(category) = token.parse::<Category>() {
            if !selected.contains(&category) {
                selected.push(category);
            }
            continue;
        }

        return Err(AppError::InvalidCategory(token.to_string()));
    }

    if selected.is_empty() {
        return Err(AppError::Cancelled);
    }

    Ok(selected)
}

fn confirm_deletion(total_size: u64) -> Result<bool, AppError> {
    println!("About to delete {}. Proceed? [y/N]", format_bytes(total_size));
    print!("Confirm: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_ascii_lowercase();
    Ok(matches!(answer.as_str(), "y" | "yes"))
}

fn print_summary(report: &ScanReport, verbose: bool) {
    println!("Deletion plan:");
    for category in report.categories() {
        if let Some(category_report) = report.report_for(category) {
            println!(
                "- {:<8} {:>10} across {} item(s)",
                category,
                format_bytes(category_report.total_size()),
                category_report.items.len()
            );
            // Always show file paths for transparency - this addresses the "I don't know what's being deleted" problem
            for item in &category_report.items {
                if verbose {
                    println!(
                        "    • {:<60} {}",
                        display_path(item.path_str()),
                        format_bytes(item.size)
                    );
                } else {
                    // Show path even in non-verbose mode for transparency
                    println!("    • {}", display_path(item.path_str()));
                }
            }
        }
    }
    println!("Total to delete: {}", format_bytes(report.total_size()));
}

fn scan_categories_for_run(
    categories: &[Category],
    roots: &[std::path::PathBuf],
    verbose: bool,
    current: bool,
    exclude: Option<globset::GlobSet>,
) -> Result<ScanReport, AppError> {
    let scanners = get_scanners(exclude.clone(), current);

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

fn delete_items(items: &[ScanItem], exclude: Option<globset::GlobSet>) -> Result<(), AppError> {
    use crate::model::ItemKind;
    use std::fs;
    use std::io;

    if items.is_empty() {
        return Ok(());
    }

    // Helper function to check exclusions
    // Create progress bar with light blue style
    let pb = ProgressBar::new(items.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    for item in items.iter() {
        if is_excluded(&item.path, exclude.as_ref()) {
            pb.inc(1);
            continue;
        }

        // Update progress bar with current file
        pb.set_message(format!(
            "Deleting {} ({})",
            display_path(item.path_str()),
            format_bytes(item.size)
        ));

        match item.kind {
            ItemKind::Directory => {
                if let Err(err) = safe_remove_dir_all(&item.path, exclude.as_ref(), false) {
                    pb.finish_and_clear();
                    return Err(err);
                }
            }
            ItemKind::File => {
                if let Err(err) = fs::remove_file(&item.path)
                    && err.kind() != io::ErrorKind::NotFound
                {
                    pb.finish_and_clear();
                    return Err(AppError::Io(err));
                }
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Deletion complete");
    Ok(())
}
