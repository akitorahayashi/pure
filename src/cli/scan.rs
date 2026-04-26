use std::path::PathBuf;

use clap::{ArgAction, Args};

use crate::error::AppError;
use crate::targets::catalog;
use crate::targets::category::Category;

#[derive(Args)]
pub struct ScanArgs {
    #[arg(short = 't', long = "type", value_name = "CATEGORY", action = ArgAction::Append, conflicts_with = "all")]
    pub categories: Vec<Category>,

    #[arg(long = "all", action = ArgAction::SetTrue)]
    pub all: bool,

    #[arg(short, long, action = ArgAction::SetTrue)]
    pub verbose: bool,

    #[arg(long = "list", action = ArgAction::SetTrue)]
    pub list: bool,

    #[arg(short = 'c', long = "current", action = ArgAction::SetTrue, conflicts_with = "paths")]
    pub current: bool,

    #[arg(value_name = "PATH", num_args = 0..)]
    pub paths: Vec<PathBuf>,
}

impl ScanArgs {
    pub fn resolve_categories(&self) -> Result<Vec<Category>, AppError> {
        let categories = if self.all || self.categories.is_empty() {
            catalog::categories_for_mode(self.current)
        } else {
            catalog::unique_categories(self.categories.clone())
        };

        if self.current {
            let unsupported = catalog::unsupported_for_current(&categories);
            if !unsupported.is_empty() {
                let names = unsupported
                    .iter()
                    .map(|category| category.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(AppError::UnsupportedCurrentModeCategory(names));
            }
        }

        Ok(categories)
    }
}
