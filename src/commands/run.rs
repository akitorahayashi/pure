use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;

use crate::commands::scan::scan_categories;
use crate::config::Config;
use crate::docker_cleanup::run_docker_cleanup;
use crate::error::AppError;
use crate::format::format_bytes;
use crate::model::{Category, ScanItem, ScanReport};
use crate::path::{display_path, is_excluded, safe_remove_dir_all};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
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

    let debug_logging = std::env::var_os("PURE_DEBUG").is_some();
    let requested_categories = if options.all {
        Category::ALL.to_vec()
    } else if let Some(explicit) = &options.categories {
        explicit.clone()
    } else {
        Category::ALL.to_vec()
    };

    let progress = Arc::new(MultiProgress::new());
    let report = scan_categories(
        &requested_categories,
        &options.roots,
        options.verbose,
        options.current,
        exclude.clone(),
        &progress,
    )?;
    if debug_logging {
        eprintln!("[pure::run] finished scan phase");
    }

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
    if debug_logging {
        eprintln!("[pure::run] printed summary, awaiting confirmation");
    }

    if !options.assume_yes && !confirm_deletion(subset.total_size())? {
        println!("Aborted. No files were deleted.");
        return Ok(());
    }
    if debug_logging {
        eprintln!("[pure::run] confirmation obtained");
    }

    let items_to_delete: Vec<ScanItem> =
        subset.categories.values().flat_map(|report| &report.items).cloned().collect();

    let fs_items_to_delete: Vec<ScanItem> =
        items_to_delete.into_iter().filter(|item| item.category != Category::Docker).collect();

    let needs_docker_cleanup = selected_categories.contains(&Category::Docker) && !options.current;

    if debug_logging {
        eprintln!("[pure::run] starting deletion (docker_cleanup={})", needs_docker_cleanup);
    }
    if needs_docker_cleanup {
        let delete_progress = Arc::clone(&progress);
        let (delete_result, docker_result) = rayon::join(
            || delete_items(&fs_items_to_delete, exclude.clone(), &delete_progress),
            || run_docker_cleanup_with_handling(options.verbose),
        );
        delete_result?;
        docker_result?;
    } else {
        delete_items(&fs_items_to_delete, exclude, &progress)?;
    }
    if debug_logging {
        eprintln!("[pure::run] deletion phase complete");
    }

    println!(
        "Attempted to delete {} across {} categor(ies).",
        format_bytes(subset.total_size()),
        selected_categories.len()
    );

    Ok(())
}

fn run_docker_cleanup_with_handling(verbose: bool) -> Result<(), AppError> {
    match run_docker_cleanup(verbose) {
        Ok(()) => Ok(()),
        Err(err) => {
            if let Some(io_err) = err.source().and_then(|e| e.downcast_ref::<std::io::Error>())
                && io_err.kind() == std::io::ErrorKind::NotFound
            {
                if verbose {
                    eprintln!("Docker CLI not available; skipping Docker cleanup.");
                }
                return Ok(());
            }
            Err(err)
        }
    }
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

fn delete_items(
    items: &[ScanItem],
    exclude: Option<globset::GlobSet>,
    progress: &Arc<MultiProgress>,
) -> Result<(), AppError> {
    use crate::model::ItemKind;
    use std::fs;
    use std::io;

    if items.is_empty() {
        return Ok(());
    }

    let pb = progress.add(ProgressBar::new(items.len() as u64));
    pb.set_style(deletion_progress_style());

    let exclude_ref = exclude.as_ref();
    items.par_iter().try_for_each(|item| {
        if is_excluded(&item.path, exclude_ref) {
            pb.inc(1);
            return Ok(());
        }

        pb.set_message(display_path(&item.path));

        match item.kind {
            ItemKind::Directory => {
                safe_remove_dir_all(&item.path, exclude_ref, false)?;
            }
            ItemKind::File => match fs::remove_file(&item.path) {
                Ok(()) => {}
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => return Err(AppError::Io(err)),
            },
        }

        pb.inc(1);
        Ok(())
    })?;

    pb.finish_and_clear();
    let _ = progress.println(format!("{}/{} Deletion complete", items.len(), items.len()));
    Ok(())
}

fn deletion_progress_style() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>6}/{len:>6} {msg}")
        .unwrap()
        .progress_chars("=|-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;

    #[test]
    fn delete_items_removes_files_and_directories() {
        let temp = TempDir::new().unwrap();
        let dir = temp.child("node_modules");
        dir.child("lib").create_dir_all().unwrap();
        dir.child("lib/index.js").write_str("console.log('cache');").unwrap();
        let file = temp.child("cache.log");
        file.write_str("hello").unwrap();

        let items = vec![
            ScanItem::directory(Category::Nodejs, dir.path().to_path_buf(), 0),
            ScanItem::file(Category::Nodejs, file.path().to_path_buf(), 0),
        ];

        let progress = Arc::new(MultiProgress::new());
        delete_items(&items, None, &progress).expect("deletion succeeds");

        dir.assert(predicates::path::missing());
        file.assert(predicates::path::missing());
    }

    #[test]
    fn delete_items_respects_exclusions() {
        let temp = TempDir::new().unwrap();
        let skip_dir = temp.child("skip");
        skip_dir.create_dir_all().unwrap();
        skip_dir.child("data.txt").write_str("cache").unwrap();
        let remove_dir = temp.child("remove-me");
        remove_dir.create_dir_all().unwrap();
        remove_dir.child("data.txt").write_str("cache").unwrap();

        let mut builder = globset::GlobSetBuilder::new();
        builder.add(globset::Glob::new("**/skip/**").unwrap());
        let exclude = Some(builder.build().unwrap());

        let items = vec![
            ScanItem::directory(Category::Nodejs, skip_dir.path().to_path_buf(), 0),
            ScanItem::directory(Category::Nodejs, remove_dir.path().to_path_buf(), 0),
        ];

        let progress = Arc::new(MultiProgress::new());
        delete_items(&items, exclude, &progress).expect("deletion succeeds");

        skip_dir.assert(predicates::path::exists());
        remove_dir.assert(predicates::path::missing());
    }
}
