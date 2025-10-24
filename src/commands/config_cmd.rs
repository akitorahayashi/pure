use std::path::Path;
use std::process::Command;

use crate::config::{config_file_path, ensure_config_file};
use crate::error::AppError;
use crate::path::display_path;

pub struct ConfigOptions {
    pub show_path: bool,
    pub edit: bool,
}

pub fn execute_config(options: ConfigOptions) -> Result<(), AppError> {
    if options.show_path {
        let path = config_file_path()?;
        println!("Configuration file: {}", display_path(&path));
    }

    if options.edit {
        let path = ensure_config_file()?;
        open_editor(&path)?;
    }

    if !options.show_path && !options.edit {
        let path = config_file_path()?;
        println!("Configuration file: {}", display_path(&path));
    }

    Ok(())
}

fn open_editor(path: &Path) -> Result<(), AppError> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nano".to_string());

    let mut parts = editor.split_whitespace();
    let prog = parts.next().ok_or_else(|| AppError::Editor("EDITOR was empty".into()))?;
    let args: Vec<&str> = parts.collect();
    let status = Command::new(prog)
        .args(args)
        .arg(path)
        .status()
        .map_err(|err| AppError::Editor(err.to_string()))?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::Editor(format!("Editor exited with status {}", status)))
    }
}
