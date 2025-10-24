use std::fs;
use std::io::Write;
use std::path::PathBuf;

use dirs_next as dirs;
use globset::{Glob, GlobSet};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        let path = config_file_path()?;
        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<(), AppError> {
        let path = config_file_path()?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut file = fs::File::create(path)?;
        let contents = toml::to_string_pretty(self)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn append_exclude(&mut self, value: String) {
        if !self.exclude.iter().any(|existing| existing == &value) {
            self.exclude.push(value);
        }
    }

    pub fn compile_excludes(&self) -> Result<Option<GlobSet>, AppError> {
        if self.exclude.is_empty() {
            return Ok(None);
        }

        let mut builder = globset::GlobSetBuilder::new();
        for pattern in &self.exclude {
            let expanded = expand_home(pattern)?;
            builder.add(Glob::new(&expanded)?);
        }

        Ok(Some(builder.build()?))
    }
}

pub fn config_file_path() -> Result<PathBuf, AppError> {
    let config_root = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(dirs::config_dir)
        .ok_or_else(|| {
            AppError::config("Unable to determine configuration directory for this platform")
        })?;
    Ok(config_root.join("pure").join("config.toml"))
}

pub fn ensure_config_file() -> Result<PathBuf, AppError> {
    let path = config_file_path()?;
    if !path.exists() {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let default = Config::default();
        let contents = toml::to_string_pretty(&default)?;
        fs::write(&path, contents)?;
    }
    Ok(path)
}

fn expand_home(value: &str) -> Result<String, AppError> {
    if !value.starts_with('~') {
        return Ok(value.to_string());
    }
    let home_dir = dirs::home_dir().ok_or_else(|| {
        AppError::config("Unable to expand '~' because the home directory is unknown")
    })?;
    if value == "~" {
        Ok(home_dir.display().to_string())
    } else if let Some(stripped) = value.strip_prefix("~/") {
        Ok(home_dir.join(stripped).display().to_string())
    } else {
        Ok(value.to_string())
    }
}
