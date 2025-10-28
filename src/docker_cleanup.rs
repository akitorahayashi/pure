use std::convert::TryFrom;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use byte_unit::Byte;

use crate::error::AppError;
use crate::model::{Category, ScanItem};

const DOCKER_SCAN_LABEL: &str = "docker:prune";

fn is_docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn parse_reclaimable(line: &str) -> Option<u64> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let mut idx = tokens.len() - 1;
    if tokens[idx].starts_with('(') && idx > 0 {
        idx -= 1;
    }

    let value = tokens.get(idx)?;
    let byte = Byte::parse_str(*value, false).ok()?;
    let bytes = byte.as_u128();
    u64::try_from(bytes).ok()
}

pub fn scan_docker(verbose: bool) -> Result<Vec<ScanItem>, AppError> {
    if !is_docker_available() {
        if verbose {
            println!("Docker CLI not available, skipping Docker scan.");
        }
        return Ok(Vec::new());
    }

    let output = Command::new("docker").args(["system", "df"]).output()?;
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

    let categories = ["Images", "Containers", "Local Volumes", "Build Cache"];

    for line in stdout.lines() {
        let line = line.trim();
        for category in &categories {
            if line.starts_with(category) {
                if let Some(bytes) = parse_reclaimable(line) {
                    total = total.saturating_add(bytes);
                }
                break; // Assuming only one category per line
            }
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

    let commands: &[&[&str]] = &[
        &["image", "prune", "-a", "-f"],
        &["container", "prune", "-f"],
        &["volume", "prune", "-f"],
        &["network", "prune", "-f"],
        &["builder", "prune", "-a", "-f"],
    ];

    for args in commands {
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
        "Dangling volumes".to_string(),
        "Unused networks".to_string(),
        "Build cache".to_string(),
    ])
}
