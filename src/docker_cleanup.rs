use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use byte_unit::Byte;
use serde_json;

use crate::error::AppError;
use crate::model::{Category, ScanItem};

const DOCKER_SCAN_LABEL: &str = "docker:prune";

fn is_docker_available() -> bool {
    Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn scan_docker(verbose: bool) -> Result<Vec<ScanItem>, AppError> {
    if !is_docker_available() {
        if verbose {
            println!("Docker CLI not available, skipping Docker scan.");
        }
        return Ok(Vec::new());
    }

    let output =
        Command::new("docker").args(["system", "df", "--format", "{{json .}}"]).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let message = if stderr.trim().is_empty() {
            format!("'docker system df' exited with status {}", output.status)
        } else {
            format!("'docker system df' failed: {}", stderr.trim())
        };
        if verbose {
            eprintln!("{message}");
        }
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut total = 0u64;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line)
            && let Some(reclaimable_str) = json.get("Reclaimable").and_then(|v| v.as_str())
            && let Some(size_str) = reclaimable_str.split(' ').next()
            && let Ok(byte) = size_str.parse::<Byte>()
        {
            total = total.saturating_add(byte.as_u64());
        }
    }

    if total == 0 {
        Ok(Vec::new())
    } else {
        Ok(vec![ScanItem::directory(Category::Docker, PathBuf::from(DOCKER_SCAN_LABEL), total)])
    }
}

pub fn run_docker_cleanup(verbose: bool) -> Result<(), AppError> {
    if !is_docker_available() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Docker CLI not available").into());
    }

    let args = &["system", "prune", "-a", "-f", "--volumes"];

    if verbose {
        println!("$ docker {}", args.join(" "));
    }

    let status = Command::new("docker").args(args.iter().copied()).status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "docker {} failed with status {}",
            args.join(" "),
            status
        ))
        .into());
    }

    Ok(())
}

pub fn list_targets_docker() -> Result<Vec<String>, AppError> {
    if !is_docker_available() {
        return Ok(Vec::new());
    }

    Ok(vec![
        "Unused images".to_string(),
        "Stopped containers".to_string(),
        "Unused volumes".to_string(),
        "Unused networks".to_string(),
        "Build cache".to_string(),
    ])
}
