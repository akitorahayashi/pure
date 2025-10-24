use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand};
use pure::commands::{config_cmd::ConfigOptions, run::RunOptions, scan::ScanOptions};
use pure::commands::{execute_config, execute_run, execute_scan};
use pure::error::AppError;
use pure::model::Category;
use pure::utils::resolve_roots;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan(args) => {
            let categories = resolve_categories(args.categories, args.all);
            let options = ScanOptions {
                categories,
                roots: resolve_roots(&args.paths),
                verbose: args.verbose,
            };
            execute_scan(options)?;
        }
        Commands::Run(args) => {
            let categories =
                if args.all || args.categories.is_empty() { None } else { Some(args.categories) };
            let options = RunOptions {
                categories,
                all: args.all,
                roots: resolve_roots(&args.paths),
                verbose: args.verbose,
                assume_yes: args.yes,
            };
            execute_run(options)?;
        }
        Commands::Config(args) => {
            let options = ConfigOptions {
                show_path: args.path,
                edit: args.edit,
                add_exclude: args.add_exclude,
            };
            execute_config(options)?;
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(name = "pure", version, about = "Safely clean macOS caches from the terminal.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Perform a dry-run scan to see what can be removed.
    Scan(ScanArgs),
    /// Delete files discovered by a scan.
    Run(RunArgs),
    /// Manage pure configuration (exclusions, etc.).
    Config(ConfigArgs),
}

#[derive(Args)]
struct ScanArgs {
    /// Restrict the scan to specific categories (e.g. dev, system).
    #[arg(short = 't', long = "type", value_name = "CATEGORY", action = ArgAction::Append, conflicts_with = "all")]
    categories: Vec<Category>,

    /// Scan all categories (default when no type is provided).
    #[arg(long = "all", action = ArgAction::SetTrue)]
    all: bool,

    /// Show every item that would be removed.
    #[arg(short, long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Optional paths to scan (defaults to $HOME).
    #[arg(value_name = "PATH", num_args = 0..)]
    paths: Vec<PathBuf>,
}

#[derive(Args)]
struct RunArgs {
    /// Delete specific categories without entering interactive mode.
    #[arg(short = 't', long = "type", value_name = "CATEGORY", action = ArgAction::Append, conflicts_with = "all")]
    categories: Vec<Category>,

    /// Delete all categories without prompting for selection.
    #[arg(long = "all", action = ArgAction::SetTrue)]
    all: bool,

    /// Skip the confirmation prompt.
    #[arg(short = 'y', long = "yes", action = ArgAction::SetTrue)]
    yes: bool,

    /// Show each deleted item.
    #[arg(short, long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Optional paths to operate on (defaults to $HOME).
    #[arg(value_name = "PATH", num_args = 0..)]
    paths: Vec<PathBuf>,
}

#[derive(Args)]
struct ConfigArgs {
    /// Show the configuration file path.
    #[arg(long = "path", action = ArgAction::SetTrue)]
    path: bool,

    /// Open the configuration file in $EDITOR.
    #[arg(long = "edit", action = ArgAction::SetTrue)]
    edit: bool,

    /// Append a new exclude pattern to the configuration file.
    #[arg(long = "add-exclude", value_name = "PATTERN")]
    add_exclude: Option<String>,
}

fn resolve_categories(categories: Vec<Category>, all: bool) -> Vec<Category> {
    if all || categories.is_empty() { Category::ALL.to_vec() } else { categories }
}
