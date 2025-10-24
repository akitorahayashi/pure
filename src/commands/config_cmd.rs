use std::path::Path;
use std::process::Command;

use crate::config::{Config, config_file_path, ensure_config_file};
use crate::error::AppError;
use crate::utils::display_path;

pub struct ConfigOptions {
    pub show_path: bool,
    pub edit: bool,
    pub add_exclude: Option<String>,
}

pub fn execute_config(options: ConfigOptions) -> Result<(), AppError> {
    if options.show_path {
        let path = config_file_path()?;
        println!("Configuration file: {}", display_path(&path));
    }

    if let Some(ref pattern) = options.add_exclude {
        let mut config = Config::load()?;
        config.append_exclude(pattern.clone());
        config.save()?;
        println!("Added exclude pattern '{}'.", pattern);
    }

    if options.edit {
        let path = ensure_config_file()?;
        open_editor(&path)?;
    }

    if !options.show_path && options.add_exclude.is_none() && !options.edit {
        let path = config_file_path()?;
        println!("Configuration file: {}", display_path(&path));
    }

    Ok(())
}

fn open_editor(path: &Path) -> Result<(), AppError> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nano".to_string());

    let status = Command::new(&editor)
        .arg(path)
        .status()
        .map_err(|err| AppError::Editor(err.to_string()))?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::Editor(format!("Editor exited with status {}", status)))
    }
}
